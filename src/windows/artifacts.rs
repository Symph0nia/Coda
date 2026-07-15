use crate::common::Context;
use std::path::{Path, PathBuf};

pub fn clean_filesystem_artifacts(ctx: &Context) {
    let user_dir = user_profile_dir();

    clean_prefetch(ctx);
    clean_usn_journal(ctx);
    clean_srum(ctx);
    clean_powershell_logs(ctx, &user_dir);
    clean_thumbcache(ctx, &user_dir);
    clean_jump_lists(ctx, &user_dir);
    clean_recycle_bin(ctx);
    clean_vss(ctx);
    clean_dns_cache(ctx);
    clean_defender(ctx, &user_dir);
}

fn user_profile_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".into()),
    )
}

fn clean_prefetch(ctx: &Context) {
    ctx.info("清理 Prefetch...");
    let prefetch = Path::new(r"C:\Windows\Prefetch");
    if prefetch.exists() {
        if let Ok(entries) = std::fs::read_dir(prefetch) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "pf") {
                    ctx.remove(&p);
                }
            }
        }
    }

    // Superfetch/SysMain
    let superfetch = Path::new(r"C:\Windows\Prefetch\ReadyBoot");
    if superfetch.exists() {
        ctx.remove(superfetch);
    }
}

fn clean_usn_journal(ctx: &Context) {
    ctx.info("清理 USN Journal...");
    ctx.run_cmd("删除 C: USN Journal", "fsutil", &["usn", "deletejournal", "/D", "C:"]);
}

fn clean_srum(ctx: &Context) {
    ctx.info("清理 SRUM 数据库...");
    // 需要先停止服务
    ctx.run_cmd("停止 DPS 服务", "net", &["stop", "DPS"]);
    let srum = Path::new(r"C:\Windows\System32\sru\SRUDB.dat");
    if srum.exists() {
        ctx.remove(srum);
    }
    ctx.run_cmd("启动 DPS 服务", "net", &["start", "DPS"]);
}

fn clean_powershell_logs(ctx: &Context, user_dir: &Path) {
    ctx.info("清理 PowerShell 日志...");

    // ScriptBlock Logging
    ctx.run_cmd(
        "清除 PowerShell EventLog",
        "wevtutil",
        &["cl", "Microsoft-Windows-PowerShell/Operational"],
    );

    // PowerShell Transcription logs
    let transcript_dirs = [
        user_dir.join("Documents").join("PowerShell_transcript"),
        user_dir.join(r"AppData\Local\Temp"),
    ];
    for dir in &transcript_dirs {
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("PowerShell_transcript") {
                        ctx.remove(&entry.path());
                    }
                }
            }
        }
    }

    // ConsoleHost_history.txt
    let ps_history = user_dir
        .join(r"AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt");
    if ps_history.exists() {
        ctx.remove(&ps_history);
    }
}

fn clean_thumbcache(ctx: &Context, user_dir: &Path) {
    ctx.info("清理 Thumbcache / IconCache...");
    let explorer_cache = user_dir.join(r"AppData\Local\Microsoft\Windows\Explorer");
    if explorer_cache.exists() {
        if let Ok(entries) = std::fs::read_dir(&explorer_cache) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("thumbcache_") || name_str.starts_with("iconcache_") {
                    ctx.remove(&entry.path());
                }
            }
        }
    }
}

fn clean_jump_lists(ctx: &Context, user_dir: &Path) {
    ctx.info("清理 Jump Lists...");
    let jump_dirs = [
        user_dir.join(r"AppData\Roaming\Microsoft\Windows\Recent\AutomaticDestinations"),
        user_dir.join(r"AppData\Roaming\Microsoft\Windows\Recent\CustomDestinations"),
    ];
    for dir in &jump_dirs {
        if dir.exists() {
            ctx.remove(dir);
        }
    }
}

fn clean_recycle_bin(ctx: &Context) {
    ctx.info("清理回收站...");
    ctx.run_cmd("清空回收站", "rd", &["/s", "/q", r"C:\$Recycle.Bin"]);
}

fn clean_vss(ctx: &Context) {
    ctx.info("删除卷影副本...");
    ctx.run_cmd(
        "删除所有卷影副本",
        "vssadmin",
        &["delete", "shadows", "/all", "/quiet"],
    );
}

fn clean_dns_cache(ctx: &Context) {
    ctx.info("清空 DNS 缓存...");
    ctx.run_cmd("ipconfig /flushdns", "ipconfig", &["/flushdns"]);
}

fn clean_defender(ctx: &Context, user_dir: &Path) {
    ctx.info("清理 Windows Defender 痕迹...");

    // Defender 隔离文件
    let quarantine = Path::new(r"C:\ProgramData\Microsoft\Windows Defender\Quarantine");
    if quarantine.exists() {
        ctx.remove(quarantine);
    }

    // Defender 扫描历史
    let scan_history = Path::new(r"C:\ProgramData\Microsoft\Windows Defender\Scans\History");
    if scan_history.exists() {
        ctx.remove(scan_history);
    }

    // Defender 日志
    let defender_logs = Path::new(r"C:\ProgramData\Microsoft\Windows Defender\Support");
    if defender_logs.exists() {
        if let Ok(entries) = std::fs::read_dir(defender_logs) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().is_some_and(|e| e == "log") {
                    ctx.remove(&p);
                }
            }
        }
    }

    let _ = user_dir; // Windows Defender logs are system-level
}
