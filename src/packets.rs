use mac_address::MacAddress;

pub const BUFFER_SIZE: usize = 1024;
pub const HEADER_SIZE: usize = 10;

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    SsrPacket = 0x01,
    SsrAckPacket = 0x02,
    SsePacket = 0x04,
    SsdPacket = 0x05,
    SsdAckPacket = 0x06,
    SsrepPacket = 0x07,
    SselPacket = 0x08,
    SselFinPacket = 0x09,
    SselGtPacket = 0x0A,
}

impl std::convert::TryFrom<u8> for PacketType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(PacketType::SsrPacket),
            0x02 => Ok(PacketType::SsrAckPacket),
            0x04 => Ok(PacketType::SsePacket),
            0x05 => Ok(PacketType::SsdPacket),
            0x06 => Ok(PacketType::SsdAckPacket),
            0x07 => Ok(PacketType::SsrepPacket),
            0x08 => Ok(PacketType::SselPacket),
            0x09 => Ok(PacketType::SselFinPacket),
            0x0A => Ok(PacketType::SselGtPacket),
            _ => Err(()),
        }
    }
}

const MAGIC_NUMBER: u16 = 0xCA31;
const MAGIC_NUMBER_INDEX: usize = 0;
const PACKET_TYPE_INDEX: usize = 3;
const LENGTH_INDEX: usize = 6;

pub fn make_header(packet_type: PacketType, length: usize) -> [u8; HEADER_SIZE] {
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
    new_packet[PACKET_TYPE_INDEX] = packet_type as u8;
    new_packet
}

pub fn get_packet_type(packet: &[u8]) -> Result<PacketType, ()> {
    let magic_number =
        (packet[MAGIC_NUMBER_INDEX] as u16) << 8 | packet[MAGIC_NUMBER_INDEX + 1] as u16;
    if magic_number != MAGIC_NUMBER {
        Err(())
    } else {
        PacketType::try_from(packet[PACKET_TYPE_INDEX])
    }
}

pub fn get_packet_length(packet: &[u8]) -> usize {
    let length = (packet[LENGTH_INDEX] as usize) << 8 | packet[LENGTH_INDEX + 1] as usize;
    length
}

pub fn check_packet(packet: &[u8], expected_packet_type: PacketType) -> Result<usize, ()> {
    let packet_type = get_packet_type(packet)?;
    if packet_type != expected_packet_type {
        return Err(());
    }
    Ok(get_packet_length(packet))
}

pub fn get_payload(packet: &[u8]) -> Result<Vec<u8>, ()> {
    let length = get_packet_length(packet);
    Ok(packet[HEADER_SIZE..HEADER_SIZE + length].to_vec())
}

pub fn get_payload_typed(packet: &[u8], expected_packet_type: PacketType) -> Result<Vec<u8>, ()> {
    check_packet(packet, expected_packet_type)?;
    get_payload(packet)
}

pub fn make_wakeup_packet(mac: &MacAddress) -> Vec<u8> {
    const FF_NUM: usize = 6;
    const WOL_HEADER: [u8; FF_NUM] = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    const MAC_NUM: usize = 16;
    const MAC_SIZE: usize = 6;
    let wol_payload: Vec<u8> = mac
        .bytes()
        .iter()
        .cycle()
        .take(MAC_NUM * MAC_SIZE)
        .cloned()
        .collect();
    [WOL_HEADER.to_vec(), wol_payload].concat()
}
