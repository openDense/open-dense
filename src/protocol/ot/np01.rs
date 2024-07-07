//! implements Naor-Pinkas OT algorithm
//! Efficient Oblivious Transfer Protocols
//! https://dl.acm.org/doi/pdf/10.5555/365411.365502

use super::{Choice, OTReceive, OTSend};
use crate::protocol::error::Result;
use crate::protocol::party::TwoParty;
use blake2::{Blake2b512, Digest};
use k256::{
    ecdh::EphemeralSecret,
    elliptic_curve::{rand_core::OsRng, Group},
    ProjectivePoint, PublicKey,
};

pub struct Sender(TwoParty);
pub struct Receiver(TwoParty);

impl<const N: usize, const L: usize> OTSend<N, L> for Sender {
    fn send(&self, messages: &[[u8; L]; N]) -> Result<()> {
        // send masks
        let sums: Vec<PublicKey> = (1..N)
            .map(|_| ProjectivePoint::random(OsRng).try_into().unwrap())
            .collect();
        let msg = sums.iter().fold(vec![], |mut acc, sum| {
            acc.extend_from_slice(sum.to_sec1_bytes().as_ref());
            acc
        });
        self.0.push(msg.as_slice())?;
        // key exchange
        let pk = self.0.pull()?;
        let pk = PublicKey::from_sec1_bytes(pk.as_slice()).unwrap();
        let sk = EphemeralSecret::random(&mut OsRng);
        self.0.push(sk.public_key().to_sec1_bytes().as_ref())?;
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
            let pk = if i == 0 {
                pk.to_projective()
            } else {
                sums[i - 1].to_projective() - pk.to_projective()
            };
            let key = sk.diffie_hellman(&pk.try_into().unwrap());
            let mut hasher = Blake2b512::new();
            hasher.update(key.raw_secret_bytes());
            hasher.update(i.to_le_bytes().as_ref());
            ciphers.append(&mut mask(&messages[i], &hasher.finalize()));
        }
        self.0.push(ciphers.as_slice())?;
        Ok(())
    }
}

impl<const N: usize, const L: usize> OTReceive<N, L> for Receiver {
    fn receive(&self, choice: &Choice<N>) -> Result<[u8; L]> {
        // receive masks
        let sums = self.0.pull()?;
        let sums = sums
            .chunks(sums.len() / (N - 1))
            .map(|bytes| PublicKey::from_sec1_bytes(bytes).unwrap())
            .collect::<Vec<_>>();
        // key exchange
        let sk = EphemeralSecret::random(&mut OsRng);
        let mut pk = sk.public_key();
        if choice.0 > 0 {
            pk = (sums[choice.0 - 1].to_projective() - pk.to_projective())
                .try_into()
                .unwrap();
        }
        self.0.push(pk.to_sec1_bytes().as_ref())?;
        let pk = self.0.pull()?;
        let pk = PublicKey::from_sec1_bytes(pk.as_slice()).unwrap();
        let key = sk.diffie_hellman(&pk);
        // receive encrypted messages
        let ciphers = self.0.pull()?;
        let ciphers: Vec<[u8; L]> = ciphers
            .chunks(L)
            .map(|bytes| bytes.try_into().unwrap())
            .collect();
        // decrypt the message
        let cipher = ciphers[choice.0];
        let mut hasher = Blake2b512::new();
        hasher.update(key.raw_secret_bytes());
        hasher.update(choice.0.to_le_bytes().as_ref());
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
        SocketAddr::from(([127, 0, 0, 1], 8090)),
        SocketAddr::from(([127, 0, 0, 1], 8091)),
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
