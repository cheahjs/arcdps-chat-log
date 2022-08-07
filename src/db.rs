use std::{
    path::PathBuf,
    sync::mpsc::{self, Sender},
    thread::Builder,
};

use anyhow::Context;
use arcdps::extras::message::{ChatMessageInfo, ChatMessageInfoOwned};
use log::error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use rusqlite_migration::{Migrations, M};

use crate::SETTINGS;

const DEFAULT_LOG_PATH: &str = "arcdps_chat_log.db";

pub struct ChatDatabase {
    pub connection_pool: Option<Pool<SqliteConnectionManager>>,
    pub insert_channel: Option<Sender<ChatMessageInfoOwned>>,
}

impl ChatDatabase {
    pub fn try_new() -> Result<Self, anyhow::Error> {
        let migrations = Migrations::new(vec![M::up(include_str!(
            "../migrations/2022-08-07-create-messages.sql"
        ))]);

        let manager = SqliteConnectionManager::file(&Self::log_path());
        let pool = Pool::new(manager).context("failed to create pool")?;
        let mut connection = pool.get().context("failed to get database connection")?;

        migrations
            .to_latest(&mut connection)
            .context("failed to migrate database")?;

        connection
            .pragma_update(None, "journal_mode", &"WAL")
            .context("failed to set journal mode")?;

        let (insert_send, insert_recv) = mpsc::channel::<ChatMessageInfoOwned>();
        let clone_pool = pool.clone();
        let _insert_thread = Builder::new()
            .name("chat_insert".to_owned())
            .spawn(move || match Self::insert_thread(clone_pool, insert_recv) {
                Ok(_) => {}
                Err(err) => {
                    error!("insert thread failed: {}", err);
                }
            });

        Ok(Self {
            connection_pool: Some(pool),
            insert_channel: Some(insert_send),
        })
    }

    pub fn process_message(&mut self, message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
        if let Some(insert_channel) = &self.insert_channel {
            insert_channel
                .send(message.to_owned().into())
                .context("failed to insert message into message channel")?;
        }
        Ok(())
    }

    pub fn release(&mut self) {
        {
            // take channel out to drop it out of scope
            // this should cause the recv channel to close and the pool
            let _ = self.insert_channel.take();
            // take pool out ot drop it out of scope
            // this should close all connections
            let _ = self.connection_pool.take();
        }
    }

    fn log_path() -> String {
        SETTINGS
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .load_data("log_path")
            .unwrap_or_else(|| Self::default_log_path().to_str().unwrap().to_string())
    }

    fn default_log_path() -> PathBuf {
        arcdps::exports::config_path()
            .map(|mut path| {
                if !path.is_dir() {
                    path.pop();
                }
                path.push(DEFAULT_LOG_PATH);
                path
            })
            .unwrap()
    }

    fn insert_thread(
        pool: Pool<SqliteConnectionManager>,
        recv_chan: mpsc::Receiver<ChatMessageInfoOwned>,
    ) -> anyhow::Result<()> {
        let connection = pool.get().context("failed to get database connection")?;
        loop {
            let event = recv_chan.recv();
            if let Ok(message) = event {
                connection
                    .execute(
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
                        params![
                            message.channel_id,
                            message.channel_type.to_string(),
                            message.subgroup,
                            message.is_broadcast,
                            message.timestamp,
                            message.account_name,
                            message.character_name,
                            message.text,
                            crate::GAME_START
                                .get()
                                .context("could not get game start")?
                        ],
                    )
                    .context("failed to insert message")?;
            } else if let Err(err) = event {
                return Err(anyhow::Error::new(err).context("failed to receive insert event"));
            }
        }
    }
}
