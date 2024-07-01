//! binary field GF(2^n) arithmetics

use crate::math::{
    abel::{AbelianMonoid, Inv},
    galois::{irreducible::BinaryIrreducible, FiniteField},
    gauss::{rand_core::OsRng, Integer, RandomBits, Uint, Zero},
};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryField<const EXP: u32, const LIMBS: usize>(Uint<LIMBS>);

impl<const EXP: u32, const LIMBS: usize> BinaryField<EXP, LIMBS> {
    const MASK: Uint<LIMBS> = {
        let mask = Uint::<LIMBS>::ONE.shl(EXP - 1);
        let mask = mask.wrapping_sub(&Uint::<LIMBS>::ONE).bitor(&mask);
        mask
    };

    pub const ZERO: Self = Self(Uint::<LIMBS>::ZERO);
    pub const ONE: Self = Self(Uint::<LIMBS>::ONE);

    pub fn random_new() -> Self {
        Self(Uint::<LIMBS>::random_bits(&mut OsRng, EXP))
    }
}

impl<const EXP: u32, const LIMBS: usize> Neg for BinaryField<EXP, LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn neg(self) -> Self::Output {
        self
    }
}

/// implement binary operators
macro_rules! impl_binary_ops {
    ($($Op:ident :: $op:ident)+) => {$(
        impl<const EXP: u32, const LIMBS: usize> $Op<Self> for BinaryField<EXP, LIMBS> {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn $op(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }

        impl<const EXP: u32, const LIMBS: usize> $Op<&Self> for BinaryField<EXP, LIMBS> {
            type Output = Self;

            #[inline]
            #[track_caller]
            fn $op(self, other: &Self) -> Self::Output {
                $Op::$op(self, *other)
            }
        }
    )+};
}

impl_binary_ops!(Add::add Sub::sub);

/// implement binary assignment operators
macro_rules! impl_binary_assign_ops {
    ($($Op:ident :: $op:ident)+) => {$(
        impl<const EXP: u32, const LIMBS: usize> $Op<Self> for BinaryField<EXP, LIMBS> {
            #[inline]
            #[track_caller]
            fn $op(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }

        impl<const EXP: u32, const LIMBS: usize> $Op<&Self> for BinaryField<EXP, LIMBS> {
            #[inline]
            #[track_caller]
            fn $op(&mut self, other: &Self) {
                $Op::$op(self, *other);
            }
        }
    )+};
}

impl_binary_assign_ops!(AddAssign::add_assign SubAssign::sub_assign);

impl<const EXP: u32, const LIMBS: usize> Mul<Self> for BinaryField<EXP, LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, other: Self) -> Self::Output {
        let irred = BinaryIrreducible::<EXP, LIMBS>::POLY;
        let mut prod = Uint::<LIMBS>::ZERO;
        let mut lhs = self.0;
        let mut rhs = other.0;
        while rhs.is_zero().unwrap_u8() == 0 {
            if rhs.bit_vartime(0) {
                prod ^= lhs;
            }
            let carry = lhs.bit_vartime(EXP - 1);
            lhs <<= 1;
            if carry {
                lhs ^= irred;
            }
            rhs >>= 1;
        }
        Self(prod & Self::MASK)
    }
}

impl<const EXP: u32, const LIMBS: usize> Mul<&Self> for BinaryField<EXP, LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, other: &Self) -> Self::Output {
        self * *other
    }
}

impl<const EXP: u32, const LIMBS: usize> MulAssign<Self> for BinaryField<EXP, LIMBS> {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<const EXP: u32, const LIMBS: usize> MulAssign<&Self> for BinaryField<EXP, LIMBS> {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, other: &Self) {
        *self *= *other;
    }
}

fn divrem<const LIMBS: usize>(lhs: Uint<LIMBS>, rhs: Uint<LIMBS>) -> (Uint<LIMBS>, Uint<LIMBS>) {
    let mut quo = Uint::<LIMBS>::ZERO;
    let mut rem = lhs;
    let mut lbits = rem.bits();
    let rbits = rhs.bits();
    while lbits >= rbits {
        let d = lbits - rbits;
        quo ^= Uint::<LIMBS>::ONE << d;
        rem ^= rhs << d;
        lbits = rem.bits();
    }
    (quo, rem)
}

impl<const EXP: u32, const LIMBS: usize> Inv for BinaryField<EXP, LIMBS> {
    fn inv(&self) -> Self {
        let r = BinaryIrreducible::<EXP, LIMBS>::POLY >> 1;
        let mut r = Self(r ^ (Self::ONE.0 << (EXP - 1)));
        let mut s = *self;
        let (mut q0, mut r0) = divrem(r.0, s.0);
        q0 <<= 1;
        r0 = (r0 << 1) ^ Self::ONE.0;
        if r0.bits() == s.0.bits() {
            q0 ^= Self::ONE.0;
            r0 ^= s.0;
        }
        (s.0, r.0) = (r0, s.0);
        let mut q = Self(q0);
        let mut v = self.one();
        let mut u = q;
        while s != s.zero() {
            (q.0, r.0) = divrem(r.0, s.0);
            (s.0, r.0) = (r.0, s.0);
            v -= q * u;
            (v, u) = (u, v);
        }
        v
    }
}

impl<const EXP: u32, const LIMBS: usize> Div<Self> for BinaryField<EXP, LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn div(self, other: Self) -> Self::Output {
        self * other.inv()
    }
}

impl<const EXP: u32, const LIMBS: usize> Div<&Self> for BinaryField<EXP, LIMBS> {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn div(self, other: &Self) -> Self::Output {
        self * other.inv()
    }
}

impl<const EXP: u32, const LIMBS: usize> DivAssign<Self> for BinaryField<EXP, LIMBS> {
    #[inline]
    #[track_caller]
    fn div_assign(&mut self, other: Self) {
        *self *= other.inv();
    }
}

impl<const EXP: u32, const LIMBS: usize> DivAssign<&Self> for BinaryField<EXP, LIMBS> {
    #[inline]
    #[track_caller]
    fn div_assign(&mut self, other: &Self) {
        *self *= other.inv();
    }
}

impl<const EXP: u32, const LIMBS: usize> AbelianMonoid for BinaryField<EXP, LIMBS> {
    type Uint = Uint<LIMBS>;

    fn one(&self) -> Self {
        Self(Uint::ONE)
    }

    fn pow(&self, exponent: &Self::Uint) -> Self {
        let mut res = *self;
        let mut exp = *exponent;
        if exp.is_zero().unwrap_u8() == 1 {
            return self.one();
        }
        while exp != Uint::ONE {
            res *= res;
            if exp.is_odd().unwrap_u8() == 1 {
                res *= self;
            }
            exp >>= 1;
        }
        res
    }
}

impl<const EXP: u32, const LIMBS: usize> FiniteField for BinaryField<EXP, LIMBS> {
    fn zero(&self) -> Self {
        Self::ZERO
    }

    fn random(&self) -> Self {
        Self(Uint::<LIMBS>::random_bits(&mut OsRng, EXP))
    }

    fn char(&self) -> Self::Uint {
        Uint::<LIMBS>::from_u8(2)
    }
}
