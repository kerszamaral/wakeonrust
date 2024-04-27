use mac_address::MacAddress;

pub const BUFFER_SIZE: usize = 4096;
pub const HEADER_SIZE: usize = 10;

type PacketType = u8;

pub const SSR_PACKET: PacketType = 0x01;
pub const SSR_ACK_PACKET: PacketType = 0x02;
pub const SSE_PACKET: PacketType = 0x04;
pub const SSD_PACKET: PacketType = 0x05;
pub const SSD_ACK_PACKET: PacketType = 0x06;

const MAGIC_NUMBER: u16 = 0xCA31;

pub fn make_header(packet_type: PacketType, length: usize) -> [u8;HEADER_SIZE] {
    let length = length as u16;
    [
        (MAGIC_NUMBER >> 8) as u8,
        MAGIC_NUMBER as u8,
        0,
        packet_type as u8,
        0,
        0,
        (length >> 8) as u8,
        length as u8,
        0,
        0,
    ]
}

pub fn swap_packet_type(packet: &Vec<u8>, packet_type: PacketType) -> Vec<u8> {
    let mut new_packet = packet.clone();
    new_packet[3] = packet_type as u8;
    new_packet
}

pub fn check_packet(packet: &[u8], expected_packet_type: PacketType) -> bool {
    let magic_number = (packet[0] as u16) << 8 | packet[1] as u16;
    if magic_number != MAGIC_NUMBER {
        return false;
    }
    let packet_type = packet[3];
    if packet_type != expected_packet_type {
        return false;
    }
    true
}

pub fn make_wakeup_packet(mac: &MacAddress) -> Vec<u8> {
    const FF_NUM: usize = 6;
    const WOL_HEADER: [u8; FF_NUM] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    const MAC_NUM: usize = 16;
    const MAC_SIZE: usize = 6;
    let wol_payload: Vec<u8> = mac.bytes().iter().cycle().take(MAC_NUM*MAC_SIZE).cloned().collect();
    [WOL_HEADER.to_vec(), wol_payload].concat()
}