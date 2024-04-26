use crate::delays::{CHECK_DELAY, WAIT_DELAY};
use crate::packets::{check_packet, make_header, BUFFER_SIZE, HEADER_SIZE, SSE_PACKET};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::ports::{EXIT_PORT, WAKEUP_PORT};
use crate::signals::Signals;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{atomic::Ordering, mpsc::Receiver, Mutex};

mod exit {
    use super::*;

    pub fn sender(signals: &Signals) {
        signals.run.store(false, Ordering::Relaxed);
        let exit_addr = SocketAddr::from(([255, 255, 255, 255], EXIT_PORT));
        let socket = UdpSocket::bind(exit_addr).unwrap();
        socket.set_broadcast(true).unwrap();
        let exit_packet = make_header(SSE_PACKET, 0);
        socket.send_to(&exit_packet, exit_addr).unwrap();
    }

    pub fn receiver(signals: &Signals, m_pc_map: &Mutex<HashMap<String, PCInfo>>) {
        let exit_addr = SocketAddr::from(([0, 0, 0, 0], EXIT_PORT));
        let socket = UdpSocket::bind(exit_addr).unwrap();
        socket.set_nonblocking(true).unwrap();
        socket.set_read_timeout(Some(CHECK_DELAY)).unwrap();

        while signals.run.load(Ordering::Relaxed) {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    if !check_packet(&buf[..amt], SSE_PACKET) {
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
            std::thread::sleep(WAIT_DELAY);
        }
    }
}

mod wakeup {
    use crate::packets::make_wakeup_packet;

    use super::*;

    pub fn sender(signals: &Signals, wake_rx: &Receiver<String>, m_pc_map: &Mutex<HashMap<String, PCInfo>>) {
        let addr = SocketAddr::from(([0,0,0,0], WAKEUP_PORT));
        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_broadcast(true).unwrap();

        while signals.run.load(Ordering::Relaxed) {
            std::thread::sleep(CHECK_DELAY);
            if let Ok(hostname) = wake_rx.try_recv() {
                let pc_map = m_pc_map.lock().unwrap();
                if let Some(pc_info) = pc_map.get(&hostname) {
                    if *pc_info.get_status() == PCStatus::Offline {
                        let wakeup_packet = make_wakeup_packet(pc_info.get_mac());
                        socket.send_to(&wakeup_packet, addr).unwrap();
                    }
                    else {
                        println!("{} is not sleeping", hostname);
                    }
                }
                else {
                    println!("PC not found");
                }
            }
        }
    }
}


pub fn initialize(
    _signals: &Signals,
    _pc_map: &Mutex<HashMap<String, PCInfo>>,
    _new_pc_rx: &Receiver<PCInfo>,
    _wake_rx: &Receiver<String>,
    _sleep_status_rx: &Receiver<(String, PCStatus)>,
) {
}
