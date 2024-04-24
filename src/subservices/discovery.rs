use crate::pcinfo;
use crate::pcinfo::PCInfo;
use crate::{
    delays::{CHECK_DELAY, WAIT_DELAY},
    ports::DISCOVERY_PORT,
    signals::Signals,
};
use mac_address::MacAddress;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{atomic::Ordering, mpsc::Sender};

use self::find::find_manager;
use self::listen::listen_for_clients;

mod find {
    use super::*;

    pub fn find_manager(socket: &UdpSocket, signals: &Signals, new_pc_tx: &Sender<PCInfo>) -> bool {
        while signals.run.load(Ordering::Relaxed) {
            let mut buf = [0; 1024];
            match socket.recv_from(&mut buf) {
                Ok((_amt, src)) => {
                    // let msg = std::str::from_utf8(&buf[..amt]).unwrap();
                    // if msg == "MANAGER" {
                    //     return true;
                    // }
                    let hostname = "pc1".to_string();
                    let mac = MacAddress::new([0, 0, 0, 0, 0, 1]);
                    let ip =
                        Ipv4Addr::from_str(src.ip().to_canonical().to_string().as_str()).unwrap();
                    let new_manager =
                        PCInfo::new(hostname, mac, ip, pcinfo::PCStatus::Online, true);
                    new_pc_tx.send(new_manager).unwrap();
                    return true;
                }
                Err(_) => {}
            }
        }
        false
    }
}

mod listen {
    use super::*;

    pub fn listen_for_clients(socket: &UdpSocket, new_pc_tx: &Sender<PCInfo>) {
        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((_amt, src)) => {
                // let msg = std::str::from_utf8(&buf[..amt]).unwrap();
                // if msg == "MANAGER" {
                //     return true;
                // }
                let hostname = "pc1".to_string();
                let mac = MacAddress::new([0, 0, 0, 0, 0, 1]);
                let ip = Ipv4Addr::from_str(src.ip().to_canonical().to_string().as_str()).unwrap();
                let new_client = PCInfo::new(hostname, mac, ip, pcinfo::PCStatus::Online, false);
                new_pc_tx.send(new_client).unwrap();
                socket.send_to("MANAGER".as_bytes(), src).unwrap();
            }
            Err(_) => {}
        }
    }
}

pub fn initialize(signals: &Signals, new_pc_tx: &Sender<PCInfo>) {
    let addr = SocketAddr::from(([127, 0, 0, 1], DISCOVERY_PORT));
    let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], DISCOVERY_PORT));
    let socket = UdpSocket::bind(addr).expect("Failed to bind monitor socket");
    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking mode");
    socket
        .set_read_timeout(Some(CHECK_DELAY))
        .expect("Failed to set read timeout");

    while signals.run.load(Ordering::Relaxed) {
        if signals.is_manager.load(Ordering::Relaxed) {
            listen_for_clients(&socket, &new_pc_tx);
        } else {
            let manager_found = find_manager(&socket, &signals, &new_pc_tx);

            if manager_found {
                signals.is_manager.store(true, Ordering::Relaxed);
                signals.manager_found.store(true, Ordering::Relaxed);
            } else {
                socket.set_broadcast(true).unwrap();
                socket
                    .send_to("MANAGER".as_bytes(), broadcast_addr)
                    .unwrap();
                socket.set_broadcast(false).unwrap();
            }

            while signals.manager_found.load(Ordering::Relaxed) {
                std::thread::sleep(CHECK_DELAY);
            }
        }
        std::thread::sleep(WAIT_DELAY);
    }
}
