pub const BUFFER_SIZE: usize = 4096;
pub const HEADER_SIZE: usize = 10;

pub const MAGIC_NUMBER: u16 = 0xCA31;

// pub const DATA_PACKET: u16 = 0x0000;
pub const SSR_PACKET: u16 = 0x0001;
pub const SSR_ACK_PACKET: u16 = 0x0002;
// pub const STR_PACKET: u16 = 0x0003;
// pub const SSE_PACKET: u16 = 0x0004;
pub const SSD_PACKET: u16 = 0x0005;
pub const SSD_ACK_PACKET: u16 = 0x0006;
// pub const MAGIC_PACKET: u16 = 0x0007;

pub fn make_header(packet_type: u16, length: usize) -> [u8;HEADER_SIZE] {
    let length = length as u16;
    [
        (MAGIC_NUMBER >> 8) as u8,
        MAGIC_NUMBER as u8,
        (packet_type >> 8) as u8,
        packet_type as u8,
        0,
        0,
        (length >> 8) as u8,
        length as u8,
        0,
        0,
    ]
}

pub fn swap_packet_type(packet: &Vec<u8>, packet_type: u16) -> Vec<u8> {
    let mut new_packet = packet.clone();
    new_packet[2] = (packet_type >> 8) as u8;
    new_packet[3] = packet_type as u8;
    new_packet
}

pub fn check_packet(packet: &[u8], expected_packet_type: u16) -> bool {
    let magic_number = (packet[0] as u16) << 8 | packet[1] as u16;
    if magic_number != MAGIC_NUMBER {
        return false;
    }
    let packet_type = (packet[2] as u16) << 8 | packet[3] as u16;
    if packet_type != expected_packet_type {
        return false;
    }
    true
}