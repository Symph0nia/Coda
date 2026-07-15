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
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    if !cli.restore && !cli.dry_run && !common::check_privilege() {
        eprintln!("权限不足，需要 root/管理员权限");
        std::process::exit(1);
    }

    let categories = Category::parse_list(&cli.categories);
    if categories.is_empty() && !cli.restore && !cli.selective {
        eprintln!(
            "无效的类别，可选: system,web,db,shell,temp,net,browser,container,audit,sec,mail,all"
        );
        std::process::exit(1);
    }

    let ctx = Context::new(
        cli.dry_run,
        cli.shred,
        cli.shred_passes,
        cli.truncate,
        cli.timestomp,
        cli.aggressive,
    );

    if cli.dry_run {
        ctx.info("DRY-RUN 模式: 以下操作不会实际执行");
        println!();
    }
    if cli.aggressive {
        ctx.info("AGGRESSIVE 模式: 允许破坏性操作");
    }

    if cli.delete {
        ctx.info("模式: 全部删除");
        dispatch_clean(&ctx, &categories);
    } else if cli.backup {
        ctx.info("模式: 备份后删除");
        backup_and_delete(&ctx, &categories);
    } else if cli.restore {
        ctx.info("模式: 恢复备份");
        restore(&ctx);
    } else if cli.selective {
        ctx.info("模式: 选择性清除");
        dispatch_selective(
            &ctx,
            cli.user.as_deref(),
            cli.ip.as_deref(),
            cli.tty.as_deref(),
        );
    }

    ctx.print_summary();

    if cli.self_destruct && !cli.dry_run {
        common::self_destruct();
    }

    if ctx.fail_count() > 0 {
        std::process::exit(1);
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

    const SIZE_LIMIT: u64 = 100 * 1024 * 1024;

    let patterns = paths::log_paths(categories);
    let targets = common::expand_targets(&patterns);

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

    let manifest_path = backup_dir.join("manifest.tsv");
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
        if size >= SIZE_LIMIT {
            ctx.info(&format!(
                "跳过备份 ({} > 100MB)，直接删除: {}",
                format_size(size),
                t.display()
            ));
            ctx.remove(t);
            continue;
        }

        let dest = backup_dir.join(common::backup_dest_name(t));
        if ctx.dry_run {
            println!("[预览] 备份 {} -> {}", t.display(), dest.display());
            ctx.record_preview();
        } else {
            // 优先 rename，跨设备则 copy+remove
            let moved = fs::rename(t, &dest).or_else(|_| {
                if t.is_dir() {
                    copy_dir_all(t, &dest)?;
                    fs::remove_dir_all(t)
                } else {
                    fs::copy(t, &dest)?;
                    fs::remove_file(t)
                }
            });
            match moved {
                Ok(()) => {
                    if let Some(ref mut m) = manifest {
                        let _ = writeln!(m, "{}\t{}", dest.display(), t.display());
                    }
                    println!("[备份] {} -> {}", t.display(), dest.display());
                    ctx.record_ok();
                }
                Err(e) => {
                    eprintln!("[失败] 备份 {} — {}", t.display(), e);
                    ctx.record_fail();
                }
            }
        }
    }

    if !ctx.dry_run {
        ctx.info(&format!("完成，清单: {}", manifest_path.display()));
    }
}

fn restore(ctx: &Context) {
    use std::fs;
    use std::io::{BufRead, BufReader};

    let backup_dir = std::env::temp_dir().join("coda_backups");
    // 兼容新旧清单
    let manifest_path = {
        let tsv = backup_dir.join("manifest.tsv");
        let csv = backup_dir.join("manifest.csv");
        if tsv.exists() {
            tsv
        } else {
            csv
        }
    };

    let file = match fs::File::open(&manifest_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("读取清单失败: {} — {}", manifest_path.display(), e);
            return;
        }
    };

    let sep = if manifest_path.extension().is_some_and(|e| e == "tsv") {
        '\t'
    } else {
        ','
    };

    let reader = BufReader::new(file);
    let mut count = 0u32;
    for line in reader.lines().flatten() {
        let parts: Vec<&str> = line.splitn(2, sep).collect();
        if parts.len() != 2 {
            continue;
        }
        let (backup, original) = (Path::new(parts[0]), Path::new(parts[1]));
        if let Some(parent) = original.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if ctx.dry_run {
            println!("[预览] 恢复 {} -> {}", backup.display(), original.display());
            ctx.record_preview();
            continue;
        }
        match fs::rename(backup, original).or_else(|_| {
            if backup.is_dir() {
                copy_dir_all(backup, original)?;
                fs::remove_dir_all(backup)
            } else {
                fs::copy(backup, original)?;
                fs::remove_file(backup)
            }
        }) {
            Ok(()) => {
                println!("[恢复] {} -> {}", backup.display(), original.display());
                count += 1;
                ctx.record_ok();
            }
            Err(e) => {
                eprintln!(
                    "[失败] {} -> {} — {}",
                    backup.display(),
                    original.display(),
                    e
                );
                ctx.record_fail();
            }
        }
    }
    if !ctx.dry_run {
        println!("恢复完成，共 {} 项", count);
    }
}

fn path_size(path: &Path) -> u64 {
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

fn format_size(n: u64) -> String {
    const MB: u64 = 1024 * 1024;
    if n >= MB {
        format!("{:.1}MB", n as f64 / MB as f64)
    } else {
        format!("{}B", n)
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}
