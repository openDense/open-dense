//! secret sharing algorithms

use crate::math::galois::FiniteField;
use crate::protocol::error::Result;

/// (T, N) - secret sharing server over field F
pub trait SSServer<const T: usize, const N: usize, F: FiniteField> {
    fn prepare(&self) -> Result<()> {
        Ok(())
    }
    fn split(&self, secret: F) -> Result<[F; N]>;
    fn recover(&self, shares: &[Option<F>; N]) -> Result<F>;
}

pub mod functionality;
