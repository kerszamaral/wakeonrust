use crate::{
    addrs::{WAKEUP_ADDR, WAKEUP_SEND_ADDR},
    delays::CHECK_DELAY,
    packets::make_wakeup_packet,
    pcinfo::{PCInfo, PCStatus},
    signals::Signals,
};
use std::net::UdpSocket;
use std::sync::{mpsc::Receiver, Mutex};
use std::{collections::HashMap, sync::mpsc::Sender};

use super::replication::UpdateType;

pub fn wakeup(
    signals: &Signals,
    m_pc_map: &Mutex<HashMap<String, PCInfo>>,
    wake_rx: Receiver<String>,
) {
    let socket = UdpSocket::bind(WAKEUP_SEND_ADDR).unwrap();
    socket.set_broadcast(true).unwrap();

    while signals.running() {
        match wake_rx.try_recv() {
            Ok(hostname) => {
                let pc_map = m_pc_map.lock().unwrap();
                if let Some(pc_info) = pc_map.get(&hostname) {
                    if *pc_info.get_status() == PCStatus::Offline {
                        let wakeup_packet = make_wakeup_packet(pc_info.get_mac());
                        socket.send_to(&wakeup_packet, WAKEUP_ADDR).unwrap();
                        println!("Waking up {}", hostname);
                    } else {
                        println!("{} is not sleeping", hostname);
                    }
                } else {
                    println!("PC not found");
                }
            }

            Err(_) => {
                std::thread::sleep(CHECK_DELAY);
            }
        }
    }
}

pub fn add_pcs(
    signals: &Signals,
    m_pc_map: &Mutex<HashMap<String, PCInfo>>,
    new_pc_rx: Receiver<PCInfo>,
    rb_update_tx: Sender<(UpdateType, PCInfo)>,
) {
    while signals.running() {
        match new_pc_rx.try_recv() {
            Ok(pc_info) => {
                let mut pc_map = m_pc_map.lock().unwrap();
                pc_map.insert(pc_info.get_hostname().clone(), pc_info.clone());
                rb_update_tx.send((UpdateType::Add, pc_info)).unwrap();
                signals.send_update();
            }
            Err(_) => {
                std::thread::sleep(CHECK_DELAY);
            }
        }
    }
}

pub fn update_statuses(
    signals: &Signals,
    m_pc_map: &Mutex<HashMap<String, PCInfo>>,
    sleep_status_rx: Receiver<(String, PCStatus)>,
    rb_update_tx: Sender<(UpdateType, PCInfo)>,
) {
    while signals.running() {
        match sleep_status_rx.try_recv() {
            Ok((hostname, status)) => {
                let mut pc_map = m_pc_map.lock().unwrap();
                if let Some(pc_info) = pc_map.get_mut(&hostname) {
                    pc_info.set_status(status);
                    rb_update_tx
                        .send((UpdateType::Change, pc_info.clone()))
                        .unwrap();
                    signals.send_update();
                }
            }
            Err(_) => {
                std::thread::sleep(CHECK_DELAY);
            }
        }
    }
}

pub fn remove_pcs(
    signals: &Signals,
    m_pc_map: &Mutex<HashMap<String, PCInfo>>,
    remove_rx: Receiver<String>,
    rb_update_tx: Sender<(UpdateType, PCInfo)>,
) {
    while signals.running() {
        match remove_rx.try_recv() {
            Ok(hostname) => {
                let mut pc_map = m_pc_map.lock().unwrap();
                if let Some(pc_info) = pc_map.remove(&hostname) {
                    rb_update_tx.send((UpdateType::Remove, pc_info)).unwrap();
                    signals.send_update();
                }
            }
            Err(_) => {
                std::thread::sleep(CHECK_DELAY);
            }
        }
    }
}
