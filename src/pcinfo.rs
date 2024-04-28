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
pub enum PCStatus {
    Online,
    Offline,
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
        let status = match bytes[1] {
            0x01 => PCStatus::Online,
            0x02 => PCStatus::Offline,
            _ => return Err(()),
        };
        let ip = IpAddr::V4(Ipv4Addr::new(bytes[2], bytes[3], bytes[4], bytes[5]));
        let mac = MacAddress::new([bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11]]);
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
        bytes.push(match self.status {
            PCStatus::Online => 0x01,
            PCStatus::Offline => 0x02,
        });
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
