use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::Ordering;
use crate::{
    signals::Signals,
    ports::DISCOVERY_PORT,
    delays::{
        WAIT_DELAY,
        CHECK_DELAY
    }
};

fn find_manager(socket: UdpSocket, signals: Signals) -> bool {
    while signals.run.load(Ordering::Relaxed) {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let msg = std::str::from_utf8(&buf[..amt]).unwrap();
                if msg == "MANAGER" {
                    return true;
                }
            }
            Err(_) => {}
        }
    };
    false
}

pub fn initialize(signals: &Signals) {
    let addr = SocketAddr::from(([127, 0, 0, 1], DISCOVERY_PORT));
    let socket = UdpSocket::bind(addr).expect("Failed to bind monitor socket");
    socket.set_nonblocking(true).expect("Failed to set non-blocking mode");
    socket.set_read_timeout(Some(CHECK_DELAY)).expect("Failed to set read timeout");

    while signals.run.load(Ordering::Relaxed) {
        
        if signals.is_manager.load(Ordering::Relaxed) {
            let mut buf = [0; 1024];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    let msg = std::str::from_utf8(&buf[..amt]).unwrap();
                    println!("Received message from {}: {}", src, msg);
                }
                Err(_) => {}
            }
        }
        else {
            
        }
        std::thread::sleep(WAIT_DELAY);
    }
}