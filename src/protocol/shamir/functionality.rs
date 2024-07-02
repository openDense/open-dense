//! demonstrates the functionality of secret sharing algorithm
//! Panic! NEVER use this protocol because it's unsafe.

use super::SSServer;
use crate::math::galois::FiniteField;
use crate::protocol::error::{Error, MPCErrorKind, Result};

pub struct Server<const T: usize, const N: usize>;

impl<const T: usize, const N: usize, F: FiniteField> SSServer<T, N, F> for Server<T, N> {
    fn split(&self, secret: F) -> Result<[F; N]> {
        Ok([secret; N])
    }

    fn recover(&self, shares: &[Option<F>; N]) -> Result<F> {
        let mut lead = None;
        let mut vote = 0;
        for &share in shares {
            if share == lead {
                vote += 1;
            } else if vote == 0 {
                lead = share;
                vote = 1;
            } else {
                vote -= 1;
            }
        }
        if vote >= T && lead.is_some() {
            Ok(lead.unwrap())
        } else {
            Err(Error::MPCError(MPCErrorKind::InsufficientShares))
        }
    }
}

#[test]
fn test_correctness() {
    use crate::math::galois::binaryfield::BinaryField;
    type GF16 = BinaryField<4, 1>;
    let server = Server::<3, 5>;
    let secret = GF16::random_new();
    let mut shares: [Option<GF16>; 5] = server
        .split(secret)
        .unwrap()
        .into_iter()
        .map(|s| Some(s))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    shares[0] = None;
    shares[1].replace(GF16::random_new());
    assert_eq!(secret, server.recover(&shares).unwrap());
    shares[2].replace(GF16::random_new());
    assert_eq!(
        Error::MPCError(MPCErrorKind::InsufficientShares),
        server.recover(&shares).unwrap_err()
    );
}
