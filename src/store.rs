//! manages the interaction with datastore.
//! To use, choose a specialized Store struct and `open` with appropriate store config.
//! Then use `get` and `set` to operate with the datastore.
//! Finally `close` the store.

use std::{error::Error, vec};

/// provides an access to the datastore.
/// # Example
/// ```
/// use dense::store::{Store, disk::DiskStore};
/// let mut store = DiskStore::new("data/Alice", "test").unwrap();
/// let write = vec![1, 2, 3, 4, 5];
/// store.set("test_data", &write).unwrap();
/// let read = store.get::<Vec<i32>>("test_data").unwrap();
/// println!("{:?}", read);
/// ```
pub trait Store {
    fn get<T: StoreValue>(&self, key: &str) -> Result<T, impl Error>;
    fn set(&mut self, key: &str, value: &impl StoreValue) -> Result<(), impl Error>;
}

/// provides a way to convert a value to bytes and back, so that it can be stored in the datastore.
pub trait StoreValue: Clone {
    fn to_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized;
}

/// error returned when parsing a value from bytes fails.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParseError;

macro_rules! impl_store_value_for_numerical_type {
    ($($t:ty)+) => {
        $(impl StoreValue for $t {
            fn to_bytes(self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> where Self:Sized
            {
                match bytes.try_into() {
                    Ok(bytes_array) => Ok(Self::from_le_bytes(bytes_array)),
                    Err(_e) => Err(ParseError),
                }
            }
        })+
    };
}

impl_store_value_for_numerical_type!(
    u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize
);

impl StoreValue for String {
    fn to_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        match Self::from_utf8(bytes.to_vec()) {
            Ok(value) => Ok(value),
            Err(_e) => Err(ParseError),
        }
    }
}

///! `Vec<T>` where `T` has no static size may cause parse error
impl<T: StoreValue> StoreValue for Vec<T> {
    fn to_bytes(self) -> Vec<u8> {
        self.into_iter().fold(vec![], |mut acc, item| {
            acc.extend(item.to_bytes());
            acc
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let elem_size = std::mem::size_of::<T>();
        let vec_size = bytes.len() / elem_size;
        let mut res = Vec::<T>::with_capacity(vec_size);
        for chunk in bytes.chunks(elem_size) {
            match T::from_bytes(chunk) {
                Ok(v) => res.push(v),
                Err(e) => return Err(e),
            };
        }
        Ok(res)
    }
}

pub mod disk;

#[cfg(feature = "duckdb")]
pub mod duckdb;

#[cfg(feature = "redis")]
pub mod redis;
