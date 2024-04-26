/*
Create a struct called PCInfo with the following fields:
- name: String
- mac: MacAddress
- ip: Ipv4Addr
- status: enum Status
- is_manager: bool
*/

use std::net::IpAddr;
extern crate mac_address;
use mac_address::MacAddress;

#[derive(Debug, PartialEq, Eq)]
pub enum PCStatus {
    Online,
    Offline,
}

#[derive(Debug)]
pub struct PCInfo {
    name: String,
    mac: MacAddress,
    ip: IpAddr,
    status: PCStatus,
    is_manager: bool,
}

impl PCInfo {
    pub fn new(name: String, mac: MacAddress, ip: IpAddr, status: PCStatus, is_manager: bool) -> PCInfo {
        PCInfo {
            name,
            mac,
            ip: ip.to_canonical(),
            status,
            is_manager,
        }
    }

    pub fn get_name(&self) -> &String {
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
}

