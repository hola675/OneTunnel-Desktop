//! SOCKS5 loopback handshake and genuine HTTPS egress verification.
//! Successful handshake alone does not prove a VPN connection.
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    process::{Command, Stdio},
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

fn validate_egress_ip(reply: &[u8]) -> Result<String, String> {
    if reply.len() > 128 {
        return Err("HTTPS verification response is too large".into());
    }
    let raw = std::str::from_utf8(reply).map_err(|_| "Invalid HTTPS verification response")?;
    let parsed: IpAddr = raw
        .trim()
        .parse()
        .map_err(|_| "HTTPS endpoint did not return an IP address")?;
    if parsed.is_loopback() || parsed.is_unspecified() || parsed.is_multicast() {
        return Err("HTTPS endpoint returned an invalid egress IP address".into());
    }
    Ok(parsed.to_string())
}

/// curl.exe is bundled in supported modern Windows builds. Using an absolute
/// System32 path avoids PATH hijacking. TLS certificates are never bypassed.
/// -q ignores user curl config; socks5h forces remote DNS through Xray.
/// This check reaches api.ipify.org and reveals the tunnel's public egress IP.
pub fn verify_https_egress(port: u16) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let curl = std::path::PathBuf::from(
        std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
    )
    .join("System32")
    .join("curl.exe");

    #[cfg(not(target_os = "windows"))]
    let curl = std::path::PathBuf::from("/usr/bin/curl");

    if !curl.is_file() {
        return Err("Windows curl.exe is unavailable for HTTPS tunnel verification".into());
    }
    let endpoint = format!("socks5h://127.0.0.1:{port}");
    let output = Command::new(curl)
        .args([
            "-q",
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "12",
            "--connect-timeout",
            "6",
            "--max-filesize",
            "128",
            "--noproxy",
            "",
            "--proxy",
            &endpoint,
            "--proto",
            "=https",
            "https://api.ipify.org",
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|_| "Cannot execute HTTPS verification")?;
    if !output.status.success() {
        return Err("HTTPS request through SOCKS5 failed or timed out".into());
    }
    validate_egress_ip(&output.stdout)
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

    #[test]
    fn only_ip_response_counts_as_verified_egress() {
        assert_eq!(validate_egress_ip(b"8.8.8.8\n").unwrap(), "8.8.8.8");
        assert!(validate_egress_ip(b"<html>success</html>").is_err());
        assert!(validate_egress_ip(b"127.0.0.1").is_err());
        assert!(validate_egress_ip(&vec![b'x'; 129]).is_err());
    }
}
