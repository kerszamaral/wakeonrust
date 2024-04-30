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
            ip: ip,
            status,
            is_manager,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<PCInfo, ()> {
        let is_manager = bytes[0] == 0x01;
        let status = PCStatus::try_from(bytes[1])?;
        let ip = IpAddr::V4(Ipv4Addr::new(bytes[2], bytes[3], bytes[4], bytes[5]));
        let mac = MacAddress::new(bytes[6..12].try_into().unwrap());
        let name = String::from_utf8(bytes[12..].to_vec()).unwrap();
        Ok(PCInfo {
            name,
            mac,
            ip,
            status,
            is_manager,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(if self.is_manager { 0x01 } else { 0x00 });
        bytes.push(self.status.clone() as u8);
        match self.ip {
            IpAddr::V4(ip) => {
                bytes.push(ip.octets()[0]);
                bytes.push(ip.octets()[1]);
                bytes.push(ip.octets()[2]);
                bytes.push(ip.octets()[3]);
            }
            _ => {}
        }
        bytes.extend_from_slice(&self.mac.bytes());
        bytes.extend_from_slice(self.name.as_bytes());
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

    pub fn is_online(&self) -> bool {
        self.status == PCStatus::Online
    }

    #[allow(dead_code)]
    pub fn is_offline(&self) -> bool {
        self.status == PCStatus::Offline
    }
}
