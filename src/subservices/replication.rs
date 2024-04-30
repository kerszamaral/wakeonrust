use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{mpsc::Receiver, Mutex},
};

use gethostname::gethostname;
use local_ip_address::local_ip;

use crate::{
    addrs::{REPLICATION_ADDR, REPLICATION_BROADCAST_ADDR},
    delays::CHECK_DELAY,
    packets::{get_payload_typed, make_header, PacketType::SsrepPacket, BUFFER_SIZE},
    pcinfo::{PCInfo, PCStatus},
    signals::Signals,
};

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UpdateType {
    Add = 0x01,
    Remove,
    Change,
    Drop,
}

impl std::convert::TryFrom<u8> for UpdateType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(UpdateType::Add),
            0x02 => Ok(UpdateType::Remove),
            0x03 => Ok(UpdateType::Change),
            0x04 => Ok(UpdateType::Drop),
            _ => Err(()),
        }
    }
}

fn send_updates(
    socket: &UdpSocket,
    pc_info: &PCInfo,
    update_type: UpdateType,
    send_addr: SocketAddr,
) {
    let mut buf = Vec::new();
    buf.push(update_type as u8);
    buf.extend(pc_info.to_bytes());
    let header = make_header(SsrepPacket, buf.len());
    let packet = [header.to_vec(), buf].concat();
    socket.send_to(&packet, send_addr).unwrap();
}

fn receive_updates(buf: &[u8]) -> Result<(UpdateType, PCInfo), ()> {
    let msg = get_payload_typed(&buf, SsrepPacket)?;
    let update_type = UpdateType::try_from(msg[0])?;
    match PCInfo::from_bytes(&msg[1..]) {
        Ok(pc_info) => Ok((update_type, pc_info)),
        Err(_) => Err(()),
    }
}

pub fn initialize(
    signals: &Signals,
    m_pc_map: &Mutex<HashMap<String, PCInfo>>,
    updates: Receiver<(UpdateType, PCInfo)>,
) {
    let socket = UdpSocket::bind(REPLICATION_ADDR).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.set_broadcast(true).unwrap();
    let mut rb_pc_map = m_pc_map.lock().unwrap().clone();
    let mut was_manager = signals.is_manager();

    // Our own PCInfo
    let our_hostname = gethostname().into_string().unwrap();
    let our_mac = mac_address::get_mac_address()
        .unwrap()
        .expect("Failed to get MAC address");
    let our_ip = local_ip().expect("Failed to get local IP address");
    // if we're the manager, when people net 
    let our_status = PCStatus::Online;
    let ourselves = PCInfo::new(our_hostname.clone(), our_mac, our_ip, our_status, false);
    rb_pc_map.insert(our_hostname.clone(), ourselves);

    while signals.running() {
        if was_manager != signals.is_manager() {
            was_manager = signals.is_manager();
            if was_manager {
                let mut pc_map = m_pc_map.lock().unwrap();
                pc_map.clear();
                for pc_info in rb_pc_map.values_mut() {
                    if *pc_info.get_name() == our_hostname {
                        // We don't want to add ourselves to the map
                        continue;
                    }
                    if *pc_info.get_is_manager() {
                        pc_info.set_is_manager(false);
                    }
                    pc_map.insert(pc_info.get_name().clone(), pc_info.clone());
                }
                signals.send_update();
            }
        }

        if signals.is_manager() {
            match updates.try_recv() {
                Ok((update_type, pc_info)) => {
                    // If a new pc joins,
                    // or it's coming back online,
                    // We need to send it the current state of the backup
                    if update_type == UpdateType::Add
                        || (update_type == UpdateType::Change && pc_info.is_online())
                    {
                        let addr =
                            SocketAddr::new(pc_info.get_ip().clone(), REPLICATION_ADDR.port());
                        send_updates(&socket, &pc_info, UpdateType::Drop, addr);
                        for pc_info in rb_pc_map.values() {
                            send_updates(&socket, &pc_info, UpdateType::Add, addr);
                        }
                    }

                    send_updates(&socket, &pc_info, update_type, REPLICATION_BROADCAST_ADDR);
                }
                Err(_) => std::thread::sleep(CHECK_DELAY),
            }
        } else {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => match receive_updates(&buf[..amt]) {
                    Ok((update_type, pc_info)) => match update_type {
                        UpdateType::Add => {
                            rb_pc_map.insert(pc_info.get_name().clone(), pc_info);
                        }
                        UpdateType::Remove => {
                            rb_pc_map.remove(pc_info.get_name());
                        }
                        UpdateType::Change => {
                            rb_pc_map.insert(pc_info.get_name().clone(), pc_info);
                        }
                        UpdateType::Drop => {
                            rb_pc_map.clear();
                            signals.relinquish_management();
                        }
                    },
                    Err(_) => continue,
                },
                Err(_) => std::thread::sleep(CHECK_DELAY),
            }
        }
    }
}
