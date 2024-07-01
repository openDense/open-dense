//! provides basic math functions

pub mod abel;
pub mod fermat;
pub mod galois;
pub mod gauss;

pub use abel::{AbelianGroup, AbelianMonoid, Inv, Modulus, UnitGroup};
pub use fermat::PrimeModulus;
pub use galois::{binaryfield::BinaryField, primefield::PrimeField, FiniteField};
pub use gauss::{prelude::*, Uint};
