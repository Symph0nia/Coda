use crate::common::Context;
use std::path::Path;

pub fn clean_network_traces(ctx: &Context) {
    ctx.info("清理网络痕迹...");

    ctx.run_cmd("清空 ARP 缓存", "ip", &["neigh", "flush", "all"]);

    // resolvectl 优先，回退 systemd-resolve
    ctx.run_cmd_fallback(
        "清空 DNS 缓存",
        &[
            ("resolvectl", &["flush-caches"]),
            ("systemd-resolve", &["--flush-caches"]),
        ],
    );

    // NetworkManager 状态/租约类痕迹（不断网）
    let nm_state = Path::new("/var/lib/NetworkManager");
    if nm_state.exists() && nm_state.is_dir() {
        if let Ok(entries) = std::fs::read_dir(nm_state) {
            for entry in entries.flatten() {
                let ep = entry.path();
                let name = entry.file_name();
                let name = name.to_string_lossy();
                // 只清租约/时间戳类，不动设备状态核心
                if name.ends_with(".lease")
                    || name.ends_with(".leases")
                    || name == "timestamps"
                    || name.starts_with("dhclient-")
                    || name.starts_with("secret_key")
                {
                    if ep.is_file() {
                        ctx.remove(&ep);
                    }
                }
            }
        }
    }

    // 连接配置会断网 — 仅 aggressive
    if ctx.aggressive {
        ctx.info("aggressive: 清理 NetworkManager 连接配置...");
        let nm_conn = Path::new("/etc/NetworkManager/system-connections");
        if nm_conn.exists() && nm_conn.is_dir() {
            if let Ok(entries) = std::fs::read_dir(nm_conn) {
                for entry in entries.flatten() {
                    let ep = entry.path();
                    if ep.is_file() {
                        ctx.remove(&ep);
                    }
                }
            }
        }
    }

    // DHCP 租约
    let dhcp_paths = [
        "/var/lib/dhcp",
        "/var/lib/dhclient",
        "/var/lib/dhcpcd",
    ];
    for p in &dhcp_paths {
        let path = Path::new(p);
        if path.exists() {
            ctx.remove(path);
        }
    }
}

/// 仅清理当前会话历史变量；历史文件由 paths 统一收集删除。
pub fn clean_shell_env(ctx: &Context) {
    ctx.info("清理当前 Shell 会话历史...");
    ctx.run_cmd(
        "清除当前会话历史",
        "bash",
        &[
            "-c",
            "unset HISTFILE; export HISTSIZE=0; history -c; history -w 2>/dev/null || true",
        ],
    );
}

pub fn hide_process(ctx: &Context) {
    ctx.info("隐藏 /proc 进程信息 (aggressive)...");
    ctx.run_cmd(
        "设置 hidepid=2",
        "mount",
        &["-o", "remount,hidepid=2", "/proc"],
    );
}

pub fn clean_container_traces(ctx: &Context) {
    ctx.info("清理容器痕迹...");

    // 日志路径已由 paths 处理；此处只处理额外运行时状态
    if ctx.aggressive {
        ctx.info("aggressive: 清理容器运行时目录...");
        let runtime_paths = [
            "/var/lib/docker/containers",
            "/var/lib/containerd",
        ];
        for p in &runtime_paths {
            let path = Path::new(p);
            if path.exists() {
                ctx.remove(path);
            }
        }

        if let Ok(entries) = std::fs::read_dir("/run/user") {
            for entry in entries.flatten() {
                let containers = entry.path().join("containers");
                if containers.exists() {
                    ctx.remove(&containers);
                }
            }
        }
    } else {
        ctx.info("跳过容器运行时目录 (需要 --aggressive)");
    }
}
