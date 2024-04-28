use std::{collections::HashMap, net::UdpSocket, sync::Mutex};

use crate::{
    addrs::{ELECTION_ADDR, ELECTION_BROADCAST_ADDR}, delays::{CHECK_DELAY, ELECTION_FINISHED_DELAY, WAIT_DELAY}, packets::{
        get_packet_type, make_header,
        PacketType::{SselFinPacket, SselGtPacket, SselPacket},
        BUFFER_SIZE,
    }, pcinfo::PCInfo, signals::Signals
};

fn elected(signals: &Signals, socket: &UdpSocket) -> bool {
    // Election variables
    let our_number = rand::random::<u32>();
    let mut someone_is_greater = false;
    const MAX_TURNS: u32 = 5;
    let mut turns_left = MAX_TURNS;

    // Packets
    let gt_packet = make_header(SselGtPacket, 0);
    let packet = make_header(SselPacket, our_number.to_be_bytes().len());
    let mut packet = packet.to_vec();
    packet.extend_from_slice(&our_number.to_be_bytes());
    
    while signals.running() && turns_left > 0 {
        // We check if someone is greater than us
        if !someone_is_greater {
            // We send our number to the network
            socket.send_to(&packet, ELECTION_BROADCAST_ADDR).unwrap();
        }
        let current_turn = turns_left;
        while signals.running() && current_turn == turns_left{
            // We wait for the response
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if let Ok(packe_type) = get_packet_type(&buf[..amt]) {
                        match packe_type {
                            SselFinPacket => {
                                // Election is finished, we wait to find manager
                                // on another thread
                                return false; // Exit election
                            }
                            SselPacket => {
                                // Election is still going on
                                let number = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                                // We compare our number with the received number
                                if our_number > number {
                                    // We are greater than the other
                                    socket.send_to(&gt_packet, src).unwrap();
                                }
                            }
                            SselGtPacket => {
                                // Someone else is greater than us
                                // We back off
                                someone_is_greater = true;
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => {
                    // No response,
                    // We decrement the turn counter
                    turns_left -= 1;
                }
            }
        }
    }
    !someone_is_greater
}

pub fn initialize(signals: &Signals) {
    let socket = UdpSocket::bind(ELECTION_ADDR).unwrap();
    socket.set_read_timeout(Some(WAIT_DELAY)).unwrap();
    socket.set_broadcast(true).unwrap();

    // Packets
    let finished_packet = make_header(SselFinPacket, 0);

    while signals.running() {
        if signals.is_manager() {
            // We flood the network with the election finished packet
            socket
                .send_to(&finished_packet, ELECTION_BROADCAST_ADDR)
                .unwrap();
            std::thread::sleep(ELECTION_FINISHED_DELAY);
            continue;
        } else {
            if !signals.manager_found() {
                // We start the election
                if elected(signals, &socket){
                    signals.i_am_manager();
                }
            } else {
                // We found the manager
                std::thread::sleep(CHECK_DELAY);
                return;
            }
        }
    }
}
