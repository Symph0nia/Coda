pub mod artifacts;
pub mod eventlog;
pub mod registry;

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

    if categories.contains(&Category::System) || categories.contains(&Category::Audit) {
        eventlog::clean_all_event_logs(ctx);
    }

    if categories.contains(&Category::Security) {
        eventlog::disable_event_logging(ctx);
    }

    // 注册表取证痕迹：system / shell / browser 相关
    if categories.iter().any(|c| {
        matches!(
            c,
            Category::System | Category::Shell | Category::Browser | Category::Network
        )
    }) {
        registry::clean_registry_traces(ctx, categories);
    }

    // 文件系统取证痕迹按类别过滤
    artifacts::clean_filesystem_artifacts(ctx, categories);
}
