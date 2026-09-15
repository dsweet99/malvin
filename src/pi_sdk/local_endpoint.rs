use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

const KEYLESS_LOCAL_REACHABILITY_TIMEOUT: Duration = Duration::from_millis(150);

fn scheme_and_rest(base_url: &str) -> Option<(&'static str, &str)> {
    let trimmed = base_url.trim();
    if let Some(rest) = trimmed.strip_prefix("https://") {
        return Some(("https", rest));
    }
    let rest = trimmed.strip_prefix("http://")?;
    Some(("http", rest))
}

fn default_port_for_scheme(scheme: &str) -> u16 {
    if scheme == "https" { 443 } else { 80 }
}

fn parse_bracketed_host_port(authority: &str, default_port: u16) -> Option<(String, u16)> {
    let rest = authority.strip_prefix('[')?;
    let (host, after) = rest.split_once(']')?;
    if after.is_empty() {
        return Some((host.to_string(), default_port));
    }
    let port = after.strip_prefix(':')?.parse().ok()?;
    Some((host.to_string(), port))
}

fn parse_host_port_pair(authority: &str) -> Option<(String, u16)> {
    let (host, port_str) = authority.rsplit_once(':')?;
    if host.is_empty() || !port_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let port = port_str.parse().ok()?;
    Some((host.to_string(), port))
}

pub(crate) fn parse_http_authority_host_port(base_url: &str) -> Option<(String, u16)> {
    let (scheme, rest) = scheme_and_rest(base_url)?;
    let authority = rest.split('/').next()?.trim();
    if authority.is_empty() {
        return None;
    }
    let default_port = default_port_for_scheme(scheme);
    if let Some(parsed) = parse_bracketed_host_port(authority, default_port) {
        return Some(parsed);
    }
    if let Some(parsed) = parse_host_port_pair(authority) {
        return Some(parsed);
    }
    Some((authority.to_string(), default_port))
}

pub(crate) fn http_base_url_is_listening(base_url: &str) -> bool {
    let Some((host, port)) = parse_http_authority_host_port(base_url) else {
        return false;
    };
    let Ok(addrs) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    addrs.into_iter().any(|addr: SocketAddr| {
        TcpStream::connect_timeout(&addr, KEYLESS_LOCAL_REACHABILITY_TIMEOUT).is_ok()
    })
}

pub(crate) fn keyless_local_provider_is_listening(provider: &str) -> bool {
    let Some(defaults) = pi::provider_metadata::provider_routing_defaults(provider) else {
        return false;
    };
    http_base_url_is_listening(defaults.base_url)
}
