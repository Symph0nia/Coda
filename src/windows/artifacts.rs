use crate::cli::Category;
use crate::common::Context;
use std::path::{Path, PathBuf};

pub fn clean_filesystem_artifacts(ctx: &Context, categories: &[Category]) {
    let do_system = categories.contains(&Category::System);
    let do_shell = categories.contains(&Category::Shell) || do_system;
    let do_security = categories.contains(&Category::Security) || do_system;
    let do_network = categories.contains(&Category::Network) || do_system;
    let do_browser = categories.contains(&Category::Browser);

    if do_system {
        clean_prefetch(ctx);
        clean_usn_journal(ctx);
        clean_srum(ctx);
        if ctx.aggressive {
            clean_vss(ctx);
            clean_recycle_bin(ctx);
        } else {
            ctx.info("跳过 VSS/回收站 (需要 --aggressive)");
        }
    }

    if do_shell {
        for user_dir in user_profile_dirs() {
            clean_powershell_logs(ctx, &user_dir);
            clean_jump_lists(ctx, &user_dir);
        }
    }

    if do_browser || do_shell {
        for user_dir in user_profile_dirs() {
            clean_thumbcache(ctx, &user_dir);
        }
    }

    if do_network {
        clean_dns_cache(ctx);
    }

    if do_security {
        for user_dir in user_profile_dirs() {
            clean_defender(ctx, &user_dir);
        }
    }
}

fn user_profile_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let users = Path::new(r"C:\Users");
    if let Ok(entries) = std::fs::read_dir(users) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.eq_ignore_ascii_case("Public")
                || name.eq_ignore_ascii_case("Default")
                || name.eq_ignore_ascii_case("Default User")
                || name.eq_ignore_ascii_case("All Users")
            {
                continue;
            }
            if entry.path().is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    if dirs.is_empty() {
        if let Ok(p) = std::env::var("USERPROFILE") {
            dirs.push(PathBuf::from(p));
        }
    }
    dirs
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

    let superfetch = Path::new(r"C:\Windows\Prefetch\ReadyBoot");
    if superfetch.exists() {
        ctx.remove(superfetch);
    }
}

fn clean_usn_journal(ctx: &Context) {
    ctx.info("清理 USN Journal...");
    ctx.run_cmd(
        "删除 C: USN Journal",
        "fsutil",
        &["usn", "deletejournal", "/D", "C:"],
    );
}

fn clean_srum(ctx: &Context) {
    ctx.info("清理 SRUM 数据库...");
    ctx.run_cmd("停止 DPS 服务", "net", &["stop", "DPS"]);
    let srum = Path::new(r"C:\Windows\System32\sru\SRUDB.dat");
    if srum.exists() {
        ctx.remove(srum);
    }
    ctx.run_cmd("启动 DPS 服务", "net", &["start", "DPS"]);
}

fn clean_powershell_logs(ctx: &Context, user_dir: &Path) {
    ctx.info(&format!("清理 PowerShell 日志 ({})...", user_dir.display()));

    ctx.run_cmd(
        "清除 PowerShell EventLog",
        "wevtutil",
        &["cl", "Microsoft-Windows-PowerShell/Operational"],
    );

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

    let ps_history = user_dir
        .join(r"AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt");
    if ps_history.exists() {
        ctx.remove(&ps_history);
    }
}

fn clean_thumbcache(ctx: &Context, user_dir: &Path) {
    ctx.info(&format!("清理 Thumbcache ({})...", user_dir.display()));
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
    ctx.info(&format!("清理 Jump Lists ({})...", user_dir.display()));
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
    ctx.info("清理回收站 (aggressive)...");
    ctx.run_cmd("清空回收站", "cmd", &["/C", r"rd /s /q C:\$Recycle.Bin"]);
}

fn clean_vss(ctx: &Context) {
    ctx.info("删除卷影副本 (aggressive)...");
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

fn clean_defender(ctx: &Context, _user_dir: &Path) {
    ctx.info("清理 Windows Defender 痕迹...");

    let quarantine = Path::new(r"C:\ProgramData\Microsoft\Windows Defender\Quarantine");
    if quarantine.exists() {
        ctx.remove(quarantine);
    }

    let scan_history = Path::new(r"C:\ProgramData\Microsoft\Windows Defender\Scans\History");
    if scan_history.exists() {
        ctx.remove(scan_history);
    }

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
}
