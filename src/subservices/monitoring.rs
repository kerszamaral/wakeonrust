use crate::delays::{CHECK_DELAY, WAIT_DELAY};
use crate::packets::{check_packet, make_header, BUFFER_SIZE, SSR_ACK_PACKET, SSR_PACKET};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::signals::Signals;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{mpsc::Sender, Mutex};

pub mod status {
    use super::*;
    use crate::addrs::{MONITOR_ADDR, MONITOR_PORT};

    fn response_from_client(signals: &Signals, socket: &UdpSocket) -> PCStatus {
        while signals.running() {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    if check_packet(&buf[..amt], SSR_ACK_PACKET).is_err() {
                        continue; // Ignore invalid packets
                    }
                    return PCStatus::Online;
                }
                Err(_) => {
                    return PCStatus::Offline;
                }
            }
        }
        PCStatus::Offline
    }

    fn listen_for_clients(
        signals: &Signals,
        socket: &UdpSocket,
        pcs: Vec<(&String, &IpAddr, &PCStatus)>,
        sleep_status: &Sender<(String, PCStatus)>,
    ) {
        let ssr = make_header(SSR_PACKET, 0);
        for (hostname, ip, status) in pcs {
            if !signals.running() {
                break;
            }
            let addr = SocketAddr::new(*ip, MONITOR_PORT);
            socket
                .send_to(&ssr, addr)
                .expect("Failed to send to client");

            let new_status = response_from_client(&signals, &socket);
            if new_status == *status {
                continue;
            }
            sleep_status.send((hostname.clone(), new_status)).unwrap();
        }
    }

    pub fn status_monitor(
        signals: &Signals,
        m_pc_map: &Mutex<HashMap<String, PCInfo>>,
        sleep_status: Sender<(String, PCStatus)>,
    ) {
        let socket = UdpSocket::bind(MONITOR_ADDR).expect("Failed to bind monitor socket");
        socket
            .set_read_timeout(Some(WAIT_DELAY))
            .expect("Failed to set monitor socket read timeout");
        const MAX_TIMEOUT: u32 = 5;
        let mut manager_timeout = MAX_TIMEOUT;

        while signals.running() {
            if signals.is_manager() {
                let pc_map = m_pc_map.lock().unwrap();
                let pcs = pc_map
                    .iter()
                    .map(|(k, v)| (k, v.get_ip(), v.get_status()))
                    .collect();

                listen_for_clients(&signals, &socket, pcs, &sleep_status);
            } else {
                let mut buf = [0; BUFFER_SIZE];
                match socket.recv_from(&mut buf) {
                    Ok((amt, src)) => {
                        if check_packet(&buf[..amt], SSR_PACKET).is_err() {
                            continue;
                        }
                        let ssra = make_header(SSR_ACK_PACKET, 0);
                        socket.send_to(&ssra, src).unwrap();
                        manager_timeout = MAX_TIMEOUT;
                    }
                    Err(_) => {
                        if signals.manager_found() {
                            if manager_timeout == 0 {
                                signals.manager_timed_out();
                            } else {
                                manager_timeout -= 1;
                            }
                        }
                    }
                }
            }
            std::thread::sleep(CHECK_DELAY);
        }
    }
}

pub mod exit {
    use crate::{
        addrs::{EXIT_ADDR, EXIT_BROADCAST_ADDR},
        packets::{HEADER_SIZE, SSE_PACKET},
    };

    use super::*;

    pub fn exit_monitor(signals: &Signals, exit_tx: Sender<String>) {
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

                    exit_tx.send(hostname).unwrap();
                }
                Err(_) => {}
            }
        }
        // Send the exit signal to other pcs
        socket.set_broadcast(true).unwrap();
        let exit_packet = make_header(SSE_PACKET, 0);
        socket.send_to(&exit_packet, EXIT_BROADCAST_ADDR).unwrap();
    }
}
