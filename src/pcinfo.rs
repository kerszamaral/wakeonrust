/*
Create a struct called PCInfo with the following fields:
- name: String
- mac: MacAddress
- ip: Ipv4Addr
- status: enum Status
- is_manager: bool
*/

use std::net::{IpAddr, Ipv4Addr};
extern crate mac_address;
use mac_address::MacAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PCStatus {
    Online = 0x01,
    Offline,
}

impl std::convert::TryFrom<u8> for PCStatus {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(PCStatus::Online),
            0x02 => Ok(PCStatus::Offline),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PCInfo {
    name: String,
    mac: MacAddress,
    ip: IpAddr,
    status: PCStatus,
    is_manager: bool,
}

impl PCInfo {
    pub fn new(
        name: String,
        mac: MacAddress,
        ip: IpAddr,
        status: PCStatus,
        is_manager: bool,
    ) -> PCInfo {
        PCInfo {
            name,
            mac,
            ip,
            status,
            is_manager,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(PCInfo, usize), ()> {
        const HSTNM_LEN_BYTES: usize = std::mem::size_of::<usize>();
        let hostname_len_bytes = match bytes[..HSTNM_LEN_BYTES].try_into() {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(())},
        };
        let hostname_len = usize::from_be_bytes(
            hostname_len_bytes,
        ) as usize;
        let mut bytes_used: usize = HSTNM_LEN_BYTES;

        let hostname = String::from_utf8(bytes[bytes_used..bytes_used + hostname_len].to_vec()).unwrap();
        bytes_used += hostname_len;
    
        let mac = MacAddress::new(
            bytes[bytes_used..bytes_used + 6]
                .try_into()
                .map_err(|_| ())?,
        );
        bytes_used += 6;

        let ip = IpAddr::V4(Ipv4Addr::new(
            bytes[bytes_used],
            bytes[bytes_used + 1],
            bytes[bytes_used + 2],
            bytes[bytes_used + 3],
        ));
        bytes_used += 4;

        let status = PCStatus::try_from(bytes[bytes_used]).map_err(|_| ())?;
        bytes_used += 1;

        let is_manager = bytes[bytes_used] == 0x01;
        bytes_used += 1;
        
        Ok((PCInfo {
            name: hostname,
            mac,
            ip,
            status,
            is_manager,
        }, bytes_used))

    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.get_hostname().len().to_be_bytes().iter());
        bytes.extend(self.get_hostname().as_bytes());
        bytes.extend(self.get_mac().bytes().iter());
        let ip_octets = match self.get_ip() {
            IpAddr::V4(ip) => ip.octets(),
            _ => [0, 0, 0, 0],
        };
        bytes.extend(ip_octets.iter());
        bytes.push(self.status.clone() as u8);
        bytes.push(if self.is_manager { 0x01 } else { 0x00 });
        bytes
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_hostname(&self) -> &String {
        &self.name
    }

    pub fn get_mac(&self) -> &MacAddress {
        &self.mac
    }

    pub fn get_ip(&self) -> &IpAddr {
        &self.ip
    }

    pub fn get_status(&self) -> &PCStatus {
        &self.status
    }

    pub fn get_is_manager(&self) -> &bool {
        &self.is_manager
    }

    pub fn set_is_manager(&mut self, is_manager: bool) {
        self.is_manager = is_manager;
    }

    pub fn is_manager(&self) -> bool {
        self.is_manager
    }

    pub fn set_status(&mut self, status: PCStatus) {
        self.status = status;
    }

    #[allow(dead_code)]
    pub fn is_online(&self) -> bool {
        self.status == PCStatus::Online
    }

    #[allow(dead_code)]
    pub fn is_offline(&self) -> bool {
        self.status == PCStatus::Offline
    }
}
