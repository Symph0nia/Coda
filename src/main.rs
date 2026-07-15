mod cli;
mod common;
mod paths;
mod shred;

#[cfg(unix)]
mod linux;
#[cfg(windows)]
mod windows;

use clap::Parser;
use cli::{Category, Cli};
use common::Context;

fn main() {
    let cli = Cli::parse();

    if !cli.restore && !cli.dry_run && !common::check_privilege() {
        eprintln!("权限不足，需要 root/管理员权限");
        std::process::exit(1);
    }

    let categories = Category::parse_list(&cli.categories);
    if categories.is_empty() && !cli.restore && !cli.selective {
        eprintln!("无效的类别，可选: system,web,db,shell,temp,net,browser,container,audit,sec,mail,all");
        std::process::exit(1);
    }

    let ctx = Context {
        dry_run: cli.dry_run,
        shred: cli.shred,
        shred_passes: cli.shred_passes,
        truncate_mode: cli.truncate,
        timestomp: cli.timestomp,
    };

    if cli.dry_run {
        ctx.info("DRY-RUN 模式: 以下操作不会实际执行");
        println!();
    }

    match (cli.delete, cli.backup, cli.restore, cli.selective) {
        (true, false, false, false) => {
            ctx.info("模式: 全部删除");
            dispatch_clean(&ctx, &categories);
        }
        (false, true, false, false) => {
            ctx.info("模式: 备份后删除");
            backup_and_delete(&ctx, &categories);
        }
        (false, false, true, false) => {
            ctx.info("模式: 恢复备份");
            restore();
        }
        (false, false, false, true) => {
            ctx.info("模式: 选择性清除");
            dispatch_selective(&ctx, cli.user.as_deref(), cli.ip.as_deref(), cli.tty.as_deref());
        }
        _ => {
            eprintln!("请指定一个操作模式: -D, -B, -R, 或 -S");
            std::process::exit(1);
        }
    }

    if cli.self_destruct {
        common::self_destruct();
    }
}

fn dispatch_clean(ctx: &Context, categories: &[Category]) {
    #[cfg(unix)]
    linux::clean_all(ctx, categories);
    #[cfg(windows)]
    windows::clean_all(ctx, categories);
}

fn dispatch_selective(ctx: &Context, user: Option<&str>, ip: Option<&str>, tty: Option<&str>) {
    #[cfg(unix)]
    linux::selective_clean(ctx, user, ip, tty);
    #[cfg(windows)]
    {
        let _ = (user, ip, tty);
        ctx.info("Windows 暂不支持选择性清除模式");
    }
}

fn backup_and_delete(ctx: &Context, categories: &[Category]) {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    const SIZE_LIMIT: u64 = 100 * 1024 * 1024;

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
    let targets: Vec<PathBuf> = targets.into_iter().filter(|p| p.exists()).collect();

    if targets.is_empty() {
        ctx.info("没有找到需要清理的目标");
        return;
    }

    let backup_dir = std::env::temp_dir().join("coda_backups");
    if ctx.dry_run {
        ctx.info(&format!("将备份到: {}", backup_dir.display()));
    } else {
        fs::create_dir_all(&backup_dir).unwrap_or_else(|e| {
            eprintln!("创建备份目录失败: {}", e);
            std::process::exit(1);
        });
    }

    let manifest_path = backup_dir.join("manifest.csv");
    let mut manifest = if ctx.dry_run {
        None
    } else {
        Some(fs::File::create(&manifest_path).unwrap_or_else(|e| {
            eprintln!("创建清单文件失败: {}", e);
            std::process::exit(1);
        }))
    };

    ctx.info(&format!("发现 {} 个目标...", targets.len()));
    for t in &targets {
        let size = path_size(t);
        if size < SIZE_LIMIT {
            let dest = backup_dir.join(t.file_name().unwrap_or_default());
            if ctx.dry_run {
                println!("[预览] 备份 {} -> {}", t.display(), dest.display());
            } else {
                match fs::rename(t, &dest) {
                    Ok(()) => {
                        if let Some(ref mut m) = manifest {
                            let _ = writeln!(m, "{},{}", dest.display(), t.display());
                        }
                        println!("[备份] {} -> {}", t.display(), dest.display());
                    }
                    Err(e) => eprintln!("[失败] 备份 {} — {}", t.display(), e),
                }
            }
        } else {
            ctx.remove(t);
        }
    }

    if !ctx.dry_run {
        ctx.info(&format!("完成，清单: {}", manifest_path.display()));
    }
}

fn restore() {
    use std::fs;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    let manifest_path = std::env::temp_dir().join("coda_backups").join("manifest.csv");
    let file = match fs::File::open(&manifest_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("读取清单失败: {} — {}", manifest_path.display(), e);
            return;
        }
    };

    let reader = BufReader::new(file);
    let mut count = 0u32;
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() != 2 {
            continue;
        }
        let (backup, original) = (Path::new(parts[0]), Path::new(parts[1]));
        if let Some(parent) = original.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::rename(backup, original) {
            Ok(()) => {
                println!("[恢复] {} -> {}", backup.display(), original.display());
                count += 1;
            }
            Err(e) => eprintln!("[失败] {} -> {} — {}", backup.display(), original.display(), e),
        }
    }
    println!("恢复完成，共 {} 项", count);
}

fn path_size(path: &std::path::Path) -> u64 {
    if path.is_file() {
        return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += path_size(&p);
            } else {
                total += std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}
