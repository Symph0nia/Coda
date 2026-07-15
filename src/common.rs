use std::fs;
use std::io;
use std::path::Path;

pub struct Context {
    pub dry_run: bool,
    pub shred: bool,
    pub shred_passes: u32,
    pub truncate_mode: bool,
    #[allow(dead_code)]
    pub timestomp: bool,
}

impl Context {
    pub fn remove(&self, path: &Path) {
        if self.dry_run {
            println!("[预览] 将删除: {}", path.display());
            return;
        }

        if self.shred && path.is_file() {
            if let Err(e) = crate::shred::shred_path(path, self.shred_passes) {
                eprintln!("[失败] 覆写 {} — {}", path.display(), e);
                return;
            }
            println!("[覆写删除] {}", path.display());
            return;
        }

        if self.truncate_mode && path.is_file() {
            match truncate_file(path) {
                Ok(()) => println!("[截断] {}", path.display()),
                Err(e) => eprintln!("[失败] 截断 {} — {}", path.display(), e),
            }
            return;
        }

        let result = if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        match result {
            Ok(()) => println!("[删除] {}", path.display()),
            Err(e) => eprintln!("[失败] {} — {}", path.display(), e),
        }
    }

    pub fn run_cmd(&self, label: &str, cmd: &str, args: &[&str]) {
        if self.dry_run {
            println!("[预览] {}: {} {}", label, cmd, args.join(" "));
            return;
        }
        match std::process::Command::new(cmd).args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    println!("[执行] {}", label);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("[失败] {} — {}", label, stderr.trim());
                }
            }
            Err(e) => eprintln!("[跳过] {} — {}", label, e),
        }
    }

    pub fn info(&self, msg: &str) {
        println!("[信息] {}", msg);
    }
}

fn truncate_file(path: &Path) -> io::Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    Ok(())
}

#[cfg(unix)]
#[allow(dead_code)]
pub fn timestomp(target: &Path, reference: &Path) {
    let _ = std::process::Command::new("touch")
        .arg("-r")
        .arg(reference)
        .arg(target)
        .output();
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn timestomp(_target: &Path, _reference: &Path) {
    // Windows timestomping 需要 SetFileTime API，暂用 PowerShell
    // 实际场景中推荐编译时链接 windows-sys
}

pub fn self_destruct() {
    if let Ok(exe) = std::env::current_exe() {
        #[cfg(unix)]
        {
            let _ = crate::shred::shred_path(&exe, 1);
        }
        let _ = fs::remove_file(&exe);
        println!("[自毁] 已删除 {}", exe.display());
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
