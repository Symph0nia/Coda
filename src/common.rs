use std::cell::Cell;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Context {
    pub dry_run: bool,
    pub shred: bool,
    pub shred_passes: u32,
    pub truncate_mode: bool,
    pub timestomp: bool,
    pub aggressive: bool,
    ok: Cell<u32>,
    fail: Cell<u32>,
    skip: Cell<u32>,
    preview: Cell<u32>,
}

impl Context {
    pub fn new(
        dry_run: bool,
        shred: bool,
        shred_passes: u32,
        truncate_mode: bool,
        timestomp: bool,
        aggressive: bool,
    ) -> Self {
        Self {
            dry_run,
            shred,
            shred_passes,
            truncate_mode,
            timestomp,
            aggressive,
            ok: Cell::new(0),
            fail: Cell::new(0),
            skip: Cell::new(0),
            preview: Cell::new(0),
        }
    }

    pub fn record_ok(&self) {
        self.ok.set(self.ok.get() + 1);
    }

    pub fn record_fail(&self) {
        self.fail.set(self.fail.get() + 1);
    }

    pub fn record_skip(&self) {
        self.skip.set(self.skip.get() + 1);
    }

    pub fn record_preview(&self) {
        self.preview.set(self.preview.get() + 1);
    }

    pub fn fail_count(&self) -> u32 {
        self.fail.get()
    }

    pub fn remove(&self, path: &Path) {
        if is_special_node(path) {
            println!("[跳过] 特殊文件: {}", path.display());
            self.record_skip();
            return;
        }

        if self.dry_run {
            println!("[预览] 将删除: {}", path.display());
            self.record_preview();
            return;
        }

        if self.shred {
            match crate::shred::shred_path(path, self.shred_passes) {
                Ok(()) => {
                    println!("[覆写删除] {}", path.display());
                    self.record_ok();
                }
                Err(e) => {
                    eprintln!("[失败] 覆写 {} — {}", path.display(), e);
                    self.record_fail();
                }
            }
            return;
        }

        if self.truncate_mode && path.is_file() {
            let times = if self.timestomp {
                save_times(path)
            } else {
                None
            };
            match truncate_file(path) {
                Ok(()) => {
                    if let Some(ref t) = times {
                        restore_times(path, t);
                    }
                    println!("[截断] {}", path.display());
                    self.record_ok();
                }
                Err(e) => {
                    eprintln!("[失败] 截断 {} — {}", path.display(), e);
                    self.record_fail();
                }
            }
            return;
        }

        let result = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        match result {
            Ok(()) => {
                println!("[删除] {}", path.display());
                self.record_ok();
            }
            Err(e) => {
                eprintln!("[失败] {} — {}", path.display(), e);
                self.record_fail();
            }
        }
    }

    pub fn run_cmd(&self, label: &str, cmd: &str, args: &[&str]) {
        if self.dry_run {
            println!("[预览] {}: {} {}", label, cmd, args.join(" "));
            self.record_preview();
            return;
        }
        match std::process::Command::new(cmd).args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    println!("[执行] {}", label);
                    self.record_ok();
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("[失败] {} — {}", label, stderr.trim());
                    self.record_fail();
                }
            }
            Err(e) => {
                eprintln!("[跳过] {} — {}", label, e);
                self.record_skip();
            }
        }
    }

    /// 依次尝试命令，成功一个即停。
    #[allow(dead_code)] // Windows 目标同样编译，主要在 Linux 路径使用
    pub fn run_cmd_fallback(&self, label: &str, attempts: &[(&str, &[&str])]) {
        if self.dry_run {
            if let Some((cmd, args)) = attempts.first() {
                println!("[预览] {}: {} {}", label, cmd, args.join(" "));
                self.record_preview();
            }
            return;
        }

        for (cmd, args) in attempts {
            match std::process::Command::new(cmd).args(*args).output() {
                Ok(output) if output.status.success() => {
                    println!("[执行] {} ({})", label, cmd);
                    self.record_ok();
                    return;
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
        eprintln!("[跳过] {} — 无可用命令", label);
        self.record_skip();
    }

    pub fn info(&self, msg: &str) {
        println!("[信息] {}", msg);
    }

    pub fn print_summary(&self) {
        let (ok, fail, skip, preview) = (
            self.ok.get(),
            self.fail.get(),
            self.skip.get(),
            self.preview.get(),
        );
        if self.dry_run {
            println!();
            println!(
                "[汇总] 预览 {} 项 (成功 {} / 失败 {} / 跳过 {})",
                preview, ok, fail, skip
            );
        } else {
            println!();
            println!("[汇总] 成功 {} / 失败 {} / 跳过 {}", ok, fail, skip);
        }
    }
}

/// 展开路径模式，过滤不存在、空节点与特殊文件。
pub fn expand_targets(patterns: &[String]) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    for p in patterns {
        if p.contains('*') || p.contains('?') || p.contains('[') {
            if let Ok(entries) = glob::glob(p) {
                targets.extend(entries.filter_map(Result::ok));
            }
        } else {
            targets.push(PathBuf::from(p));
        }
    }
    targets
        .into_iter()
        .filter(|p| p.exists())
        .filter(|p| !is_special_node(p))
        .filter(|p| !is_empty(p))
        .collect()
}

pub fn is_empty(path: &Path) -> bool {
    if path.is_dir() {
        std::fs::read_dir(path)
            .map(|mut d| d.next().is_none())
            .unwrap_or(true)
    } else {
        std::fs::metadata(path)
            .map(|m| m.len() == 0)
            .unwrap_or(true)
    }
}

/// socket / fifo / 设备节点等，不应当普通文件删除或覆写。
pub fn is_special_node(path: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    let ft = meta.file_type();
    if ft.is_dir() || ft.is_file() || ft.is_symlink() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        return ft.is_socket() || ft.is_fifo() || ft.is_block_device() || ft.is_char_device();
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// 备份目标文件名：完整路径编码，避免同名冲突。
pub fn backup_dest_name(original: &Path) -> String {
    original
        .to_string_lossy()
        .replace(['/', '\\', ':'], "_")
}

fn truncate_file(path: &Path) -> io::Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    Ok(())
}

struct FileTimes {
    #[cfg(unix)]
    atime: libc::timespec,
    #[cfg(unix)]
    mtime: libc::timespec,
    #[cfg(windows)]
    accessed: std::time::SystemTime,
    #[cfg(windows)]
    modified: std::time::SystemTime,
}

fn save_times(path: &Path) -> Option<FileTimes> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = fs::metadata(path).ok()?;
        let atime = libc::timespec {
            tv_sec: meta.atime(),
            tv_nsec: meta.atime_nsec(),
        };
        let mtime = libc::timespec {
            tv_sec: meta.mtime(),
            tv_nsec: meta.mtime_nsec(),
        };
        Some(FileTimes { atime, mtime })
    }
    #[cfg(windows)]
    {
        let meta = fs::metadata(path).ok()?;
        Some(FileTimes {
            accessed: meta.accessed().ok()?,
            modified: meta.modified().ok()?,
        })
    }
}

fn restore_times(path: &Path, times: &FileTimes) {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c_path = match CString::new(path.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return,
        };
        let ts = [times.atime, times.mtime];
        unsafe {
            libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), ts.as_ptr(), 0);
        }
    }
    #[cfg(windows)]
    {
        let accessed = system_time_secs(times.accessed);
        let modified = system_time_secs(times.modified);
        let path_str = path.display().to_string().replace('\'', "''");
        let script = format!(
            "$p='{path_str}'; $i=Get-Item -LiteralPath $p -Force; \
             $epoch=[DateTime]'1970-01-01T00:00:00Z'; \
             $i.LastAccessTime=$epoch.AddSeconds({accessed}).ToLocalTime(); \
             $i.LastWriteTime=$epoch.AddSeconds({modified}).ToLocalTime()"
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
    }
}

#[cfg(windows)]
fn system_time_secs(t: std::time::SystemTime) -> u64 {
    t.duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn self_destruct() {
    if let Ok(exe) = std::env::current_exe() {
        #[cfg(unix)]
        {
            let _ = crate::shred::shred_path(&exe, 1);
            match fs::remove_file(&exe) {
                Ok(()) => println!("[自毁] 已删除 {}", exe.display()),
                Err(e) => eprintln!("[自毁] 失败 {} — {}", exe.display(), e),
            }
        }
        #[cfg(windows)]
        {
            let path = exe.display().to_string();
            let cmd = format!(
                "ping 127.0.0.1 -n 2 >nul & del /f /q \"{}\"",
                path.replace('"', "")
            );
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", "/b", "cmd", "/c", &cmd])
                .spawn();
            println!("[自毁] 已安排延迟删除 {}", exe.display());
        }
    }
}

#[cfg(unix)]
pub fn check_privilege() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(windows)]
pub fn check_privilege() -> bool {
    std::process::Command::new("net")
        .arg("session")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn backup_dest_encodes_path() {
        assert_eq!(
            backup_dest_name(Path::new("/var/log/auth.log")),
            "_var_log_auth.log"
        );
        assert_eq!(
            backup_dest_name(Path::new(r"C:\Windows\Temp\a.log")),
            "C__Windows_Temp_a.log"
        );
    }

    #[test]
    fn empty_file_detected() {
        let dir = std::env::temp_dir().join("coda_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("empty.txt");
        fs::write(&f, b"").unwrap();
        assert!(is_empty(&f));
        fs::write(&f, b"x").unwrap();
        assert!(!is_empty(&f));
        let _ = fs::remove_dir_all(&dir);
    }
}
