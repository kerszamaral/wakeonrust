use crate::delays::{CHECK_DELAY, WAIT_DELAY};
use crate::packets::{check_packet, make_header, BUFFER_SIZE, HEADER_SIZE, SSE_PACKET};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::signals::Signals;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{mpsc::Receiver, Mutex};

pub mod exit {
    use crate::addrs::{EXIT_ADDR, EXIT_BROADCAST_ADDR, EXIT_SEND_ADDR};

    use super::*;

    pub fn sender(signals: &Signals) {
        while signals.running() {
            std::thread::sleep(CHECK_DELAY);
        }
        let socket = UdpSocket::bind(EXIT_SEND_ADDR).unwrap();
        socket.set_broadcast(true).unwrap();
        let exit_packet = make_header(SSE_PACKET, 0);
        socket.send_to(&exit_packet, EXIT_BROADCAST_ADDR).unwrap();
    }

    pub fn receiver(signals: &Signals, m_pc_map: &Mutex<HashMap<String, PCInfo>>) {
        let socket = UdpSocket::bind(EXIT_ADDR).unwrap();
        socket.set_read_timeout(Some(WAIT_DELAY)).unwrap();

        while signals.running() {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    if check_packet(&buf[..amt], SSE_PACKET).is_err() {
                        continue; // Ignore invalid packets
                    }

                    let hostname = buf[HEADER_SIZE..amt]
                        .iter()
                        .map(|&c| c as char)
                        .collect::<String>();

                    let mut pc_map = m_pc_map.lock().unwrap();
                    pc_map.remove(&hostname);
                }
                Err(_) => {}
            }
        }
    }
}

pub mod wakeup {
    use crate::{packets::make_wakeup_packet, addrs::{WAKEUP_ADDR, WAKEUP_SEND_ADDR}};

    use super::*;

    pub fn sender(
        signals: &Signals,
        m_pc_map: &Mutex<HashMap<String, PCInfo>>,
        wake_rx: Receiver<String>,
    ) {
        let socket = UdpSocket::bind(WAKEUP_SEND_ADDR).unwrap();
        socket.set_broadcast(true).unwrap();

        while signals.running() {
            std::thread::sleep(CHECK_DELAY);
            if let Ok(hostname) = wake_rx.try_recv() {
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
        }
    }
}

pub mod update {
    use super::*;

    pub fn add_pcs(
        signals: &Signals,
        m_pc_map: &Mutex<HashMap<String, PCInfo>>,
        new_pc_rx: Receiver<PCInfo>,
    ) {
        while signals.running() {
            std::thread::sleep(CHECK_DELAY);
            if let Ok(pc_info) = new_pc_rx.try_recv() {
                let mut pc_map = m_pc_map.lock().unwrap();
                pc_map.insert(pc_info.get_hostname().clone(), pc_info);
                signals.send_update();
            }
        }
    }

    pub fn update_statuses(
        signals: &Signals,
        m_pc_map: &Mutex<HashMap<String, PCInfo>>,
        sleep_status_rx: Receiver<(String, PCStatus)>,
    ) {
        while signals.running() {
            std::thread::sleep(CHECK_DELAY);
            if let Ok((hostname, status)) = sleep_status_rx.try_recv() {
                let mut pc_map = m_pc_map.lock().unwrap();
                if let Some(pc_info) = pc_map.get_mut(&hostname) {
                    pc_info.set_status(status);
                    signals.send_update();
                }
            }
        }
    }
}
