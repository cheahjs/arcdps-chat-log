use std::sync::mpsc;

use anyhow::Context;
use arcdps::extras::{ChatMessageInfo, ChatMessageInfoOwned};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use super::{
    query::{Note, QueriedNote},
    ChatDatabase,
};

pub enum DbInsert {
    ChatMessage(ChatMessageInfoOwned),
    AddNote(NoteToAdd),
    DeleteNote(String),
}

#[derive(Clone)]
pub struct NoteToAdd {
    pub(crate) account_name: String,
    pub(crate) note: String,
    cur_time: i64,
}

impl NoteToAdd {
    pub fn new(account_name: &str, note: &str) -> Self {
        Self {
            account_name: account_name.to_owned(),
            note: note.to_owned(),
            cur_time: chrono::Utc::now().timestamp(),
        }
    }
}

impl ChatDatabase {
    pub fn process_message(&self, message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
        if let Some(insert_channel) = &self.insert_channel {
            insert_channel
                .lock()
                .unwrap()
                .send(DbInsert::ChatMessage(message.to_owned().into()))
                .context("failed to insert message into insert channel")?;
        }
        Ok(())
    }

    pub fn insert_note(&self, note: NoteToAdd) -> Result<(), anyhow::Error> {
        if let Some(insert_channel) = &self.insert_channel {
            insert_channel
                .lock()
                .unwrap()
                .send(DbInsert::AddNote(note.clone()))
                .context("failed to insert note into insert channel")?;
            // update the cache for immediate read-back
            let mut note_cache = self.note_cache.lock().unwrap();
            let new_note = Note {
                account_name: note.account_name.to_owned(),
                note: note.note,
                note_added: note.cur_time,
                note_updated: note.cur_time,
            };
            let existing_note = note_cache.get(&note.account_name);
            match existing_note {
                Some(existing_note) => match existing_note {
                    QueriedNote::Success(existing_note) => {
                        let mut existing_note = existing_note.clone();
                        existing_note.note = new_note.note;
                        existing_note.note_updated = new_note.note_updated;
                        note_cache.insert(note.account_name, QueriedNote::Success(existing_note));
                    }
                    QueriedNote::Error | QueriedNote::NotFound | QueriedNote::Pending => {
                        note_cache.insert(note.account_name, QueriedNote::Success(new_note));
                    }
                },
                None => {
                    note_cache.insert(note.account_name, QueriedNote::Success(new_note));
                }
            }
        }
        Ok(())
    }

    pub fn delete_note(&self, account_name: &str) -> Result<(), anyhow::Error> {
        if let Some(insert_channel) = &self.insert_channel {
            insert_channel
                .lock()
                .unwrap()
                .send(DbInsert::DeleteNote(account_name.to_owned()))
                .context("failed to insert note deletion into insert channel")?;
            // delete from cache as well
            self.note_cache
                .lock()
                .unwrap()
                .insert(account_name.to_owned(), super::query::QueriedNote::NotFound);
        }
        Ok(())
    }

    pub(crate) fn insert_thread(
        game_start: i64,
        pool: Pool<SqliteConnectionManager>,
        recv_chan: mpsc::Receiver<DbInsert>,
    ) -> anyhow::Result<()> {
        let connection = pool.get().context("failed to get database connection")?;
        loop {
            let event = recv_chan.recv();
            if let Ok(insert) = event {
                match insert {
                    DbInsert::ChatMessage(message) => {
                        let mut statement = connection
                            .prepare_cached(
                                "INSERT INTO messages (
                                            channel_id,
                                            channel_type,
                                            subgroup,
                                            is_broadcast,
                                            timestamp,
                                            account_name,
                                            character_name,
                                            text,
                                            game_start
                                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            )
                            .context("failed to prepare message insert statement")?;
                        statement
                            .execute(params![
                                message.channel_id,
                                message.channel_type.to_string(),
                                message.subgroup,
                                message.is_broadcast,
                                message.timestamp,
                                message.account_name,
                                message.character_name,
                                message.text,
                                game_start
                            ])
                            .context("failed to insert message")?;
                    }
                    DbInsert::AddNote(note) => {
                        let mut statement = connection
                            .prepare_cached(
                                "INSERT INTO notes (
                                            account_name,
                                            note_added,
                                            note_updated,
                                            note
                                     ) VALUES (?1, ?2, ?2, ?3)
                                     ON CONFLICT (account_name) DO UPDATE SET note_updated=?2, note=?3",
                            )
                            .context("failed to prepare note insert statement")?;
                        statement
                            .execute(params![
                                note.account_name,
                                note.cur_time.to_string(),
                                note.note
                            ])
                            .context("failed to insert note")?;
                    }
                    DbInsert::DeleteNote(account_name) => {
                        let mut statement = connection
                            .prepare_cached("DELETE FROM notes WHERE account_name=?1")
                            .context("failed to prepare delete note statement")?;
                        statement
                            .execute(params![account_name,])
                            .context("failed to delete note")?;
                    }
                }
            } else if let Err(err) = event {
                return Err(anyhow::Error::new(err).context("failed to receive insert event"));
            }
        }
    }
}
