use std::path::PathBuf;

use anyhow::Context;
use arcdps::extras::message::ChatMessageInfo;
use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};

use crate::SETTINGS;

const DEFAULT_LOG_PATH: &str = "arcdps_chat_log.db";

pub struct ChatDatabase {
    pub connection: Connection,
}

impl ChatDatabase {
    pub fn try_new() -> Result<Self, anyhow::Error> {
        let migrations = Migrations::new(vec![M::up(include_str!(
            "../migrations/2022-08-07-create-messages.sql"
        ))]);

        let mut connection = Connection::open(&Self::log_path())
            .context("failed to establish database connection")?;

        migrations
            .to_latest(&mut connection)
            .context("failed to migrate database")?;

        connection
            .pragma_update(None, "journal_mode", &"WAL")
            .context("failed to set journal mode")?;

        Ok(Self { connection })
    }

    pub fn process_message(&mut self, message: &ChatMessageInfo) -> Result<(), anyhow::Error> {
        self.connection
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
        Ok(())
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
}
