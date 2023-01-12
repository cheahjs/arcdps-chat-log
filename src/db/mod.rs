pub mod insert;
pub mod query;

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::Builder,
};

use anyhow::Context;
use log::error;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite_migration::{Migrations, M};

use self::{
    insert::DbInsert,
    query::{DbQuery, QueriedNote},
};

pub struct ChatDatabase {
    pub log_path: String,
    pub connection_pool: Option<Pool<SqliteConnectionManager>>,
    pub insert_channel: Option<Mutex<Sender<DbInsert>>>,
    pub query_channel: Option<Mutex<Sender<DbQuery>>>,
    pub note_cache: Arc<Mutex<HashMap<String, QueriedNote>>>,
}

impl ChatDatabase {
    pub fn try_new(log_path: &str, game_start: i64) -> anyhow::Result<Self> {
        let migrations = Migrations::new(vec![
            M::up(include_str!(
                "../../migrations/2022-08-07-create-messages.sql"
            )),
            M::up(include_str!(
                "../../migrations/2022-08-07-messages-timestamp-index.sql"
            )),
            M::up(include_str!("../../migrations/2023-01-05-create-notes.sql")),
        ]);

        let manager = SqliteConnectionManager::file(log_path);
        let pool = Pool::new(manager).context("failed to create pool")?;
        let mut connection = pool.get().context("failed to get database connection")?;

        migrations
            .to_latest(&mut connection)
            .context("failed to migrate database")?;

        connection
            .pragma_update(None, "journal_mode", "WAL")
            .context("failed to set journal mode")?;

        let (insert_send, insert_recv) = mpsc::channel::<DbInsert>();
        let clone_pool = pool.clone();
        let _insert_thread = Builder::new()
            .name("chat_insert".to_owned())
            .spawn(
                move || match Self::insert_thread(game_start, clone_pool, insert_recv) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("insert thread failed: {:#}", err);
                    }
                },
            );

        let (query_send, query_recv) = mpsc::channel::<DbQuery>();
        let clone_pool = pool.clone();
        let note_cache = Arc::new(Mutex::new(HashMap::new()));
        let clone_note_cache = note_cache.clone();
        let _query_thread = Builder::new().name("chat_query".to_owned()).spawn(move || {
            match Self::query_thread(clone_pool, query_recv, clone_note_cache) {
                Ok(_) => {}
                Err(err) => {
                    error!("query thread failed: {:#}", err);
                }
            }
        });

        Ok(Self {
            log_path: log_path.to_string(),
            connection_pool: Some(pool),
            insert_channel: Some(Mutex::new(insert_send)),
            query_channel: Some(Mutex::new(query_send)),
            note_cache,
            // game_start,
        })
    }

    pub fn release(&mut self) {
        {
            // take channel out to drop it out of scope
            // this should cause the recv channel to close and the pool
            let _ = self.insert_channel.take();
            let _ = self.query_channel.take();
            // take pool out to drop it out of scope
            // this should close all connections
            let _ = self.connection_pool.take();
        }
    }
}
