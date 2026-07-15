use crate::common::Context;

pub fn clean_registry_traces(ctx: &Context) {
    ctx.info("清理注册表取证痕迹...");

    // AmCache — 程序执行历史
    clean_amcache(ctx);

    // ShimCache / AppCompatCache
    clean_shimcache(ctx);

    // BAM/DAM — 后台活动监控
    clean_bam(ctx);

    // UserAssist — Explorer 运行计数
    clean_userassist(ctx);

    // ShellBags — 目录浏览历史
    clean_shellbags(ctx);

    // MRU 列表
    clean_mru(ctx);

    // MUI Cache
    clean_muicache(ctx);

    // RDP 连接历史
    clean_rdp_history(ctx);
}

fn clean_amcache(ctx: &Context) {
    ctx.run_cmd(
        "删除 AmCache",
        "reg",
        &["delete", r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\InboxApplications", "/f"],
    );
    // AmCache.hve 文件
    let amcache = r"C:\Windows\appcompat\Programs\Amcache.hve";
    let p = std::path::Path::new(amcache);
    if p.exists() {
        ctx.remove(p);
    }
}

fn clean_shimcache(ctx: &Context) {
    ctx.run_cmd(
        "清除 ShimCache",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatCache",
            "/v", "AppCompatCache",
            "/f",
        ],
    );
}

fn clean_bam(ctx: &Context) {
    // BAM (Background Activity Moderator)
    ctx.run_cmd(
        "清除 BAM",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Services\bam\State\UserSettings",
            "/f",
        ],
    );
    // DAM (Desktop Activity Moderator)
    ctx.run_cmd(
        "清除 DAM",
        "reg",
        &[
            "delete",
            r"HKLM\SYSTEM\CurrentControlSet\Services\dam\State\UserSettings",
            "/f",
        ],
    );
}

fn clean_userassist(ctx: &Context) {
    // UserAssist 在 HKCU 下，ROT13 编码的 GUID 子键
    let guids = [
        "{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}",
        "{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}",
    ];
    for guid in &guids {
        ctx.run_cmd(
            &format!("清除 UserAssist {}", guid),
            "reg",
            &[
                "delete",
                &format!(
                    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{}\Count",
                    guid
                ),
                "/f",
            ],
        );
    }
}

fn clean_shellbags(ctx: &Context) {
    let keys = [
        r"HKCU\SOFTWARE\Microsoft\Windows\Shell\BagMRU",
        r"HKCU\SOFTWARE\Microsoft\Windows\Shell\Bags",
        r"HKCU\SOFTWARE\Microsoft\Windows\ShellNoRoam\BagMRU",
        r"HKCU\SOFTWARE\Microsoft\Windows\ShellNoRoam\Bags",
    ];
    for key in &keys {
        ctx.run_cmd(
            &format!("清除 ShellBags: {}", key),
            "reg",
            &["delete", key, "/f"],
        );
    }
}

fn clean_mru(ctx: &Context) {
    let keys = [
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\RunMRU",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\OpenSavePidlMRU",
        r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\LastVisitedPidlMRU",
        r"HKCU\SOFTWARE\Microsoft\Office",  // Office MRU
    ];
    for key in &keys {
        ctx.run_cmd(
            &format!("清除 MRU: {}", key),
            "reg",
            &["delete", key, "/f"],
        );
    }
}

fn clean_muicache(ctx: &Context) {
    ctx.run_cmd(
        "清除 MUI Cache",
        "reg",
        &[
            "delete",
            r"HKCU\SOFTWARE\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache",
            "/f",
        ],
    );
}

fn clean_rdp_history(ctx: &Context) {
    ctx.run_cmd(
        "清除 RDP 连接历史",
        "reg",
        &[
            "delete",
            r"HKCU\SOFTWARE\Microsoft\Terminal Server Client",
            "/f",
        ],
    );
    // Default.rdp 文件
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let rdp = std::path::Path::new(&userprofile).join("Documents").join("Default.rdp");
        if rdp.exists() {
            ctx.remove(&rdp);
        }
    }
}
