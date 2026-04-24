use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

use crate::xtrem::{DataAddress, Frame, Function, XtremRequest};

pub struct Scales {
    sock_rx: UdpSocket,
    sock_tx: UdpSocket,

    weight_0: Option<f64>,
    weight_1: Option<f64>,

    cmds: Vec<Vec<u8>>,
}

impl Scales {
    pub fn new() -> Self {
        let (sock_rx, sock_tx) = setup_sockets(5555, "192.168.4.255:4444");

        let device_ids = [0x01, 0x02];
        let cmds: Vec<Vec<u8>> = device_ids.iter().map(|&id| build_request(id)).collect();

        Self {
            sock_rx,
            sock_tx,
            weight_0: None,
            weight_1: None,
            cmds,
        }
    }

    pub fn update(&mut self) {
        send_requests(&self.sock_tx, &self.cmds);
        let (weight_0, weight_1) = collect_data(&self.sock_rx);
        self.weight_0 = weight_0;
        self.weight_1 = weight_1;
    }

    pub fn weight_0(&self) -> Option<f64> {
        let tare: f64 = 0.8;
        self.weight_0.map(|x| x - tare)
    }

    pub fn weight_1(&self) -> Option<f64> {
        let tare: f64 = 0.4;
        self.weight_1.map(|x| x - tare)
    }

    pub fn weight_total(&self) -> Option<f64> {
        if self.weight_0().is_none() && self.weight_1().is_none() {
            return None;
        }

        let mut total: f64 = 0.0;

        if let Some(weight) = self.weight_0() {
            total += weight;
        }

        if let Some(weight) = self.weight_1() {
            total += weight;
        }

        return Some(total);
    }
}

fn setup_sockets(rx_port: u16, tx_addr: &str) -> (UdpSocket, UdpSocket) {
    let sock_rx = UdpSocket::bind(("0.0.0.0", rx_port)).unwrap();
    let _ = sock_rx.set_nonblocking(true);

    let sock_tx = UdpSocket::bind("0.0.0.0:0").unwrap();
    sock_tx.set_broadcast(true).unwrap();
    sock_tx.connect(tx_addr).unwrap();

    (sock_rx, sock_tx)
}

/// Helper: Build frame for a device ID
fn build_request(dest_id: u8) -> Vec<u8> {
    let request = XtremRequest {
        id_origin: 0x00,
        id_dest: dest_id,
        data_address: DataAddress::Weight,
        function: Function::ReadRequest,
        data: Vec::new(),
    };
    let frame: Frame = request.into();
    frame.as_bytes()
}

/// Helper: Send requests
fn send_requests(sock_tx: &UdpSocket, cmds: &[Vec<u8>]) {
    for cmd in cmds {
        sock_tx.send(cmd).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn collect_data(sock_rx: &UdpSocket) -> (Option<f64>, Option<f64>) {
    let start = Instant::now();
    let timeout = Duration::from_millis(100);

    let mut buf = [0u8; 2048];

    let mut weight_0: Option<f64> = None;
    let mut weight_1: Option<f64> = None;

    while start.elapsed() < timeout {
        match sock_rx.recv(&mut buf) {
            Ok(n) => {
                if let Some((id, weight)) = parse_response(&buf[..n]) {
                    _ = id;
                    if weight_0.is_some() {
                        weight_1 = Some(weight);
                        break;
                    } else {
                        weight_0 = Some(weight);
                    }
                } else {
                    println!("Failed to parse response...");
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                println!("Err: Socket error: {:?}", e);
                break;
            }
        }
    }
    (weight_0, weight_1)
}

/// Helper: Parse a single response
fn parse_response(buf: &[u8]) -> Option<(u8, f64)> {
    let clean: String = buf
        .iter()
        .filter(|b| b.is_ascii_graphic() || **b == b' ')
        .map(|&b| b as char)
        .collect();

    if clean.len() < 2 {
        return None;
    }

    let id_str = &clean[0..2];
    if let std::result::Result::Ok(id) = id_str.parse::<u8>() {
        let weight = Frame::parse_weight_from_response(buf);
        Some((id, weight))
    } else {
        println!("Failed to parse ID from '{id_str}'");
        None
    }
}
