//! provides party utilities

use super::error::Result;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Abstract party trait
pub struct MultiParty<const N: usize> {
    /// party id (assume server or sender id is 0)
    pub id: usize,
    /// used to send and receive messages between parties
    session: RefCell<Session<N>>,
}

impl<const N: usize> MultiParty<N> {
    /// create a new party
    pub fn new(id: usize, peers: &[SocketAddr; N]) -> Result<Self> {
        Ok(Self {
            id,
            session: RefCell::new(Session::<N>::new(id, peers)?),
        })
    }

    /// send message to a party
    pub fn send(&self, id: &usize, msg: &[u8]) -> Result<()> {
        self.session.borrow_mut().send(*id, msg)
    }

    /// receive message from a party
    pub fn recv(&self, id: &usize) -> Result<Vec<u8>> {
        self.session.borrow_mut().recv(*id)
    }

    /// broadcast message to all parties
    pub fn broadcast(&self, msg: &[u8]) -> Result<()> {
        self.session.borrow_mut().broadcast(msg)
    }

    /// send message to the server (assume id is 0)
    pub fn upload(&self, msg: &[u8]) -> Result<()> {
        self.session.borrow_mut().send(0, msg)
    }

    /// receive message from the server (assume id is 0)
    pub fn download(&self) -> Result<Vec<u8>> {
        self.session.borrow_mut().recv(0)
    }
}

pub type TwoParty = MultiParty<2>;

impl TwoParty {
    /// send message to the other party
    pub fn push(&self, msg: &[u8]) -> Result<()> {
        self.session.borrow_mut().send(1 ^ self.id, msg)
    }

    /// receive message from the other party
    pub fn pull(&self) -> Result<Vec<u8>> {
        self.session.borrow_mut().recv(1 ^ self.id)
    }
}

/// A session is a list of sockets between the current peer to others, plus a listener for incoming connections.
struct Session<const N: usize> {
    sockets: [Option<TcpStream>; N],
}

impl<const N: usize> Session<N> {
    /// create a Session of N peers.
    /// * `id` - the id of the current peer. (0, 1, 2, ..., N-1)
    /// * `peers` - the addresses of the peers. Note that the address of peer `id` is in `peers[id]`.
    pub fn new(id: usize, peers: &[SocketAddr; N]) -> Result<Self> {
        // todo: check the validity of the peers
        let mut sockets = Vec::with_capacity(N);
        for _ in 0..N {
            sockets.push(None);
        }
        let sockets = Arc::new(Mutex::new(sockets));
        thread::scope(|scope| {
            let my_id = id;
            let mut threads = vec![];
            // listen to peers of lower ids
            {
                let sockets = sockets.clone();
                threads.push(scope.spawn(move || {
                    let mut slots = my_id;
                    let listener = TcpListener::bind(peers[my_id]).unwrap();
                    while slots > 0 {
                        let (mut socket, _) = listener.accept().unwrap();
                        // hack: authentication, now simply by admitting whom they claimed to be (id)
                        let mut buf = [0u8; 8];
                        socket.read_exact(&mut buf).unwrap();
                        let id = usize::from_le_bytes(buf);
                        // check id is in the peer list but not in the socket list
                        let success = id < my_id;
                        socket
                            .write_all((success as usize).to_le_bytes().as_ref())
                            .unwrap();
                        if success {
                            sockets.lock().unwrap()[id] = Some(socket);
                            slots -= 1;
                        }
                    }
                }));
            }

            // connect to peers of higher ids
            for id in my_id + 1..N {
                let sockets = Arc::clone(&sockets);
                threads.push(scope.spawn(move || {
                    let mut socket: TcpStream;
                    if let Ok(stream) = TcpStream::connect(peers[id]) {
                        socket = stream;
                    } else {
                        // hack: retry if connection fails
                        thread::sleep(Duration::from_millis(10));
                        socket = TcpStream::connect(peers[id]).unwrap();
                    }
                    // hack: authentication, now simply by sending who I am (my_id)
                    let mut buf = [0u8; 8];
                    socket.write_all(my_id.to_le_bytes().as_ref()).unwrap();
                    socket.read_exact(&mut buf).unwrap();
                    if usize::from_le_bytes(buf) != 0 {
                        sockets.lock().unwrap()[id] = Some(socket);
                    }
                }));
            }
            // todo: error propagation
            for thread in threads {
                thread.join().unwrap();
            }
        });
        Ok(Self {
            sockets: Arc::into_inner(sockets)
                .unwrap()
                .into_inner()
                .unwrap()
                .try_into()
                .unwrap(),
        })
    }

    /// send `data` to peer `id`
    pub fn send(&mut self, id: usize, data: &[u8]) -> Result<()> {
        self.sockets[id]
            .as_mut()
            .map_or(Ok(()), |socket| Ok(socket.write_all(data)?))
    }

    /// receive from peer `id`
    pub fn recv(&mut self, id: usize) -> Result<Vec<u8>> {
        // todo: handle large message, possibly use BufReader
        let mut buf = vec![0; 1024];
        let len = self.sockets[id]
            .as_mut()
            .map_or(Ok(0), |socket| socket.read(&mut buf));
        buf.truncate(len?);
        Ok(buf)
    }

    /// send `data` to all peers
    pub fn broadcast(&mut self, data: &[u8]) -> Result<()> {
        thread::scope(|scope| {
            self.sockets.iter_mut().for_each(|socket| {
                scope.spawn(|| socket.as_mut().map_or(Ok(()), |sock| sock.write_all(data)));
            });
        });
        Ok(())
    }
}
