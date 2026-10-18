//! SHA-256 verification before executing downloaded runtimes.
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Expected Xray SHA-256 is missing or malformed".into());
    }
    let mut input = File::open(path).map_err(|e| format!("Cannot open Xray runtime: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = input.read(&mut buffer).map_err(|e| format!("Cannot hash Xray runtime: {e}"))?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected) {
        return Err("Xray executable SHA-256 does not match the pinned value".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_file_is_rejected() {
        assert!(verify_sha256(Path::new("does-not-exist-xray.exe"), &"a".repeat(64)).is_err());
    }
    #[test]
    fn invalid_digest_is_rejected() {
        assert!(verify_sha256(Path::new("xray.exe"), "none").is_err());
    }
    #[test]
    fn incorrect_digest_is_rejected() {
        let temp = std::env::temp_dir().join(format!("onetunnel-sha-mismatch-{}", std::process::id()));
        std::fs::write(&temp, b"abc").unwrap();
        let outcome = verify_sha256(&temp, &"0".repeat(64));
        std::fs::remove_file(&temp).unwrap();
        assert!(outcome.is_err());
    }
    #[test]
    fn known_digest_is_accepted() {
        let temp = std::env::temp_dir().join(format!("onetunnel-sha-test-{}", std::process::id()));
        std::fs::write(&temp, b"abc").unwrap();
        let outcome = verify_sha256(&temp, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        std::fs::remove_file(&temp).unwrap();
        assert!(outcome.is_ok());
    }
}
