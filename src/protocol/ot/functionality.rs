//! demonstrates the functionality of OT protocol
//! Panic! NEVER use this protocol because it's unsafe.

use super::{Choice, OTReceive, OTSend};
use crate::protocol::error::Result;
use crate::protocol::party::TwoParty;

pub struct Sender(TwoParty);
pub struct Receiver(TwoParty);

impl<const N: usize, const L: usize> OTSend<N, L> for Sender {
    fn send(&self, messages: &[[u8; L]; N]) -> Result<()> {
        let choice = usize::from_le_bytes(self.0.pull()?.try_into().unwrap());
        self.0.push(&messages[choice]).unwrap();
        Ok(())
    }
}

impl<const N: usize, const L: usize> OTReceive<N, L> for Receiver {
    fn receive(&self, choice: &Choice<N>) -> Result<[u8; L]> {
        self.0.push(&choice.to_le_bytes())?;
        Ok(self.0.pull()?.try_into().unwrap())
    }
}

#[test]
fn test_correctness() {
    use crate::protocol::party::TwoParty;
    use std::net::SocketAddr;
    use std::thread;

    let peers = [
        SocketAddr::from(([127, 0, 0, 1], 8080)),
        SocketAddr::from(([127, 0, 0, 1], 8081)),
    ];
    let msgs = [[0u8; 4], [1u8; 4]];
    let mut result = [2u8; 4];
    thread::scope(|scope| {
        scope.spawn(move || {
            let sender = Sender(TwoParty::new(0, &peers).unwrap());
            sender.send(&msgs).unwrap();
        });
        scope.spawn(|| {
            let receiver = Receiver(TwoParty::new(1, &peers).unwrap());
            let choice = Choice::<2>::new(1).unwrap();
            result = receiver.receive(&choice).unwrap();
        });
    });
    assert_ne!(result, [0; 4]);
    assert_eq!(result, [1; 4]);
}
