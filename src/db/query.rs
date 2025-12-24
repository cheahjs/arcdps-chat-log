use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};

use anyhow::Context;
use chrono::TimeZone;
use log::error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use super::ChatDatabase;

pub enum DbQuery {
    Note(String),
    SearchMessages(SearchQuery),
}

/// Parameters for searching chat messages
#[derive(Clone, Debug)]
pub struct SearchQuery {
    /// Text to search for (searches account_name, character_name, and text)
    pub search_text: String,
    /// Filter by channel type (e.g., "Squad", "Party")
    pub channel_type: Option<String>,
    /// Filter by account name
    pub account_name: Option<String>,
    /// Minimum timestamp (Unix timestamp)
    pub timestamp_min: Option<i64>,
    /// Maximum timestamp (Unix timestamp)
    pub timestamp_max: Option<i64>,
    /// Number of results to return per batch
    pub batch_size: u32,
    /// Offset for pagination (for streaming results)
    pub offset: u32,
    /// Unique search ID to correlate results
    pub search_id: u64,
}

impl SearchQuery {
    pub fn new(search_text: String, search_id: u64) -> Self {
        Self {
            search_text,
            channel_type: None,
            account_name: None,
            timestamp_min: None,
            timestamp_max: None,
            batch_size: 50,
            offset: 0,
            search_id,
        }
    }
}

/// A single search result message
#[derive(Clone, Debug)]
pub struct SearchResultMessage {
    pub channel_type: String,
    pub subgroup: i32,
    pub is_broadcast: bool,
    pub timestamp: i64,
    pub account_name: String,
    pub character_name: String,
    pub text: String,
}

impl SearchResultMessage {
    pub fn timestamp_datetime(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Utc
            .timestamp_opt(self.timestamp, 0)
            .unwrap()
            .with_timezone(&chrono::Local)
    }
}

/// Container for search results with metadata
#[derive(Clone, Debug)]
pub struct SearchResults {
    /// The search ID this result corresponds to
    pub search_id: u64,
    /// The messages found
    pub messages: Vec<SearchResultMessage>,
    /// Whether there are more results available
    pub has_more: bool,
    /// Total count of matching messages (if available)
    pub total_count: Option<u64>,
    /// Current offset
    pub offset: u32,
}

/// State of a search operation
#[derive(Clone, Debug)]
pub enum SearchState {
    /// No search in progress
    Idle,
    /// Search is in progress
    Searching(u64),
    /// Search completed with results
    Results(SearchResults),
    /// Search failed with error message
    Error(String),
}

#[derive(Clone)]
pub struct Note {
    pub account_name: String,
    pub note: String,
    pub note_added: i64,
    pub note_updated: i64,
    pub color: Option<[f32; 3]>,
}

impl Note {
    pub fn note_added(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Utc
            .timestamp_opt(self.note_added, 0)
            .unwrap()
            .with_timezone(&chrono::Local)
    }

    pub fn note_updated(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Utc
            .timestamp_opt(self.note_updated, 0)
            .unwrap()
            .with_timezone(&chrono::Local)
    }
}

#[derive(Clone)]
pub enum QueriedNote {
    Success(Note),
    Error,
    NotFound,
    Pending,
}

impl ChatDatabase {
    /// Returns the note associated with the `account_name`.
    /// Returns:
    /// - `QueriedNote::Success` if a note is in the cache
    /// - `QueriedNote::Error` if the query failed
    /// - `QueriedNote::NotFound` if a note was not found
    /// - `QueriedNote::Pending` if `account_name` is not in the cache and is waiting for a query
    pub fn get_or_query_note(&mut self, account_name: &str) -> QueriedNote {
        if let Some(note) = self.note_cache.lock().unwrap().get(account_name) {
            return note.clone();
        }
        // Account not found in cache, start a query
        // Put marker in cache to prevent spawning queries per-frame
        self.note_cache
            .lock()
            .unwrap()
            .insert(account_name.to_owned(), QueriedNote::Pending);
        if let Some(query_channel) = &self.query_channel {
            if let Err(err) = query_channel
                .lock()
                .unwrap()
                .send(DbQuery::Note(account_name.to_owned()))
            {
                error!("failed to send query: {:#}", err);
            }
        }
        QueriedNote::Pending
    }

    /// Initiates a search query for messages
    pub fn search_messages(&self, query: SearchQuery) {
        if let Some(query_channel) = &self.query_channel {
            // Mark search as in progress
            {
                let mut search_state = self.search_state.lock().unwrap();
                *search_state = SearchState::Searching(query.search_id);
            }
            if let Err(err) = query_channel
                .lock()
                .unwrap()
                .send(DbQuery::SearchMessages(query))
            {
                error!("failed to send search query: {:#}", err);
                let mut search_state = self.search_state.lock().unwrap();
                *search_state = SearchState::Error(format!("Failed to send query: {}", err));
            }
        }
    }

    /// Get the current search state
    pub fn get_search_state(&self) -> SearchState {
        self.search_state.lock().unwrap().clone()
    }

    /// Clear search results
    pub fn clear_search(&self) {
        let mut search_state = self.search_state.lock().unwrap();
        *search_state = SearchState::Idle;
    }

    pub(crate) fn query_thread(
        pool: Pool<SqliteConnectionManager>,
        recv_chan: mpsc::Receiver<DbQuery>,
        note_cache: Arc<Mutex<HashMap<String, QueriedNote>>>,
        search_state: Arc<Mutex<SearchState>>,
    ) -> anyhow::Result<()> {
        let connection = pool.get().context("failed to get database connection")?;
        loop {
            let event = recv_chan.recv();
            if let Ok(query) = event {
                match query {
                    DbQuery::Note(account_name) => {
                        let mut statement = connection
                            .prepare_cached(
                                "SELECT account_name, note, note_added, note_updated, color1, color2, color3 FROM notes
                                WHERE account_name=?1 LIMIT 1",
                            )
                            .context("failed to prepare statement")?;
                        let note_iter = statement
                            .query_map(params![account_name], |row| {
                                let color1: Option<f32> = row.get(4)?;
                                let color2: Option<f32> = row.get(5)?;
                                let color3: Option<f32> = row.get(6)?;
                                #[allow(clippy::unnecessary_unwrap)]
                                let color =
                                    if color1.is_none() || color2.is_none() || color3.is_none() {
                                        None
                                    } else {
                                        Some([color1.unwrap(), color2.unwrap(), color3.unwrap()])
                                    };
                                Ok(Note {
                                    account_name: row.get(0)?,
                                    note: row.get(1)?,
                                    note_added: row.get(2)?,
                                    note_updated: row.get(3)?,
                                    color,
                                })
                            })
                            .context("failed to query note")?;
                        let mut found = false;
                        for note in note_iter {
                            found = true;
                            match note {
                                Ok(note) => {
                                    note_cache.lock().unwrap().insert(
                                        account_name.to_owned(),
                                        QueriedNote::Success(note),
                                    );
                                }
                                Err(err) => {
                                    error!("failed to query note: {:#}", err);
                                    note_cache
                                        .lock()
                                        .unwrap()
                                        .insert(account_name.to_owned(), QueriedNote::Error);
                                }
                            }
                        }
                        if !found {
                            note_cache
                                .lock()
                                .unwrap()
                                .insert(account_name.to_owned(), QueriedNote::NotFound);
                        }
                    }
                    DbQuery::SearchMessages(search_query) => {
                        match Self::execute_search(&connection, &search_query) {
                            Ok(results) => {
                                let mut state = search_state.lock().unwrap();
                                // Only update if this is still the current search
                                if let SearchState::Searching(id) = &*state {
                                    if *id == search_query.search_id {
                                        *state = SearchState::Results(results);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("failed to execute search: {:#}", err);
                                let mut state = search_state.lock().unwrap();
                                *state = SearchState::Error(format!("Search failed: {}", err));
                            }
                        }
                    }
                }
            } else if let Err(err) = event {
                return Err(anyhow::Error::new(err).context("failed to receive query event"));
            }
        }
    }

    fn execute_search(
        connection: &r2d2::PooledConnection<SqliteConnectionManager>,
        query: &SearchQuery,
    ) -> anyhow::Result<SearchResults> {
        // Build the WHERE clause dynamically
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Text search across multiple fields
        if !query.search_text.is_empty() {
            conditions.push(
                "(account_name LIKE ?1 OR character_name LIKE ?1 OR text LIKE ?1)".to_string(),
            );
            param_values.push(Box::new(format!("%{}%", query.search_text)));
        }

        // Channel type filter
        if let Some(channel_type) = &query.channel_type {
            let param_idx = param_values.len() + 1;
            conditions.push(format!("channel_type = ?{}", param_idx));
            param_values.push(Box::new(channel_type.clone()));
        }

        // Account name filter
        if let Some(account_name) = &query.account_name {
            let param_idx = param_values.len() + 1;
            conditions.push(format!("account_name LIKE ?{}", param_idx));
            param_values.push(Box::new(format!("%{}%", account_name)));
        }

        // Timestamp filters
        if let Some(ts_min) = query.timestamp_min {
            let param_idx = param_values.len() + 1;
            conditions.push(format!("timestamp >= ?{}", param_idx));
            param_values.push(Box::new(ts_min));
        }
        if let Some(ts_max) = query.timestamp_max {
            let param_idx = param_values.len() + 1;
            conditions.push(format!("timestamp <= ?{}", param_idx));
            param_values.push(Box::new(ts_max));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Query for batch_size + 1 results to determine if there are more
        // This avoids an expensive COUNT(*) query that would scan all matching rows
        let limit_param_idx = param_values.len() + 1;
        let offset_param_idx = param_values.len() + 2;

        let results_sql = format!(
            "SELECT channel_type, subgroup, is_broadcast, timestamp, account_name, character_name, text 
             FROM messages {} 
             ORDER BY timestamp DESC 
             LIMIT ?{} OFFSET ?{}",
            where_clause, limit_param_idx, offset_param_idx
        );

        // Add limit (batch_size + 1 to check for more) and offset params
        let mut all_params: Vec<Box<dyn rusqlite::ToSql>> = param_values;
        all_params.push(Box::new((query.batch_size + 1) as i64));
        all_params.push(Box::new(query.offset as i64));

        let all_params_refs: Vec<&dyn rusqlite::ToSql> =
            all_params.iter().map(|p| p.as_ref()).collect();

        let mut statement = connection
            .prepare(&results_sql)
            .context("failed to prepare search statement")?;

        let results_iter = statement
            .query_map(all_params_refs.as_slice(), |row| {
                Ok(SearchResultMessage {
                    channel_type: row.get(0)?,
                    subgroup: row.get(1)?,
                    is_broadcast: row.get(2)?,
                    timestamp: row.get(3)?,
                    account_name: row.get(4)?,
                    character_name: row.get(5)?,
                    text: row.get(6)?,
                })
            })
            .context("failed to execute search query")?;

        let mut messages = Vec::new();
        for result in results_iter {
            match result {
                Ok(msg) => messages.push(msg),
                Err(err) => error!("failed to read search result row: {:#}", err),
            }
        }

        // If we got more than batch_size results, there are more to fetch
        let has_more = messages.len() > query.batch_size as usize;

        // Only return batch_size results (remove the extra one we fetched to check has_more)
        if has_more {
            messages.pop();
        }

        Ok(SearchResults {
            search_id: query.search_id,
            messages,
            has_more,
            total_count: None, // No longer computing total count
            offset: query.offset,
        })
    }
}
