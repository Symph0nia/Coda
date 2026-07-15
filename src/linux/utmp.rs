use crate::common::Context;
use std::fs;
use std::io::{self, Seek, Write};
use std::path::Path;

// Linux utmp 结构体：固定 384 字节/条目
// 参考 /usr/include/bits/utmp.h
const UT_LINESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;
const UTMP_ENTRY_SIZE: usize = 384;

// lastlog: time_t + ut_line[32] + ut_host[256]
// 32-bit time_t → 292；64-bit time_t → 296（不少 64 位发行版仍用 292）

#[derive(Debug)]
pub struct UtmpEntry {
    pub raw: [u8; UTMP_ENTRY_SIZE],
}

impl UtmpEntry {
    pub fn ut_user(&self) -> String {
        extract_cstr(&self.raw[44..44 + UT_LINESIZE])
    }

    pub fn ut_line(&self) -> String {
        extract_cstr(&self.raw[8..8 + UT_LINESIZE])
    }

    pub fn ut_host(&self) -> String {
        extract_cstr(&self.raw[76..76 + UT_HOSTSIZE])
    }

    pub fn matches(&self, user: Option<&str>, ip: Option<&str>, tty: Option<&str>) -> bool {
        if user.is_none() && ip.is_none() && tty.is_none() {
            return false;
        }
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
        true
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
    ctx: &Context,
    path: &Path,
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
) -> io::Result<u32> {
    let entries = read_utmp_entries(path)?;
    let mut removed = 0u32;

    let remaining: Vec<&UtmpEntry> = entries
        .iter()
        .filter(|e| {
            if e.matches(user, ip, tty) {
                removed += 1;
                if ctx.dry_run {
                    println!(
                        "[预览] 移除 {} 条目: user={} host={} line={}",
                        path.display(),
                        e.ut_user(),
                        e.ut_host(),
                        e.ut_line()
                    );
                    ctx.record_preview();
                }
                false
            } else {
                true
            }
        })
        .collect();

    if !ctx.dry_run && removed > 0 {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;

        lock_exclusive(&file)?;

        for entry in &remaining {
            file.write_all(&entry.raw)?;
        }
        file.flush()?;
        let _ = file.sync_all();
        unlock_file(&file);

        println!(
            "[清除] {} — 移除 {} 条记录，保留 {} 条",
            path.display(),
            removed,
            remaining.len()
        );
        ctx.record_ok();
    }

    Ok(removed)
}

fn lock_exclusive(file: &fs::File) -> io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    let rc = unsafe { libc::flock(fd, libc::LOCK_EX) };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn unlock_file(file: &fs::File) {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    unsafe {
        libc::flock(fd, libc::LOCK_UN);
    }
}

pub struct SavedTimes {
    atime: libc::timespec,
    mtime: libc::timespec,
}

pub fn save_file_times(path: &Path) -> Option<SavedTimes> {
    use std::os::unix::fs::MetadataExt;
    let meta = fs::metadata(path).ok()?;
    Some(SavedTimes {
        atime: libc::timespec {
            tv_sec: meta.atime(),
            tv_nsec: meta.atime_nsec(),
        },
        mtime: libc::timespec {
            tv_sec: meta.mtime(),
            tv_nsec: meta.mtime_nsec(),
        },
    })
}

pub fn restore_file_times(path: &Path, times: &SavedTimes) {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return;
    };
    let ts = [times.atime, times.mtime];
    unsafe {
        libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), ts.as_ptr(), 0);
    }
}

pub fn clean_lastlog(ctx: &Context, user: Option<&str>) -> io::Result<()> {
    let path = Path::new("/var/log/lastlog");
    if !path.exists() {
        return Ok(());
    }

    let username = match user {
        Some(u) => u,
        None => return Ok(()),
    };

    let uid = unsafe {
        let c_name = std::ffi::CString::new(username)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let pw = libc::getpwnam(c_name.as_ptr());
        if pw.is_null() {
            eprintln!("[跳过] 用户 {} 不存在", username);
            ctx.record_skip();
            return Ok(());
        }
        (*pw).pw_uid as u64
    };

    let file_len = fs::metadata(path)?.len();
    let entry_size = detect_lastlog_entry_size(file_len);
    let offset = uid * entry_size as u64;

    if offset + entry_size as u64 > file_len {
        println!(
            "[信息] lastlog — uid={} 超出文件范围 (size={}, entry={})，无需清零",
            uid, file_len, entry_size
        );
        return Ok(());
    }

    if ctx.dry_run {
        println!(
            "[预览] 清零 lastlog 条目: user={} uid={} offset={} size={}",
            username, uid, offset, entry_size
        );
        ctx.record_preview();
        return Ok(());
    }

    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    lock_exclusive(&file)?;
    file.seek(io::SeekFrom::Start(offset))?;
    let zeros = vec![0u8; entry_size];
    file.write_all(&zeros)?;
    file.flush()?;
    unlock_file(&file);
    println!(
        "[清除] lastlog — 已清零 {} (uid={}, entry_size={})",
        username, uid, entry_size
    );
    ctx.record_ok();

    Ok(())
}

/// 按文件长度推断 lastlog 记录大小，避免硬编码 292/296。
pub fn detect_lastlog_entry_size(file_len: u64) -> usize {
    const CANDIDATES: [usize; 2] = [292, 296];
    if file_len == 0 {
        return 292;
    }
    let exact: Vec<usize> = CANDIDATES
        .iter()
        .copied()
        .filter(|s| file_len % *s as u64 == 0)
        .collect();
    match exact.as_slice() {
        [only] => *only,
        [_, _] => 292,
        _ => CANDIDATES
            .iter()
            .copied()
            .min_by_key(|s| file_len % *s as u64)
            .unwrap_or(292),
    }
}

pub fn clean_all_login_records(
    ctx: &Context,
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
) {
    let utmp_files = [
        "/var/run/utmp",
        "/var/log/wtmp",
        "/var/log/btmp",
        "/var/log/wtmp.1",
        "/var/log/btmp.1",
    ];

    for path_str in &utmp_files {
        let path = Path::new(path_str);
        if path.exists() {
            match selective_clean_utmp(ctx, path, user, ip, tty) {
                Ok(n) if n > 0 => {}
                Ok(_) => println!("[信息] {} — 没有匹配的条目", path.display()),
                Err(e) => {
                    eprintln!("[失败] {} — {}", path.display(), e);
                    ctx.record_fail();
                }
            }
        }
    }

    if let Err(e) = clean_lastlog(ctx, user) {
        eprintln!("[失败] lastlog — {}", e);
        ctx.record_fail();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_with(user: &str, host: &str, line: &str) -> UtmpEntry {
        let mut raw = [0u8; UTMP_ENTRY_SIZE];
        let u = user.as_bytes();
        let h = host.as_bytes();
        let l = line.as_bytes();
        raw[44..44 + u.len().min(UT_LINESIZE)].copy_from_slice(&u[..u.len().min(UT_LINESIZE)]);
        raw[76..76 + h.len().min(UT_HOSTSIZE)].copy_from_slice(&h[..h.len().min(UT_HOSTSIZE)]);
        raw[8..8 + l.len().min(UT_LINESIZE)].copy_from_slice(&l[..l.len().min(UT_LINESIZE)]);
        UtmpEntry { raw }
    }

    #[test]
    fn matches_and_filter() {
        let e = entry_with("alice", "10.0.0.1", "pts/1");
        assert!(e.matches(Some("alice"), None, None));
        assert!(e.matches(Some("alice"), Some("10.0.0.1"), Some("pts/1")));
        assert!(!e.matches(Some("bob"), None, None));
        assert!(!e.matches(None, Some("10.0.0.2"), None));
        assert!(!e.matches(None, None, None));
    }

    #[test]
    fn lastlog_size_detect() {
        assert_eq!(detect_lastlog_entry_size(0), 292);
        assert_eq!(detect_lastlog_entry_size(292), 292);
        assert_eq!(detect_lastlog_entry_size(296), 296);
        assert_eq!(detect_lastlog_entry_size(292 * 10), 292);
        assert_eq!(detect_lastlog_entry_size(296 * 5), 296);
    }
}
