use std::sync::{Arc, Mutex};

use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct State {
    // NOTE: SQLite connections cannot be shared across threads. However, the bot maintains a single connection
    //       that may be accessed from multiple threads, so we need `Arc` + `Mutex` here. This unfortunately means
    //       that only a single command can be executed at any given time, since each query would lock the database
    //       connection. Fortunately, SQLite is fast and the bot is intended for mostly personal use, so this is OK.
    pub conn: Arc<Mutex<Connection>>,
}
