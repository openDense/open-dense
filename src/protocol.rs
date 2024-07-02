//! implementations of MPC protocols

// pub mod gentry;
pub mod shamir;
// pub mod yao;

/// defines security parameters in computational and statistical levels
pub struct SecParams(u16, u16);

/// defines adversary behavior
pub enum Adversary {
    SemiHonest,
    Malicious,
}

pub mod error;
pub mod ot;
pub mod party;
