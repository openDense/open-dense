//! a `Store` adapted to Redis database

use super::{Store, StoreValue};
use redis::{Commands, Connection, ErrorKind, RedisResult};
use std::error::Error;

/// contains redis connection and key prefix
pub struct RedisStore {
    connection: std::cell::RefCell<Connection>,
    docname: String,
}

impl RedisStore {
    pub fn new(
        address: &str,
        username: &str,
        password: &str,
        docname: &str,
    ) -> Result<RedisStore, impl Error> {
        let params = "redis://".to_owned() + username + ":" + password + "@" + address;
        match redis::Client::open(params) {
            Ok(client) => Ok(RedisStore {
                connection: client.get_connection()?.into(),
                docname: username.to_owned() + ":" + docname + ":",
            }),
            Err(e) => Err(e),
        }
    }
}

impl Store for RedisStore {
    fn get<T: StoreValue>(&self, key: &str) -> Result<T, impl Error> {
        let res: RedisResult<Vec<u8>> =
            (*self.connection.borrow_mut()).get(self.docname.to_owned() + key);
        match res {
            Ok(bytes) => match T::from_bytes(bytes.as_slice()) {
                Ok(v) => Ok(v),
                Err(_e) => Err((ErrorKind::ParseError, "Cannot convert to StoreValue").into()),
            },
            Err(e) => Err(e),
        }
    }

    fn set(&mut self, key: &str, value: &impl StoreValue) -> Result<(), impl Error> {
        self.connection
            .get_mut()
            .set(self.docname.to_owned() + key, value.to_bytes())
    }
}
