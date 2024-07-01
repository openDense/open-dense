//! finite field arithmetic

use super::abel::AbelianGroup;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// provides functionality to finite field
pub trait FiniteField:
    AbelianGroup
    + Neg
    + Add<Self>
    + Sub<Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + for<'a> Add<&'a Self>
    + for<'a> Sub<&'a Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
{
    fn zero(&self) -> Self;
    fn random(&self) -> Self;
    fn char(&self) -> Self::Uint;
}

pub mod binaryfield;
mod irreducible;
pub mod primefield;
