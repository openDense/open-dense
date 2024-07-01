//! a `Store` adapted to local filesystem

use super::{Store, StoreValue};
use serde_json::json;
use std::error::Error;
use std::io::{Error as IOError, ErrorKind, Result as IOResult};
use std::path::PathBuf;

/// contains path and json to the document
/// # Example
/// ```
/// use dense::store::disk::DiskStore;
/// let mut store = DiskStore::new("data/Alice", "test");
/// ```
pub struct DiskStore {
    path: PathBuf,
    store: serde_json::Value,
    modified: bool,
}

impl DiskStore {
    pub fn new(path: &str, docname: &str) -> IOResult<DiskStore> {
        let mut path = PathBuf::from(path);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        path.push(docname);
        let store: serde_json::Value = if path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&path)?)?
        } else {
            json!({})
        };
        Ok(DiskStore {
            path,
            store,
            modified: false,
        })
    }
}

impl Drop for DiskStore {
    fn drop(&mut self) {
        if self.modified {
            serde_json::to_writer(std::fs::File::create(&self.path).unwrap(), &self.store).unwrap();
        }
    }
}

impl Store for DiskStore {
    fn get<T: StoreValue>(&self, key: &str) -> Result<T, impl Error> {
        if let Some(value) = self.store.get(key) {
            match T::from_bytes(serde_json::from_value::<Vec<u8>>(value.to_owned())?.as_slice()) {
                Ok(value) => Result::Ok(value),
                Err(_e) => Result::Err(IOError::new(
                    ErrorKind::InvalidData,
                    "Cannot convert to StoreValue",
                )),
            }
        } else {
            Result::Err(IOError::new(ErrorKind::NotFound, "key not found"))
        }
    }

    fn set(&mut self, key: &str, value: &impl StoreValue) -> Result<(), impl Error> {
        self.store[key] = serde_json::to_value(value.clone().to_bytes())?;
        self.modified = true;
        IOResult::Ok(())
    }
}
