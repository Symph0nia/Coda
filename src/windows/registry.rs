use crate::cli::Category;
use crate::common::Context;
use std::path::{Path, PathBuf};

pub fn clean_registry_traces(ctx: &Context, categories: &[Category]) {
    ctx.info("清理注册表取证痕迹...");

    let do_system = categories.contains(&Category::System);
    let do_shell = categories.contains(&Category::Shell) || categories.contains(&Category::System);
    let do_network =
        categories.contains(&Category::Network) || categories.contains(&Category::System);

    if do_system {
        clean_amcache(ctx);
        clean_shimcache(ctx);
        clean_bam(ctx);
    }

    // HKLM 侧与当前 HKCU
    if do_shell {
        clean_user_hive(ctx, "HKCU", "当前用户");
    }
    if do_network {
        clean_rdp_hive(ctx, "HKCU");
    }

    // 其他用户：load NTUSER.DAT → 清理 → unload
    if do_shell || do_network {
        clean_other_user_hives(ctx, do_shell, do_network);
    }

    // 文件侧 RDP
    if do_network {
        clean_rdp_files(ctx);
    }
}

fn clean_other_user_hives(ctx: &Context, do_shell: bool, do_network: bool) {
    let current = std::env::var("USERNAME").unwrap_or_default();
    let users_dir = Path::new(r"C:\Users");
    let Ok(entries) = std::fs::read_dir(users_dir) else {
        return;
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if is_skipped_profile(&name) || name.eq_ignore_ascii_case(&current) {
            continue;
        }

        let ntuser = entry.path().join("NTUSER.DAT");
        if !ntuser.exists() {
            continue;
        }

        // 合法的注册表子键名（仅字母数字下划线）
        let safe: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        let hive = format!("CODA_{safe}");
        let hive_root = format!(r"HKU\{hive}");
        let ntuser_str = ntuser.display().to_string();

        ctx.info(&format!("加载用户配置单元: {name}"));
        if ctx.dry_run {
            println!(
                "[预览] reg load {} {}",
                hive_root, ntuser_str
            );
            ctx.record_preview();
            if do_shell {
                clean_user_hive(ctx, &hive_root, &name);
            }
            if do_network {
                clean_rdp_hive(ctx, &hive_root);
            }
            println!("[预览] reg unload {hive_root}");
            ctx.record_preview();
            continue;
        }

        let load = std::process::Command::new("reg")
            .args(["load", &hive_root, &ntuser_str])
            .output();

        match load {
            Ok(o) if o.status.success() => {
                ctx.record_ok();
                println!("[执行] 加载 {hive_root}");
                if do_shell {
                    clean_user_hive(ctx, &hive_root, &name);
                }
                if do_network {
                    clean_rdp_hive(ctx, &hive_root);
                }
                ctx.run_cmd(
                    &format!("卸载 {hive_root}"),
                    "reg",
                    &["unload", &hive_root],
                );
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                // 用户已登录时 NTUSER.DAT 被锁，跳过
                eprintln!(
                    "[跳过] 无法加载 {} 的 NTUSER.DAT — {}",
                    name,
                    stderr.trim()
                );
                ctx.record_skip();
            }
            Err(e) => {
                eprintln!("[跳过] reg load 失败 — {e}");
                ctx.record_skip();
            }
        }
    }
}

fn is_skipped_profile(name: &str) -> bool {
    name.eq_ignore_ascii_case("Public")
        || name.eq_ignore_ascii_case("Default")
        || name.eq_ignore_ascii_case("Default User")
        || name.eq_ignore_ascii_case("All Users")
}

fn clean_user_hive(ctx: &Context, root: &str, label: &str) {
    ctx.info(&format!("清理用户注册表痕迹 ({label} @ {root})..."));
    clean_userassist(ctx, root);
    clean_shellbags(ctx, root);
    clean_mru(ctx, root);
    clean_muicache(ctx, root);
}

fn clean_amcache(ctx: &Context) {
    let amcache = Path::new(r"C:\Windows\appcompat\Programs\Amcache.hve");
    if amcache.exists() {
        ctx.run_cmd("停止 AppIDSvc", "net", &["stop", "AppIDSvc"]);
        ctx.remove(amcache);
    }

    let recent_file_cache = Path::new(r"C:\Windows\AppCompat\Programs\RecentFileCache.bcf");
    if recent_file_cache.exists() {
        ctx.remove(recent_file_cache);
    }
}

fn clean_shimcache(ctx: &Context) {
    ctx.run_cmd(
        "清除 ShimCache",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatCache",
            "/v",
            "AppCompatCache",
            "/f",
        ],
    );
}

fn clean_bam(ctx: &Context) {
    ctx.run_cmd(
        "清除 BAM",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Services\bam\State\UserSettings",
            "/f",
        ],
    );
    ctx.run_cmd(
        "清除 DAM",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Services\dam\State\UserSettings",
            "/f",
        ],
    );
}

fn clean_userassist(ctx: &Context, root: &str) {
    let guids = [
        "{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}",
        "{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}",
    ];
    for guid in &guids {
        let key = format!(
            r"{root}\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{guid}\Count"
        );
        ctx.run_cmd(
            &format!("清除 UserAssist {guid}"),
            "reg",
            &["delete", &key, "/f"],
        );
    }
}

fn clean_shellbags(ctx: &Context, root: &str) {
    let suffixes = [
        r"SOFTWARE\Microsoft\Windows\Shell\BagMRU",
        r"SOFTWARE\Microsoft\Windows\Shell\Bags",
        r"SOFTWARE\Microsoft\Windows\ShellNoRoam\BagMRU",
        r"SOFTWARE\Microsoft\Windows\ShellNoRoam\Bags",
    ];
    for s in &suffixes {
        let key = format!(r"{root}\{s}");
        ctx.run_cmd(
            &format!("清除 ShellBags: {key}"),
            "reg",
            &["delete", &key, "/f"],
        );
    }
}

fn clean_mru(ctx: &Context, root: &str) {
    let suffixes = [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\OpenSavePidlMRU",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\LastVisitedPidlMRU",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs",
    ];
    for s in &suffixes {
        let key = format!(r"{root}\{s}");
        ctx.run_cmd(
            &format!("清除 MRU: {key}"),
            "reg",
            &["delete", &key, "/f"],
        );
    }
}

fn clean_muicache(ctx: &Context, root: &str) {
    let key = format!(
        r"{root}\SOFTWARE\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache"
    );
    ctx.run_cmd("清除 MUI Cache", "reg", &["delete", &key, "/f"]);
}

fn clean_rdp_hive(ctx: &Context, root: &str) {
    let key = format!(r"{root}\SOFTWARE\Microsoft\Terminal Server Client");
    ctx.run_cmd(
        &format!("清除 RDP 连接历史 ({root})"),
        "reg",
        &["delete", &key, "/f"],
    );
}

fn clean_rdp_files(ctx: &Context) {
    let users = Path::new(r"C:\Users");
    if let Ok(entries) = std::fs::read_dir(users) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let rdp = entry.path().join("Documents").join("Default.rdp");
            if rdp.exists() {
                ctx.remove(&rdp);
            }
        }
    }
}

#[allow(dead_code)]
fn user_profile_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(r"C:\Users") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if is_skipped_profile(&name) {
                continue;
            }
            if entry.path().is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    dirs
}
