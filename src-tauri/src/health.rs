//! Minimal SOCKS5 handshake probe through the loopback interface.
//! Successful handshake proves the listener responds, NOT VPN internet egress.
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

pub fn socks5_ready(port: u16) -> Result<(), String> {
    let target = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let timeout = Duration::from_millis(750);
    let mut stream = TcpStream::connect_timeout(&target, timeout)
        .map_err(|e| format!("SOCKS connection unavailable: {e}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    stream
        .write_all(&[0x05, 0x01, 0x00])
        .map_err(|e| e.to_string())?;
    let mut reply = [0u8; 2];
    stream
        .read_exact(&mut reply)
        .map_err(|e| format!("SOCKS handshake failed: {e}"))?;
    if reply != [0x05, 0x00] {
        return Err("SOCKS listener returned an invalid authentication response".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};

    #[test]
    fn fake_socks_server_answers_handshake() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut greeting = [0; 3];
            socket.read_exact(&mut greeting).unwrap();
            assert_eq!(greeting, [5, 1, 0]);
            socket.write_all(&[5, 0]).unwrap();
        });
        assert!(socks5_ready(port).is_ok());
        worker.join().unwrap();
    }

    #[test]
    fn absent_listener_is_rejected() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        assert!(socks5_ready(port).is_err());
    }

    #[test]
    fn malformed_response_is_rejected() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let worker = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut greeting = [0; 3];
            socket.read_exact(&mut greeting).unwrap();
            socket.write_all(&[5, 255]).unwrap();
        });
        assert!(socks5_ready(port).is_err());
        worker.join().unwrap();
    }
}
