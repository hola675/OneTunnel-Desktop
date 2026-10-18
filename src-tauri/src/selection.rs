//! F5: ranked candidates from the authorized 1VPN catalog.
//! TCP preflight is a reachability signal, never proof of a working Reality tunnel.
use crate::provider::UserData;
use serde::Serialize;
use std::{net::{TcpStream, ToSocketAddrs}, time::{Duration, Instant}};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCandidate {
    pub id: String,
    pub city: Option<String>,
    pub country: String,
    pub country_code: String,
    pub premium: bool,
    pub host: String,
    pub port: u16,
    pub eligible: bool,
    pub has_reality_sni: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMeasurement {
    pub id: String,
    pub attempts: u8,
    pub successes: u8,
    pub median_ms: Option<u32>,
    pub jitter_ms: Option<u32>,
    pub score: Option<f64>,
    pub status: &'static str,
}

pub fn catalog(account: &UserData) -> Vec<ServerCandidate> {
    let mut out = Vec::new();
    for (location_index, location) in account.locations.iter().enumerate() {
        if let Some(servers) = &location.servers {
            for (server_index, server) in servers.iter().enumerate() {
                let eligible = (!location.is_premium || account.is_premium)
                    && account.to_reality_server(location_index, server_index).is_ok();
                out.push(ServerCandidate {
                    id: format!("{location_index}:{server_index}"),
                    city: location.city.clone(),
                    country: location.country.clone(),
                    country_code: location.country_code.clone(),
                    premium: location.is_premium,
                    host: server.host.clone(),
                    port: 443,
                    eligible,
                    has_reality_sni: server.reality_server_name.as_ref().is_some_and(|s| !s.is_empty()),
                });
            }
        }
    }
    out
}

fn median(values: &mut [u32]) -> u32 {
    values.sort_unstable();
    let length = values.len();
    if length % 2 == 1 { values[length / 2] }
    else { ((values[length / 2 - 1] as u64 + values[length / 2] as u64) / 2) as u32 }
}

/// Measurements are intentionally bounded: 3 attempts x 650 ms maximum.
pub fn probe(candidate: &ServerCandidate) -> ServerMeasurement {
    let mut successes = Vec::new();
    if candidate.eligible {
        for _ in 0..3 {
            let address = format!("{}:{}", candidate.host, candidate.port);
            let result = address.to_socket_addrs()
                .ok().and_then(|addresses| {
                    addresses.filter(|a| !a.ip().is_loopback() && !a.ip().is_unspecified() && !a.ip().is_private())
                        .find_map(|endpoint| {
                            let start = Instant::now();
                            TcpStream::connect_timeout(&endpoint, Duration::from_millis(650))
                                .ok().map(|_| start.elapsed().as_millis().min(u32::MAX as u128) as u32)
                        })
                });
            if let Some(ms) = result { successes.push(ms.max(1)); }
        }
    }
    let success_count = successes.len() as u8;
    if success_count == 0 {
        return ServerMeasurement { id: candidate.id.clone(), attempts: 3, successes: 0,
            median_ms: None, jitter_ms: None, score: None, status: "unreachable" };
    }
    let median_ms = median(&mut successes);
    let jitter_ms = successes.iter().map(|value| value.abs_diff(median_ms) as u64).sum::<u64>() / successes.len() as u64;
    // Balanced 0-100 ranking: reachability dominates; latency + jitter break ties.
    let reliability = success_count as f64 / 3.0;
    let latency = 1.0 / (1.0 + median_ms as f64 / 150.0);
    let stability = 1.0 / (1.0 + jitter_ms as f64 / 90.0);
    let score = 100.0 * (0.65 * reliability + 0.25 * latency + 0.10 * stability);
    ServerMeasurement { id: candidate.id.clone(), attempts: 3, successes: success_count,
        median_ms: Some(median_ms), jitter_ms: Some(jitter_ms as u32),
        score: Some((score * 10.0).round() / 10.0), status: "tcp_preflight_only" }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn median_of_even_values_is_correct() {
        assert_eq!(median(&mut [120, 10, 50, 80]), 65);
    }
    #[test] fn ineligible_candidate_does_not_provoke_network_probe() {
        let candidate = ServerCandidate { id: "0:0".into(), city: None, country: "Test".into(),
            country_code: "ZZ".into(), premium: true, host: "localhost".into(), port: 443,
            eligible: false, has_reality_sni: false };
        let result = probe(&candidate);
        assert!(result.score.is_none());
        assert_eq!(result.successes, 0);
    }
}
