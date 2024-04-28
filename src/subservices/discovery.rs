use crate::addrs::{DISCOVERY_ADDR, DISCOVERY_BROADCAST_ADDR};
use crate::packets::{
    check_packet, make_header, swap_packet_type, PacketType,
    PacketType::{SsdAckPacket, SsdPacket},
    BUFFER_SIZE, HEADER_SIZE,
};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::{
    delays::{CHECK_DELAY, WAIT_DELAY},
    signals::Signals,
};
use gethostname::gethostname;
use mac_address::MacAddress;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;

fn from_buffer(buf: &[u8], amt: usize, packet_type: PacketType) -> Option<(String, MacAddress)> {
    let msg = &buf[..amt];
    if check_packet(&msg.to_vec(), packet_type).is_err() {
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

pub fn find_manager(socket: &UdpSocket, new_pc_tx: &Sender<PCInfo>) -> bool {
    let mut buf = [0; BUFFER_SIZE];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            let (hostname, mac) = match from_buffer(&buf, amt, SsdAckPacket) {
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

pub fn listen_for_clients(socket: &UdpSocket, new_pc_tx: &Sender<PCInfo>, ssra: &Vec<u8>, our_hosname: &String) {
    let mut buf = [0; BUFFER_SIZE];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            let (hostname, mac) = match from_buffer(&buf, amt, SsdPacket) {
                Some((hostname, mac)) => (hostname, mac),
                None => return,
            };

            if hostname == *our_hosname {
                return;
            }

            let new_client = PCInfo::new(hostname, mac, src.ip(), PCStatus::Online, false);
            new_pc_tx.send(new_client).unwrap();
            socket.send_to(&ssra, src).unwrap();
        }
        Err(_) => {}
    }
}

pub fn discover(signals: &Signals, new_pc_tx: Sender<PCInfo>) {
    // Setup the socket
    let socket = UdpSocket::bind(DISCOVERY_ADDR).expect("Failed to bind monitor socket");
    socket
        .set_nonblocking(true)
        .expect("Failed to set discovery socket to non-blocking mode");
    socket
        .set_read_timeout(Some(CHECK_DELAY))
        .expect("Failed to set discovery socket read timeout");
    socket.set_broadcast(true).unwrap();
    let our_hostname = gethostname().into_string().unwrap();

    // Setup the SSR packet
    let our_mac = mac_address::get_mac_address()
        .unwrap()
        .expect("Failed to get MAC address");
    let our_name = gethostname::gethostname().into_string().unwrap();
    let length = our_mac.bytes().len() + our_name.as_bytes().len();

    // Make the SSR packet and its ACK
    let ssr = [
        make_header(SsdPacket, length).to_vec(),
        our_mac.bytes().to_vec(),
        our_name.as_bytes().to_vec(),
    ]
    .concat();
    let ssra = swap_packet_type(&ssr, SsdAckPacket);

    while signals.running() {
         if signals.is_manager() {
            listen_for_clients(&socket, &new_pc_tx, &ssra, &our_hostname);
        } else {
            let manager_found = find_manager(&socket, &new_pc_tx);

            if manager_found {
                signals.found_manager();
            } else {
                socket.send_to(&ssr, DISCOVERY_BROADCAST_ADDR).unwrap();
            }

            while signals.running() && signals.manager_found() {
                std::thread::sleep(CHECK_DELAY);
            }
        }
        std::thread::sleep(WAIT_DELAY);
    }
}
