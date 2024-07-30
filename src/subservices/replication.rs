use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{mpsc::Receiver, Mutex},
};

use gethostname::gethostname;
use local_ip_address::local_ip;

use crate::{
    addrs::{REPLICATION_ADDR, REPLICATION_BROADCAST_ADDR},
    delays::CHECK_DELAY,
    packets::{HEADER_SIZE, get_packet_length, make_header, PacketType::SsrepPacket, BUFFER_SIZE},
    pcinfo::{PCInfo, PCStatus},
    signals::Signals,
};

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UpdateType {
    Add = 0x01,
    Remove,
    Change,
}

impl std::convert::TryFrom<u8> for UpdateType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(UpdateType::Add),
            0x02 => Ok(UpdateType::Remove),
            0x03 => Ok(UpdateType::Change),
            _ => Err(()),
        }
    }
}

fn receive_update(buf: &[u8]) -> Result<(HashMap<String, PCInfo>, u32), ()> {
    let mut num_entries = get_packet_length(buf);
    let msg = buf[HEADER_SIZE..].to_vec();
    let table_version = u32::from_be_bytes(msg[..4].try_into().unwrap());
    let mut bytes_used: usize = 4;
    let mut pc_map = HashMap::new();
    while num_entries > 0 {
        match PCInfo::from_bytes(&msg[bytes_used..]) {
            Ok((pc_info, used)) => {
                pc_map.insert(pc_info.get_name().clone(), pc_info);
                bytes_used += used;
                num_entries -= 1;
            }
            Err(_) => return Err(()),
        }
    }
    Ok((pc_map, table_version))
}

pub fn serialize_pc_map(pc_map: &HashMap<String, PCInfo>) -> Vec<u8> {
    let mut buf = Vec::new();
    for pc_info in pc_map.values() {
        buf.extend(pc_info.to_bytes());
    }
    buf
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
                if !pc_map.is_empty() {
                    signals.send_update();
                }
            } else {
                // We are no longer the manager
                let mut pc_map = m_pc_map.lock().unwrap();
                // remove everything but the manager, if there is any
                pc_map.retain(|_, v| v.is_manager());
                if !pc_map.is_empty() {
                    signals.send_update();
                }
            }
        }

        if signals.is_manager() {
            match updates.try_recv() {
                Ok((update_type, pc_info)) => {
                    // Update backup table
                    let curr_table_version = signals.update_table_version();
                    match update_type {
                        UpdateType::Add => {
                            rb_pc_map.insert(pc_info.get_name().clone(), pc_info);
                        }
                        UpdateType::Remove => {
                            rb_pc_map.remove(pc_info.get_name());
                        }
                        UpdateType::Change => {
                            rb_pc_map.insert(pc_info.get_name().clone(), pc_info);
                        }
                    }

                    // Serialize the PC map
                    let mut buf = Vec::new();
                    buf.extend(curr_table_version.to_be_bytes().iter());
                    buf.extend(serialize_pc_map(&rb_pc_map).iter());
                    let header = make_header(SsrepPacket, rb_pc_map.len());
                    let packet = [header.to_vec(), buf].concat();

                    // Send the update
                    socket.send_to(&packet, REPLICATION_BROADCAST_ADDR).unwrap();
                }
                Err(_) => std::thread::sleep(CHECK_DELAY),
            }
        } else {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => match receive_update(&buf[..amt]) {
                    Ok((pc_map, table_version)) => {
                        rb_pc_map = pc_map;
                        signals.overwrite_table_version(table_version);
                    }
                    Err(_) => continue,
                },
                Err(_) => std::thread::sleep(CHECK_DELAY),
            }
        }
    }
}
