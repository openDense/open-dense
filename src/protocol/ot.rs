//! oblivious transfer protocols

use std::ops::Deref;

use super::error::Result;

/// choice in OT with range guaranteed to be in [0, N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Choice<const N: usize>(usize);

impl<const N: usize> Choice<N> {
    pub const fn new(choice: usize) -> Option<Self> {
        if choice < N {
            Some(Self(choice))
        } else {
            None
        }
    }
}

impl<const N: usize> Deref for Choice<N> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 1 out of N oblivious transfer send of length L
pub trait OTSend<const N: usize, const L: usize> {
    fn send(&self, messages: &[[u8; L]; N]) -> Result<()>;
}

/// 1 out of N oblivious transfer receive of length L
pub trait OTReceive<const N: usize, const L: usize> {
    fn receive(&self, choice: &Choice<N>) -> Result<[u8; L]>;
}

pub mod functionality;
