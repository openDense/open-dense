//! commutative ring arithmetic

use super::gauss::{
    modular::{MontyForm, MontyParams},
    rand_core::OsRng,
    Concat, Odd, Random, RandomMod, Split, Uint,
};
use std::ops::{Mul, MulAssign};

/// supports multiplicative inverse operation
pub trait Inv {
    fn inv(&self) -> Self;
}

/// provides functionality to Abelian multiplication monoid
pub trait AbelianMonoid:
    'static
    + Clone
    + Copy
    + Eq
    + Send
    + Sized
    + Sync
    + Mul<Self>
    + MulAssign<Self>
    + for<'a> Mul<&'a Self>
    + for<'a> MulAssign<&'a Self>
{
    type Uint;

    fn one(&self) -> Self;
    fn pow(&self, exponent: &Self::Uint) -> Self;
}

/// provides functionality to Abelian multiplication group
pub trait AbelianGroup: AbelianMonoid + Inv {}

impl<T: AbelianMonoid + Inv> AbelianGroup for T {}

/// represents an odd modulus for UnitGroup
/// Example:
/// ```
/// use dense::math::Modulus;
///
/// let modulus = Modulus::<2>::from_random();
/// let x = modulus.random_make();
/// let y = modulus.random_make();
/// println!("{:?} * {:?} = {:?}", x.rep(), y.rep(), (x * y).rep());
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Modulus<const LIMBS: usize>(MontyParams<LIMBS>);

impl<const LIMBS: usize> From<Modulus<LIMBS>> for Odd<Uint<LIMBS>> {
    fn from(modulus: Modulus<LIMBS>) -> Self
    {
        *modulus.0.modulus()
    }
}

/// implements methods of Modulus
impl<const LIMBS: usize> Modulus<LIMBS> {
    /// constructs a modulus
    pub fn from<const WIDE_LIMBS: usize>(modulus: Odd<Uint<LIMBS>>) -> Self
    where
        Uint<LIMBS>: Concat<Output = Uint<WIDE_LIMBS>>,
        Uint<WIDE_LIMBS>: Split<Output = Uint<LIMBS>>,
    {
        Self(MontyParams::new(modulus))
    }

    /// constructs a random modulus
    pub fn from_random<const WIDE_LIMBS: usize>() -> Self
    where
        Uint<LIMBS>: Concat<Output = Uint<WIDE_LIMBS>>,
        Uint<WIDE_LIMBS>: Split<Output = Uint<LIMBS>>,
    {
        Self(MontyParams::new(Odd::<Uint<LIMBS>>::random(&mut OsRng)))
    }
}

/// provides methods to construct UnitGroup
macro_rules! impl_make_new_unitgroup_elem {
    ($($limbs:literal)+) => {$(
        impl Modulus<$limbs> {
            /// constructs a number in UnitGroup of this modulus
            pub fn make(&self, value: &Uint<$limbs>) -> Option<UnitGroup<$limbs>> {
                if self.0.modulus().gcd(value).unwrap() == Uint::<$limbs>::ONE {
                    Some(UnitGroup(MontyForm::new(value, self.0)))
                } else {
                    None
                }
            }

            /// constructs a random number in UnitGroup of this modulus
            pub fn random_make(&self) -> UnitGroup<$limbs> {
                loop {
                    if let Some(val) = self.make(&Uint::<$limbs>::random_mod(
                        &mut OsRng,
                        &self.0.modulus().as_nz_ref(),
                    )) {
                        return val;
                    }
                }
            }
        }
    )+};
}

impl_make_new_unitgroup_elem!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 20 24 28 32 48 56 64 66 68 96 128 256 512);

/// represents multiplicative unit subgroup Z*(n) of Z(n)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UnitGroup<const LIMBS: usize>(pub(crate) MontyForm<LIMBS>);

impl<const LIMBS: usize> From<MontyForm<LIMBS>> for UnitGroup<LIMBS> {
    fn from(value: MontyForm<LIMBS>) -> Self {
        Self(value)
    }
}

/// implements methods of UnitGroup
impl<const LIMBS: usize> UnitGroup<LIMBS> {
    /// get the modulus of this number
    pub fn modulus(&self) -> Uint<LIMBS> {
        self.0.params().modulus().get()
    }

    /// get the integer representation of this number
    pub fn rep(&self) -> Uint<LIMBS> {
        self.0.retrieve()
    }
}

impl<const LIMBS: usize> Mul<Self> for UnitGroup<LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, other: Self) -> Self::Output {
        Self(self.0 * other.0)
    }
}

impl<const LIMBS: usize> Mul<&Self> for UnitGroup<LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, other: &Self) -> Self::Output {
        self * *other
    }
}

impl<const LIMBS: usize> MulAssign<Self> for UnitGroup<LIMBS> {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, other: Self) {
        self.0 *= other.0;
    }
}

impl<const LIMBS: usize> MulAssign<&Self> for UnitGroup<LIMBS> {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, other: &Self) {
        *self *= *other;
    }
}

impl<const LIMBS: usize> AbelianMonoid for UnitGroup<LIMBS> {
    type Uint = Uint<LIMBS>;

    fn one(&self) -> Self {
        MontyForm::one(*self.0.params()).into()
    }

    fn pow(&self, exponent: &Self::Uint) -> Self {
        self.0.pow(exponent).into()
    }
}

/// implements inv trait for UnitGroup
macro_rules! impl_inv_trait {
    ($($limbs: literal)+) => {$(
        impl Inv for UnitGroup<$limbs> {
            fn inv(&self) -> Self {
                self.0.inv().unwrap().into()
            }
        }

        impl UnitGroup<$limbs> {
            /// get minus one
            pub fn minus_one(&self) -> Self {
                Modulus(*self.0.params()).make(&(self.modulus() - Uint::<$limbs>::ONE)).unwrap()
            }
        }
    )+};
}

impl_inv_trait!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 20 24 28 32 48 56 64 66 68 96 128 256 512);

macro_rules! impl_div_as_mulinv {
    ($t:ty) => {
        impl Div<$t> for $t {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn div(self, other: Self) -> Self::Output {
                self * other.inv()
            }
        }

        impl<'a> Div<&'a $t> for $t {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn div(self, other: &Self) -> Self::Output {
                self * other.inv()
            }
        }

        impl DivAssign<$t> for $t {
            #[inline]
            #[track_caller]
            fn div_assign(&mut self, other: Self) {
                *self *= other.inv();
            }
        }

        impl<'a> DivAssign<&'a $t> for $t {
            #[inline]
            #[track_caller]
            fn div_assign(&mut self, other: &Self) {
                *self *= other.inv();
            }
        }
    };
}

pub(crate) use impl_div_as_mulinv;
