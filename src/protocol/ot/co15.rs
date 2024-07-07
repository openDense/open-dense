//! implements Simplest OT algorithm
//! The Simplest Protocol for Oblivious Transfer
//! https://eprint.iacr.org/2015/267.pdf

use super::{Choice, OTReceive, OTSend};
use crate::protocol::error::Result;
use crate::protocol::party::TwoParty;
use blake2::{Blake2b512, Digest};
use k256::{
    elliptic_curve::{
        group::GroupEncoding,
        ops::{LinearCombination, MulByGenerator},
        rand_core::OsRng,
        Field,
    },
    ProjectivePoint, Scalar,
};

pub struct Sender(TwoParty);
pub struct Receiver(TwoParty);

impl<const N: usize, const L: usize> OTSend<N, L> for Sender {
    fn send(&self, messages: &[[u8; L]; N]) -> Result<()> {
        // todo: use precomputation to speed up
        // key exchange
        let y = Scalar::random(&mut OsRng);
        let s = ProjectivePoint::mul_by_generator(&y);
        let t = s * y;
        self.0.push(s.to_bytes().as_ref())?;
        let bytes: [u8; 33] = self.0.pull()?.try_into().unwrap();
        let r = ProjectivePoint::from_bytes(&bytes.into()).unwrap();
        // todo: xor message with mask will be a common operation
        let mask = |msg: &[u8], hash: &[u8]| {
            assert!(L <= 64, "L must be less than or equal to 64 to be safe");
            msg.iter()
                .zip(hash.iter())
                .map(|(m, h)| m ^ h)
                .collect::<Vec<_>>()
        };
        // send encrypted messages
        let mut ciphers = Vec::with_capacity(N * L);
        for i in 0..N {
            let key = ProjectivePoint::lincomb(&r, &y, &t, &Scalar::from(i as u128).negate());
            let mut hasher = Blake2b512::new();
            hasher.update(key.to_bytes());
            hasher.update(s.to_bytes());
            hasher.update(r.to_bytes());
            ciphers.append(&mut mask(&messages[i], &hasher.finalize()));
        }
        self.0.push(ciphers.as_slice())?;
        Ok(())
    }
}

impl<const N: usize, const L: usize> OTReceive<N, L> for Receiver {
    fn receive(&self, choice: &Choice<N>) -> Result<[u8; L]> {
        // key exchange
        let bytes: [u8; 33] = self.0.pull()?.try_into().unwrap();
        let s = ProjectivePoint::from_bytes(&bytes.into()).unwrap();
        let x = Scalar::random(&mut OsRng);
        let r = ProjectivePoint::lincomb(
            &s,
            &Scalar::from(choice.0 as u128),
            &ProjectivePoint::GENERATOR,
            &x,
        );
        self.0.push(r.to_bytes().as_ref())?;
        // receive encrypted messages
        let ciphers = self.0.pull()?;
        let ciphers: Vec<[u8; L]> = ciphers
            .chunks(L)
            .map(|bytes| bytes.try_into().unwrap())
            .collect();
        // decrypt the message
        let key = s * x;
        let cipher = ciphers[choice.0];
        let mut hasher = Blake2b512::new();
        hasher.update(key.to_bytes());
        hasher.update(s.to_bytes());
        hasher.update(r.to_bytes());
        let hash = hasher.finalize();
        let mut result = [0u8; L];
        for i in 0..L {
            result[i] = cipher[i] ^ hash[i];
        }
        Ok(result)
    }
}

#[test]
fn test_correctness() {
    use crate::protocol::party::TwoParty;
    use std::net::SocketAddr;
    use std::thread;

    let peers = [
        SocketAddr::from(([127, 0, 0, 1], 8070)),
        SocketAddr::from(([127, 0, 0, 1], 8071)),
    ];

    let msgs = &[[0u8; 4], [1u8; 4], [2u8; 4], [3u8; 4]];
    let index = 2;
    let choice = Choice::<4>::new(index).unwrap();
    let mut result = [4u8; 4];
    thread::scope(|scope| {
        scope.spawn(|| {
            let sender = Sender(TwoParty::new(0, &peers).unwrap());
            sender.send(msgs).unwrap();
        });
        scope.spawn(|| {
            let receiver = Receiver(TwoParty::new(1, &peers).unwrap());
            result = receiver.receive(&choice).unwrap();
        });
    });
    assert_eq!(result, msgs[index]);
}
