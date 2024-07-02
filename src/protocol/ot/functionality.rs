//! demonstrates the functionality of OT protocol
//! Panic! NEVER use this protocol because it's unsafe.

use super::{Choice, OTReceive, OTSend};
use crate::protocol::error::Result;
use crate::protocol::party::MultiParty;

pub struct Trust(MultiParty<3>);
pub struct Sender(MultiParty<3>);
pub struct Receiver(MultiParty<3>);

impl Trust {
    pub fn run<const N: usize, const L: usize>(&self) -> Result<()> {
        let message: Vec<u8> = self.0.recv(&1usize)?;
        let choice =
            usize::from_le_bytes(self.0.recv(&2usize).unwrap().as_slice().try_into().unwrap());
        self.0
            .send(&2usize, &message[choice * L..(choice + 1) * L])?;
        Ok(())
    }
}

impl<const N: usize, const L: usize> OTSend<N, L> for Sender {
    fn send(&self, messages: &[[u8; L]; N]) -> Result<()> {
        let message = messages.concat();
        self.0.send(&0usize, message.as_slice())?;
        Ok(())
    }
}

impl<const N: usize, const L: usize> OTReceive<N, L> for Receiver {
    fn receive(&self, choice: &Choice<N>) -> Result<[u8; L]> {
        self.0.send(&0usize, &choice.to_le_bytes())?;
        let result = self.0.recv(&0usize).unwrap();
        let result: [u8; L] = result.try_into().unwrap();
        Ok(result)
    }
}

#[test]
fn test_correctness() {
    use crate::protocol::party::MultiParty;
    use std::net::SocketAddr;
    use std::thread;

    let peers = [
        SocketAddr::from(([127, 0, 0, 1], 8080)),
        SocketAddr::from(([127, 0, 0, 1], 8081)),
        SocketAddr::from(([127, 0, 0, 1], 8082)),
    ];

    let msgs = &[[0u8; 4], [1u8; 4]];
    let index = 1;
    let choice = Choice::<2>::new(index).unwrap();
    let mut result = [4u8; 4];
    thread::scope(|scope| {
        scope.spawn(|| {
            let trust = Trust(MultiParty::<3>::new(0, &peers).unwrap());
            trust.run::<2, 4>().unwrap();
        });
        scope.spawn(|| {
            let sender = Sender(MultiParty::<3>::new(1, &peers).unwrap());
            sender.send(msgs).unwrap();
        });
        scope.spawn(|| {
            let receiver = Receiver(MultiParty::<3>::new(2, &peers).unwrap());
            result = receiver.receive(&choice).unwrap();
        });
    });
    assert_eq!(result, msgs[index]);
}
