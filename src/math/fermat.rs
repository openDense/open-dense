//! prime number arithmetic

use super::{
    abel::{AbelianMonoid, Modulus, UnitGroup},
    galois::primefield::PrimeField,
    gauss::{
        modular::{MontyForm, MontyParams},
        rand_core::OsRng,
        Concat, Integer, Odd, RandomBits, RandomMod, Split, Uint,
    },
};

// represents an odd prime modulus for PrimeField
/// Example:
/// ```
/// use dense::math::PrimeModulus;
///
/// let modulus = PrimeModulus::<2>::from_random(100);
/// let x = modulus.random_make();
/// let y = modulus.random_make();
/// println!("{:?} + {:?} = {:?}", x.rep(), y.rep(), (x + y).rep());
/// println!("{:?} * {:?} = {:?}", x.rep(), y.rep(), (x * y).rep());
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PrimeModulus<const LIMBS: usize>(pub(crate) MontyParams<LIMBS>);

impl<const LIMBS: usize> From<PrimeModulus<LIMBS>> for Odd<Uint<LIMBS>> {
    fn from(modulus: PrimeModulus<LIMBS>) -> Self {
        *modulus.0.modulus()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TryFromCompositeError;

pub trait PseudoPrimalityTester<const LIMBS: usize> {
    const MAX_ITERTIME: usize = 128;

    fn check(a: PrimeField<LIMBS>) -> bool;

    fn is_prime(n: Uint<LIMBS>) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FermatTester<const LIMBS: usize>;

macro_rules! impl_is_prime {
    ($limbs:literal) => {
        fn is_prime(n: Uint<$limbs>) -> bool
        where
            Uint<$limbs>: Concat<Output = Uint<{ 2 * $limbs }>>,
            Uint<{ 2 * $limbs }>: Split<Output = Uint<$limbs>>,
        {
            if n == Uint::<$limbs>::from_u8(2) {
                return true;
            }
            if n.is_even().into() {
                return false;
            }
            let modulus = Modulus::from(n.to_odd().unwrap());
            for _ in 0..Self::MAX_ITERTIME {
                if !Self::check(modulus.random_make()) {
                    return false;
                }
            }
            true
        }
    };
}

/// implements Fermat tester
macro_rules! impl_fermat_tester {
    ($($limbs:literal)+) => {$(
        impl PseudoPrimalityTester<$limbs> for FermatTester<$limbs> {
            fn check(a: PrimeField<$limbs>) -> bool {
                a.pow(&(a.modulus() - Uint::<$limbs>::ONE)) == a.one()
            }

            impl_is_prime!($limbs);
        }
    )+};
}

impl_fermat_tester!(1 2 3 4 5 6 7 8 10 12 14 16 24 28 32 48 64 128);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MillerRabinTester<const LIMBS: usize>;

/// implements Miller-Rabin tester
macro_rules! impl_miller_rabin_tester {
    ($($limbs:literal)+) => {$(
        impl PseudoPrimalityTester<$limbs> for MillerRabinTester<$limbs> {
            fn check(a: PrimeField<$limbs>) -> bool {
                let t = a.modulus() - Uint::<$limbs>::ONE;
                let h = t.trailing_zeros();
                let p1 = a.one();
                let m1 = a.minus_one();
                let t = t >> h;
                let mut b = a.pow(&t);
                if b == p1 {
                    return true;
                }
                for _ in 0..h {
                    if b == m1 {
                        return true;
                    }
                    if b == p1 {
                        return false;
                    }
                    b *= b;
                }
                false
            }

            impl_is_prime!($limbs);
        }
    )+};
}

impl_miller_rabin_tester!(1 2 3 4 5 6 7 8 10 12 14 16 24 28 32 48 64 128);

/// implements prime modulus
macro_rules! impl_prime_modulus {
    ($($limbs:literal)+) => {$(
        impl PrimeModulus<$limbs> {
            /// constructs a prime modulus from given odd number
            pub fn try_from(modulus: Odd<Uint<$limbs>>) -> Result<Self, TryFromCompositeError> {
                if MillerRabinTester::is_prime(modulus.get()) {
                    Ok(Self(MontyParams::new(modulus)))
                } else {
                    Err(TryFromCompositeError)
                }
            }

            /// constructs a random prime modulus of given bit size
            pub fn from_random(nbits: u32) -> Self {
                loop {
                    let mask = Uint::<$limbs>::ONE | (Uint::<$limbs>::ONE << (nbits - 1));
                    let candidiate =
                        Uint::<$limbs>::try_random_bits(&mut OsRng, nbits).unwrap() | mask;
                    if let Ok(modulus) = Self::try_from(candidiate.to_odd().unwrap()) {
                        return modulus;
                    }
                }
            }

            /// constructs a number in PrimeField of this modulus
            pub fn make(&self, value: &Uint<$limbs>) -> PrimeField<$limbs> {
                UnitGroup(MontyForm::new(value, self.0))
            }

            /// constructs a random number in PrimeField of this modulus
            pub fn random_make(&self) -> PrimeField<$limbs> {
                self.make(&Uint::<$limbs>::random_mod(
                    &mut OsRng,
                    &self.0.modulus().as_nz_ref(),
                ))
            }
        }
    )+};
}

impl_prime_modulus!(1 2 3 4 5 6 7 8 10 12 14 16 24 28 32 48 64 128);
