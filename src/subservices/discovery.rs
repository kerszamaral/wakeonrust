use self::find::find_manager;
use self::listen::listen_for_clients;
use crate::packets::{
    check_packet, make_header, swap_packet_type, BUFFER_SIZE, HEADER_SIZE, SSD_ACK_PACKET,
    SSD_PACKET,
};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::{
    delays::{CHECK_DELAY, WAIT_DELAY},
    ports::DISCOVERY_PORT,
    signals::Signals,
};
use mac_address::MacAddress;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{atomic::Ordering, mpsc::Sender};

fn from_buffer(buf: &[u8], amt: usize) -> Option<(String, MacAddress)> {
    let msg = &buf[..amt];
    if !check_packet(&msg.to_vec(), SSD_PACKET) {
        return None;
    }
    let msg = &buf[HEADER_SIZE..amt];

    const MAC_SIZE: usize = 6;
    let hostname_bytes = &msg[MAC_SIZE..];
    let mac_bytes = &msg[..MAC_SIZE];

    let hostname = String::from_utf8(hostname_bytes.to_vec()).unwrap_or("".to_string());
    let mac = MacAddress::new(mac_bytes.try_into().unwrap_or([0 as u8; MAC_SIZE]));
    return Some((hostname, mac));
}

mod find {
    use super::*;

    pub fn find_manager(socket: &UdpSocket, new_pc_tx: &Sender<PCInfo>) -> bool {
        let mut buf = [0; BUFFER_SIZE];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let (hostname, mac) = match from_buffer(&buf, amt) {
                    Some((hostname, mac)) => (hostname, mac),
                    None => return false,
                };

                let new_manager = PCInfo::new(hostname, mac, src.ip(), PCStatus::Online, true);
                new_pc_tx.send(new_manager).unwrap();
                return true;
            }
            Err(_) => {
                return false;
            }
        }
    }
}

mod listen {
    use super::*;

    pub fn listen_for_clients(socket: &UdpSocket, new_pc_tx: &Sender<PCInfo>, ssra: &Vec<u8>) {
        let mut buf = [0; BUFFER_SIZE];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let (hostname, mac) = match from_buffer(&buf, amt) {
                    Some((hostname, mac)) => (hostname, mac),
                    None => return,
                };

                let new_client = PCInfo::new(hostname, mac, src.ip(), PCStatus::Online, false);
                new_pc_tx.send(new_client).unwrap();
                socket.send_to(&ssra, src).unwrap();
            }
            Err(_) => {}
        }
    }
}

pub fn initialize(signals: &Signals, new_pc_tx: &Sender<PCInfo>) {
    // Setup the socket
    let addr = SocketAddr::from(([127, 0, 0, 1], DISCOVERY_PORT));
    let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], DISCOVERY_PORT));
    let socket = UdpSocket::bind(addr).expect("Failed to bind monitor socket");
    socket
        .set_nonblocking(true)
        .expect("Failed to set discovery socket to non-blocking mode");
    socket
        .set_read_timeout(Some(CHECK_DELAY))
        .expect("Failed to set discovery socket read timeout");

    // Setup the SSR packet
    let our_mac = mac_address::get_mac_address()
        .unwrap()
        .expect("Failed to get MAC address");
    let our_name = gethostname::gethostname().into_string().unwrap();
    let length = our_mac.bytes().len() + our_name.as_bytes().len();

    // Make the SSR packet and its ACK
    let ssr = [
        make_header(SSD_PACKET, length).to_vec(),
        our_mac.bytes().to_vec(),
        our_name.as_bytes().to_vec(),
    ]
    .concat();
    let ssra = swap_packet_type(&ssr, SSD_ACK_PACKET);

    while signals.run.load(Ordering::Relaxed) {
        if signals.is_manager.load(Ordering::Relaxed) {
            listen_for_clients(&socket, &new_pc_tx, &ssra);
        } else {
            let manager_found = find_manager(&socket, &new_pc_tx);

            if manager_found {
                signals.manager_found.store(true, Ordering::Relaxed);
            } else {
                socket.set_broadcast(true).unwrap();
                socket.send_to(&ssr, broadcast_addr).unwrap();
                socket.set_broadcast(false).unwrap();
            }

            while signals.manager_found.load(Ordering::Relaxed) {
                std::thread::sleep(CHECK_DELAY);
            }
        }
        std::thread::sleep(WAIT_DELAY);
    }
}
