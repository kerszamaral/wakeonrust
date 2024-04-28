use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{mpsc::Receiver, Mutex},
};

use gethostname::gethostname;

use crate::{
    addrs::{REPLICATION_ADDR, REPLICATION_BROADCAST_ADDR},
    delays::CHECK_DELAY,
    packets::{check_packet, make_header, BUFFER_SIZE, HEADER_SIZE, SSREP_PACKET},
    pcinfo::PCInfo,
    signals::Signals,
};

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateType {
    Add,
    Remove,
    Change,
    Drop,
}

fn send_updates(
    socket: &UdpSocket,
    pc_info: &PCInfo,
    update_type: UpdateType,
    send_addr: SocketAddr,
) {
    let mut buf = Vec::new();
    match update_type {
        UpdateType::Add => {
            buf.push(0x01);
        }
        UpdateType::Remove => {
            buf.push(0x02);
        }
        UpdateType::Change => {
            buf.push(0x03);
        }
        UpdateType::Drop => {
            buf.push(0x04);
        }
    }
    buf.extend(pc_info.to_bytes());
    let header = make_header(SSREP_PACKET, buf.len());
    let packet = [header.to_vec(), buf].concat();
    socket.send_to(&packet, send_addr).unwrap();
}

fn receive_updates(buf: &[u8], amt: usize) -> Result<(UpdateType, PCInfo), ()> {
    if check_packet(&buf[..amt], SSREP_PACKET).is_err() {
        return Err(());
    }
    let msg = &buf[HEADER_SIZE..amt];
    let update_type = match msg[0] {
        0x01 => UpdateType::Add,
        0x02 => UpdateType::Remove,
        0x03 => UpdateType::Change,
        0x04 => UpdateType::Drop,
        _ => return Err(()),
    };
    match PCInfo::from_bytes(&buf[1..amt]) {
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
    let mut rb_pc_map = m_pc_map.lock().unwrap().clone();
    let mut was_manager = signals.is_manager();

    while signals.running() {
        if was_manager != signals.is_manager() {
            was_manager = signals.is_manager();
            if was_manager {
                let mut pc_map = m_pc_map.lock().unwrap();
                let our_hostname = gethostname().into_string().unwrap();
                rb_pc_map.remove(&our_hostname); // Remove our own PCInfo
                pc_map.clear();
                for pc_info in rb_pc_map.values() {
                    pc_map.insert(pc_info.get_name().clone(), pc_info.clone());
                }
                signals.send_update();
            }
        }

        if signals.is_manager() {
            match updates.try_recv() {
                Ok((update_type, pc_info)) => {
                    // If we are changing a PC's status to online,
                    // it was offline before and we need to sync the backups
                    if update_type == UpdateType::Change && pc_info.is_online() {
                        let addr =
                            SocketAddr::new(pc_info.get_ip().clone(), REPLICATION_ADDR.port());
                        send_updates(&socket, &pc_info, UpdateType::Drop, addr);
                        for pc_info in rb_pc_map.values() {
                            send_updates(&socket, &pc_info, UpdateType::Add, addr);
                        }
                    }

                    send_updates(&socket, &pc_info, update_type, REPLICATION_BROADCAST_ADDR);
                }
                Err(_) => {
                    std::thread::sleep(CHECK_DELAY);
                }
            }
        } else {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => match receive_updates(&buf, amt) {
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
                        }
                    },
                    Err(_) => {
                        continue;
                    }
                },
                Err(_) => {
                    std::thread::sleep(CHECK_DELAY);
                }
            }
        }
    }
}
