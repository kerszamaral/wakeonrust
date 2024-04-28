use std::net::{SocketAddr, IpAddr, Ipv4Addr};

pub const DEFAULT_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
pub const BROADCAST_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));

pub const WAKEUP_PORT: u16 = 9;
pub const WAKEUP_SEND_PORT: u16 = 10010;
pub const DISCOVERY_PORT: u16 = 10000;
pub const EXIT_PORT: u16 = 12345;
pub const MONITOR_PORT: u16 = 14321;
pub const REPLICATION_PORT: u16 = 14444;

pub const WAKEUP_ADDR: SocketAddr = SocketAddr::new(BROADCAST_ADDR, WAKEUP_PORT);
pub const WAKEUP_SEND_ADDR: SocketAddr = SocketAddr::new(DEFAULT_ADDR, WAKEUP_SEND_PORT);
pub const DISCOVERY_ADDR: SocketAddr = SocketAddr::new(DEFAULT_ADDR, DISCOVERY_PORT);
pub const DISCOVERY_BROADCAST_ADDR: SocketAddr = SocketAddr::new(BROADCAST_ADDR, DISCOVERY_PORT);
pub const EXIT_ADDR: SocketAddr = SocketAddr::new(DEFAULT_ADDR, EXIT_PORT);
pub const EXIT_BROADCAST_ADDR: SocketAddr = SocketAddr::new(BROADCAST_ADDR, EXIT_PORT);
pub const MONITOR_ADDR: SocketAddr = SocketAddr::new(DEFAULT_ADDR, MONITOR_PORT);
pub const REPLICATION_ADDR: SocketAddr = SocketAddr::new(DEFAULT_ADDR, REPLICATION_PORT);
pub const REPLICATION_BROADCAST_ADDR: SocketAddr = SocketAddr::new(BROADCAST_ADDR, REPLICATION_PORT);