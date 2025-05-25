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
    SearchMessages {
        query: String,
        batch_size: usize,
        sender: mpsc::Sender<Vec<ArchivedMessage>>,
    },
}

#[derive(Clone, Debug)]
pub struct ArchivedMessage {
    pub timestamp: i64,
    pub account_name: String,
    pub character_name: String,
    pub text: String,
    pub channel_type: i32,
    pub subgroup: i32,
    pub is_broadcast: bool,
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

    pub(crate) fn query_thread(
        pool: Pool<SqliteConnectionManager>,
        recv_chan: mpsc::Receiver<DbQuery>,
        note_cache: Arc<Mutex<HashMap<String, QueriedNote>>>,
    ) -> anyhow::Result<()> {
        let connection = pool.get().context("failed to get database connection")?;
        loop {
            let event = recv_chan.recv();
            if let Ok(insert) = event {
                match insert {
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
                    DbQuery::SearchMessages {
                        query,
                        batch_size,
                        sender,
                    } => {
                        let sql = "SELECT timestamp, account_name, character_name, text, channel_type, subgroup, is_broadcast
                                   FROM messages
                                   WHERE account_name LIKE ?1 OR character_name LIKE ?2 OR text LIKE ?3
                                   ORDER BY timestamp DESC";
                        match connection.prepare_cached(sql) {
                            Ok(mut statement) => {
                                let like_query = format!("%{}%", query);
                                let params = params![like_query, like_query, like_query];
                                let messages_iter =
                                    statement.query_map(params, |row| {
                                        Ok(ArchivedMessage {
                                            timestamp: row.get(0)?,
                                            account_name: row.get(1)?,
                                            character_name: row.get(2)?,
                                            text: row.get(3)?,
                                            channel_type: row.get(4)?,
                                            subgroup: row.get(5)?,
                                            is_broadcast: row.get(6)?,
                                        })
                                    });

                                match messages_iter {
                                    Ok(rows) => {
                                        let mut batch = Vec::with_capacity(batch_size);
                                        for row_result in rows {
                                            match row_result {
                                                Ok(message) => {
                                                    batch.push(message);
                                                    if batch.len() >= batch_size {
                                                        if sender.send(batch.clone()).is_err() {
                                                            error!("Failed to send message batch for query: {}", query);
                                                            break; // Stop processing for this query
                                                        }
                                                        batch.clear();
                                                    }
                                                }
                                                Err(err) => {
                                                    error!("Failed to process row for query '{}': {:?}", query, err);
                                                }
                                            }
                                        }
                                        // Send any remaining messages in the last batch
                                        if !batch.is_empty() {
                                            if sender.send(batch).is_err() {
                                                error!("Failed to send final message batch for query: {}", query);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        error!("Failed to execute message search query '{}': {:?}", query, err);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Failed to prepare message search statement for query '{}': {:?}", query, err);
                            }
                        }
                    }
                }
            } else if let Err(err) = event {
                return Err(anyhow::Error::new(err).context("failed to receive query event"));
            }
        }
    }
}
