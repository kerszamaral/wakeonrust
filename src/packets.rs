use mac_address::MacAddress;

pub const BUFFER_SIZE: usize = 1024;
pub const HEADER_SIZE: usize = 5;

pub type PacketType = u8;

pub const SSR_PACKET: PacketType = 0x01;
pub const SSR_ACK_PACKET: PacketType = 0x02;
pub const SSE_PACKET: PacketType = 0x04;
pub const SSD_PACKET: PacketType = 0x05;
pub const SSD_ACK_PACKET: PacketType = 0x06;
pub const SSREP_PACKET: PacketType = 0x07;

const MAGIC_NUMBER: u16 = 0xCA31;
const MAGIC_NUMBER_INDEX: usize = 0;
const PACKET_TYPE_INDEX: usize = 2;
const LENGTH_INDEX: usize = 3;

pub fn make_header(packet_type: PacketType, length: usize) -> [u8; HEADER_SIZE] {
    let length = length as u16;
    [
        (MAGIC_NUMBER >> 8) as u8,
        MAGIC_NUMBER as u8,
        packet_type as u8,
        (length >> 8) as u8,
        length as u8,
    ]
}

pub fn swap_packet_type(packet: &Vec<u8>, packet_type: PacketType) -> Vec<u8> {
    let mut new_packet = packet.clone();
    new_packet[PACKET_TYPE_INDEX] = packet_type as u8;
    new_packet
}

pub fn check_packet(packet: &[u8], expected_packet_type: PacketType) -> Result<usize, ()> {
    let magic_number =
        (packet[MAGIC_NUMBER_INDEX] as u16) << 8 | packet[MAGIC_NUMBER_INDEX + 1] as u16;
    if magic_number != MAGIC_NUMBER {
        return Err(());
    }
    let packet_type = packet[PACKET_TYPE_INDEX];
    if packet_type != expected_packet_type {
        return Err(());
    }
    let length = (packet[LENGTH_INDEX] as usize) << 8 | packet[LENGTH_INDEX + 1] as usize;
    Ok(length)
}

pub fn make_wakeup_packet(mac: &MacAddress) -> Vec<u8> {
    const FF_NUM: usize = 6;
    const WOL_HEADER: [u8; FF_NUM] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    const MAC_NUM: usize = 16;
    const MAC_SIZE: usize = 6;
    let wol_payload: Vec<u8> = mac.bytes().iter().cycle().take(MAC_NUM*MAC_SIZE).cloned().collect();
    [WOL_HEADER.to_vec(), wol_payload].concat()
}
