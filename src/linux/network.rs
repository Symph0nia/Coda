use crate::common::Context;
use std::path::Path;

pub fn clean_network_traces(ctx: &Context) {
    ctx.info("清理网络痕迹...");

    // ARP 缓存
    ctx.run_cmd("清空 ARP 缓存", "ip", &["neigh", "flush", "all"]);

    // DNS 缓存 (systemd-resolved)
    ctx.run_cmd("清空 DNS 缓存", "systemd-resolve", &["--flush-caches"]);

    // NetworkManager 连接记录
    let nm_paths = [
        "/etc/NetworkManager/system-connections",
        "/var/lib/NetworkManager",
    ];
    for p in &nm_paths {
        let path = Path::new(p);
        if path.exists() && path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
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

pub fn clean_shell_env(ctx: &Context) {
    ctx.info("清理 Shell 环境与历史...");

    // 当前会话的环境变量清理通过执行 shell 命令
    ctx.run_cmd(
        "清除当前会话历史",
        "bash",
        &["-c", "unset HISTFILE; export HISTSIZE=0; history -c; history -w 2>/dev/null || true"],
    );

    // 所有用户的 shell 历史文件
    let history_patterns = [
        "/root/.bash_history",
        "/root/.zsh_history",
        "/root/.python_history",
        "/root/.node_repl_history",
        "/root/.mysql_history",
        "/root/.psql_history",
        "/root/.rediscli_history",
        "/root/.lesshst",
        "/root/.viminfo",
    ];

    for p in &history_patterns {
        let path = Path::new(p);
        if path.exists() {
            ctx.remove(path);
        }
    }

    // /home 下所有用户
    if let Ok(entries) = std::fs::read_dir("/home") {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let home = entry.path();
            let history_files = [
                ".bash_history",
                ".zsh_history",
                ".python_history",
                ".node_repl_history",
                ".mysql_history",
                ".psql_history",
                ".rediscli_history",
                ".lesshst",
                ".viminfo",
                ".wget-hsts",
            ];
            for f in &history_files {
                let p = home.join(f);
                if p.exists() {
                    ctx.remove(&p);
                }
            }
        }
    }
}

pub fn hide_process(ctx: &Context) {
    ctx.info("隐藏 /proc 进程信息...");
    ctx.run_cmd(
        "设置 hidepid=2",
        "mount",
        &["-o", "remount,hidepid=2", "/proc"],
    );
}

pub fn clean_container_traces(ctx: &Context) {
    ctx.info("清理容器痕迹...");

    // Docker
    let docker_paths = [
        "/var/lib/docker/containers",
        "/var/log/docker.log",
    ];
    for p in &docker_paths {
        let path = Path::new(p);
        if path.exists() {
            ctx.remove(path);
        }
    }

    // Kubernetes
    let k8s_paths = [
        "/var/log/kubernetes",
        "/var/log/containers",
        "/var/log/pods",
    ];
    for p in &k8s_paths {
        let path = Path::new(p);
        if path.exists() {
            ctx.remove(path);
        }
    }

    // Podman
    if let Ok(entries) = std::fs::read_dir("/run/user") {
        for entry in entries.flatten() {
            let containers = entry.path().join("containers");
            if containers.exists() {
                ctx.remove(&containers);
            }
        }
    }
}
