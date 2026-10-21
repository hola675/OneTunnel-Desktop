//! Bounded SOCKS5 CONNECT probe to a fixed public TLS port.
//! CONNECT success proves a remote TCP circuit, not TLS/HTTPS integrity or VPN identity.
use std::{io::{Read, Write}, net::{Ipv4Addr, SocketAddr, TcpStream}, time::{Duration, Instant}};

const TARGET_HOST: &str = "www.cloudflare.com";
const TARGET_PORT: u16 = 443;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressProbe {
    pub verification_level: &'static str,
    pub target_port: u16,
    pub tcp_connected: bool,
    pub elapsed_ms: Option<u128>,
    pub target: &'static str,
    pub message: String,
}

fn connect_via_socks(port: u16) -> Result<u128, String> {
    let timeout = Duration::from_secs(4);
    let start = Instant::now();
    let mut stream = TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), timeout)
        .map_err(|_| "Local SOCKS5 service unavailable")?;
    stream.set_read_timeout(Some(timeout)).map_err(|_| "Cannot set SOCKS read timeout")?;
    stream.set_write_timeout(Some(timeout)).map_err(|_| "Cannot set SOCKS write timeout")?;
    stream.write_all(&[5, 1, 0]).map_err(|_| "SOCKS greeting failed")?;
    let mut greeting = [0; 2];
    stream.read_exact(&mut greeting).map_err(|_| "SOCKS greeting unanswered")?;
    if greeting != [5, 0] { return Err("SOCKS authentication method rejected".into()); }
    let host = TARGET_HOST.as_bytes();
    let mut request = vec![5, 1, 0, 3, host.len() as u8];
    request.extend_from_slice(host);
    request.extend_from_slice(&TARGET_PORT.to_be_bytes());
    stream.write_all(&request).map_err(|_| "SOCKS CONNECT request failed")?;
    let mut header = [0; 4];
    stream.read_exact(&mut header).map_err(|_| "SOCKS CONNECT reply missing")?;
    if header[0] != 5 || header[2] != 0 { return Err("Malformed SOCKS CONNECT reply".into()); }
    if header[1] != 0 { return Err(format!("SOCKS CONNECT rejected (code {})", header[1])); }
    let address_len = match header[3] {
        1 => 4,
        4 => 16,
        3 => { let mut len = [0]; stream.read_exact(&mut len).map_err(|_| "Malformed SOCKS address")?; len[0] as usize },
        _ => return Err("Invalid SOCKS address type".into()),
    };
    let mut rest = vec![0; address_len + 2];
    stream.read_exact(&mut rest).map_err(|_| "Incomplete SOCKS CONNECT reply")?;
    Ok(start.elapsed().as_millis())
}

/// A remote TCP check is advisory and must never be treated as VPN identity proof.
pub fn probe(port: u16) -> EgressProbe {
    match connect_via_socks(port) {
        Ok(elapsed_ms) => EgressProbe { verification_level: "remote_tcp_only", target_port: TARGET_PORT, tcp_connected: true, elapsed_ms: Some(elapsed_ms), target: TARGET_HOST, message: "Remote TCP CONNECT accepted; HTTPS and VPN exit not verified".into() },
        Err(message) => EgressProbe { verification_level: "unverified", target_port: TARGET_PORT, tcp_connected: false, elapsed_ms: None, target: TARGET_HOST, message },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};
    #[test]
    fn unreachable_listener_never_reports_tcp_success() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let result = probe(port);
        assert!(!result.tcp_connected);
        assert_eq!(result.verification_level, "unverified");
        assert!(result.elapsed_ms.is_none());
    }
    #[test]
    fn successful_connect_is_only_tcp_verified() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut greeting = [0; 3];
            stream.read_exact(&mut greeting).unwrap();
            stream.write_all(&[5, 0]).unwrap();
            let mut request = [0; 5];
            stream.read_exact(&mut request).unwrap();
            let mut tail = vec![0; request[4] as usize + 2];
            stream.read_exact(&mut tail).unwrap();
            stream.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
        });
        let result = probe(port);
        assert!(result.tcp_connected);
        assert_eq!(result.verification_level, "remote_tcp_only");
        worker.join().unwrap();
    }
    #[test]
    fn reject_failed_remote_connect() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut greeting = [0; 3];
            stream.read_exact(&mut greeting).unwrap();
            stream.write_all(&[5, 0]).unwrap();
            let mut request = [0; 5];
            stream.read_exact(&mut request).unwrap();
            let mut tail = vec![0; request[4] as usize + 2];
            stream.read_exact(&mut tail).unwrap();
            stream.write_all(&[5, 5, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
        });
        assert!(!probe(port).tcp_connected);
        worker.join().unwrap();
    }
}
