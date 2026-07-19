//! SSRF 防护：URL 校验与内网地址拦截。
//! 从 fetcher.rs 拆出，供 fetcher / discovery / extract 等所有发起外网请求的入口统一调用。

use reqwest::Url;
use std::net::{Ipv4Addr, Ipv6Addr};

/// SSRF 防护：仅允许 http/https，拒绝 localhost 与私有/内网地址。
/// 所有发起外网请求的入口（fetcher / discovery / extract）统一走这里校验。
pub(crate) fn validate_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("Unsupported URL scheme: {}", scheme)),
    }
    if let Some(host) = parsed.host_str() {
        if is_blocked_host(host) || is_blocked_ip(host) {
            return Err("Private/internal addresses are not allowed".to_string());
        }
    }
    Ok(parsed)
}

/// 判断 IPv4 是否属于私有/内网段（含 loopback、link-local、unspecified）。
fn is_private_v4(octets: [u8; 4]) -> bool {
    // 127.0.0.0/8 整段都是 loopback，不能只拦 127.0.0.1
    octets[0] == 127
        // 10.0.0.0/8
        || octets[0] == 10
        // 192.168.0.0/16
        || (octets[0] == 192 && octets[1] == 168)
        // 172.16.0.0/12 才是私有段：仅第二段在 16-31 范围内时拦截，
        // 其余 172.x 是公网地址，不能一刀切
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        // 169.254.0.0/16 link-local（云厂商 metadata 接口常用此段）
        || (octets[0] == 169 && octets[1] == 254)
        // 0.0.0.0
        || octets == [0, 0, 0, 0]
}

/// 第一层：字符串前缀快速过滤常见内网 host，省去解析开销。
/// 只拦截一眼能确认的形式；其余（十进制 IP、IPv6 literal 等）交给 is_blocked_ip 兜底。
fn is_blocked_host(host: &str) -> bool {
    host == "localhost"
        // 127.0.0.0/8 整段都是 loopback，不能只拦 127.0.0.1
        || host.starts_with("127.")
        || host == "0.0.0.0"
        || host == "[::1]"
        || host == "::1"
        || host.starts_with("169.254.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        // 172.16.0.0/12 才是私有段：仅第二段在 16-31 范围内时拦截，
        // 其余 172.x 是公网地址，不能一刀切
        || (host.starts_with("172.")
            && host
                .split(".")
                .nth(1)
                .and_then(|s| s.parse::<u8>().ok())
                .is_some_and(|n| n >= 16 && n <= 31))
}

/// 第二层：字符串匹配之后，再用标准库 IP 解析兜底，
/// 防止十进制 IP（如 2130706433）、八进制（0177.0.0.1）、十六进制等形式绕过。
/// 注：Url::parse 已按 WHATWG 规范把这类 host 归一化为点分十进制，
/// 这里做解析校验是防御性的双保险。
/// 带方括号的 IPv6 提取内部地址解析，只拦 loopback / unspecified /
/// IPv4-mapped 私有地址（如 ::ffff:127.0.0.1），公网 IPv6 正常放行。
fn is_blocked_ip(host: &str) -> bool {
    if let Ok(ipv4) = host.parse::<Ipv4Addr>() {
        return is_private_v4(ipv4.octets());
    }
    if let Ok(ipv6) = host
        .trim_matches(|c| c == '[' || c == ']')
        .parse::<Ipv6Addr>()
    {
        if ipv6.is_loopback() || ipv6.is_unspecified() {
            return true;
        }
        // IPv4-mapped IPv6 按映射后的 IPv4 地址判定
        if let Some(v4) = ipv6.to_ipv4_mapped() {
            return is_private_v4(v4.octets());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_blocks_internal_addresses() {
        // 127.0.0.0/8 整段，不只是 127.0.0.1
        assert!(validate_url("http://127.0.0.1/feed").is_err());
        assert!(validate_url("http://127.0.0.2/feed").is_err());
        assert!(validate_url("http://127.1/feed").is_err());
        // 链路本地与私有段
        assert!(validate_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(validate_url("http://10.0.0.1/feed").is_err());
        assert!(validate_url("http://192.168.1.1/feed").is_err());
        assert!(validate_url("http://172.16.0.1/feed").is_err());
        // 172.32+ 是公网，不应误伤
        assert!(validate_url("http://172.32.0.1/feed").is_ok());
        // IPv6 loopback 与 IPv4-mapped IPv6
        assert!(validate_url("http://[::1]/feed").is_err());
        assert!(validate_url("http://[::ffff:127.0.0.1]/feed").is_err());
        // IPv6 unspecified
        assert!(validate_url("http://[::]/feed").is_err());
        // 公网 IPv6 放行，不能一刀切拦截所有带方括号的 IPv6 literal
        assert!(validate_url("http://[2606:4700:4700::1111]/feed").is_ok());
        // 十进制 / 八进制 / 十六进制形式的 IP（Url::parse 会归一化为 127.0.0.1）
        assert!(validate_url("http://2130706433/feed").is_err());
        assert!(validate_url("http://0177.0.0.1/feed").is_err());
        assert!(validate_url("http://0x7f.0.0.1/feed").is_err());
        assert!(validate_url("http://0x7f000001/feed").is_err());
        // 非 http/https scheme
        assert!(validate_url("file:///etc/passwd").is_err());
        // 正常公网地址放行
        assert!(validate_url("https://example.com/feed.xml").is_ok());
    }
}
