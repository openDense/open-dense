//! prime field is a field of prime order

use crate::math::{
    abel::{Inv, UnitGroup},
    fermat::PrimeModulus,
    galois::FiniteField,
    gauss::modular::MontyForm,
};
use std::ops::{Add, AddAssign, Div, DivAssign, Neg, Sub, SubAssign};

/// represents a number in prime field
/// PrimeField is essentially an UnitGroup in disguise
pub type PrimeField<const LIMBS: usize> = UnitGroup<LIMBS>;

/// derives negation from MontyForm
impl<const LIMBS: usize> Neg for PrimeField<LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

/// derives binary operators from MontyForm
macro_rules! derive_binary_ops {
    ($($Op:ident :: $op:ident)+) => {$(
        impl<const LIMBS: usize> $Op<Self> for PrimeField<LIMBS> {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn $op(self, other: Self) -> Self::Output {
                Self(self.0.$op(other.0))
            }
        }

        impl<const LIMBS: usize> $Op<&Self> for PrimeField<LIMBS> {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn $op(self, other: &Self) -> Self::Output {
                $Op::$op(self, *other)
            }
        }
    )+};
}

derive_binary_ops!(Add::add Sub::sub);

/// derives binary assignment operators from MontyForm
macro_rules! derive_binary_assign_ops {
    ($($Op:ident :: $op:ident)+) => {$(
        impl<const LIMBS: usize> $Op<Self> for PrimeField<LIMBS> {
            #[inline]
            #[track_caller]
            fn $op(&mut self, other: Self) {
                self.0.$op(other.0);
            }
        }

        impl<const LIMBS: usize> $Op<&Self> for PrimeField<LIMBS> {
            #[inline]
            #[track_caller]
            fn $op(&mut self, other: &Self) {
                $Op::$op(self, *other);
            }
        }
    )+};
}

derive_binary_assign_ops!(AddAssign::add_assign SubAssign::sub_assign);

/// implements trait methods for PrimeField
macro_rules! impl_trait_fns {
    ($($limbs: literal)+) => {$(
        impl FiniteField for PrimeField<$limbs> {
            fn zero(&self) -> Self {
                Self(MontyForm::zero(*self.0.params()))
            }

            fn random(&self) -> Self {
                PrimeModulus(*self.0.params()).random_make()
            }

            fn char(&self) -> Self::Uint {
                self.0.params().modulus().get()
            }
        }

        crate::math::abel::impl_div_as_mulinv!(PrimeField<$limbs>);
    )+};
}

impl_trait_fns!(1 2 3 4 5 6 7 8 10 12 14 16 24 28 32 48 64 128);
