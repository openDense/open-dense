//! a `Store` adapted to Redis database

use super::{Store, StoreValue};
use duckdb::{params, Connection, Error as DuckdbError};
use std::error::Error;

/// contains a connection to a duckdb database
pub struct DuckdbStore {
    connection: std::cell::RefCell<Connection>,
    docname: String,
}

impl DuckdbStore {
    /// open a DuckdbStore at the given path
    /// if `str` is empty, then create one in memory
    pub fn new(path: &str, docname: &str) -> Result<DuckdbStore, impl Error> {
        let connection = if path.is_empty() {
            let conn = Connection::open_in_memory();
            if conn.is_ok() {
                conn.as_ref().unwrap().execute(
                    format!(
                        "CREATE TABLE {} (key TEXT PRIMARY KEY, value BLOB);",
                        docname
                    )
                    .as_str(),
                    params![],
                )?;
            }
            conn
        } else {
            Connection::open(path)
        };
        match connection {
            Ok(conn) => Ok(DuckdbStore {
                connection: conn.into(),
                docname: docname.to_string(),
            }),
            Err(e) => Err(e),
        }
    }
}

impl Store for DuckdbStore {
    fn get<T: StoreValue>(&self, key: &str) -> Result<T, impl Error> {
        let bytes = self.connection.borrow_mut().query_row(
            format!("SELECT VALUE FROM {} WHERE KEY = ?;", self.docname).as_str(),
            params![key],
            |row| row.get::<_, Vec<u8>>(0),
        )?;
        match T::from_bytes(bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_e) => Err(DuckdbError::InvalidParameterName(
                "Cannot convert to StoreValue".into(),
            )),
        }
    }

    fn set(&mut self, key: &str, value: &impl StoreValue) -> Result<(), impl Error> {
        self.connection.get_mut().execute(
            format!("INSERT INTO {} (KEY, VALUE) VALUES (?, ?) ON CONFLICT (KEY) DO UPDATE SET VALUE = EXCLUDED.VALUE;", self.docname).as_str(),
            params![key, value.clone().to_bytes()],
        ).and_then(|_|Ok(()))
    }
}
