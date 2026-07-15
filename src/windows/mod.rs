pub mod eventlog;
pub mod registry;
pub mod artifacts;

use crate::cli::Category;
use crate::common::Context;
use crate::paths;
use std::path::{Path, PathBuf};

pub fn clean_all(ctx: &Context, categories: &[Category]) {
    // 文件目标清理
    let targets = collect_targets(categories);
    if targets.is_empty() {
        ctx.info("没有找到需要清理的目标文件");
    } else {
        ctx.info(&format!("发现 {} 个文件/目录目标", targets.len()));
        for t in &targets {
            ctx.remove(t);
        }
    }

    // Event Log 批量清除
    if categories.contains(&Category::System) || categories.contains(&Category::Audit) {
        eventlog::clean_all_event_logs(ctx);
    }

    // 禁用安全日志通道
    if categories.contains(&Category::Security) {
        eventlog::disable_event_logging(ctx);
    }

    // 注册表痕迹
    registry::clean_registry_traces(ctx);

    // 文件系统取证痕迹
    artifacts::clean_filesystem_artifacts(ctx);
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
