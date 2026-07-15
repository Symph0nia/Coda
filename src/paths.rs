use crate::cli::Category;

struct CategorizedPath {
    category: Category,
    path: &'static str,
}

macro_rules! cp {
    ($cat:ident, $path:expr) => {
        CategorizedPath { category: Category::$cat, path: $path }
    };
}

#[cfg(unix)]
pub fn log_paths(categories: &[Category]) -> Vec<String> {
    let all: Vec<CategorizedPath> = vec![
        // 系统日志
        cp!(System, "/var/log/syslog"),
        cp!(System, "/var/log/messages"),
        cp!(System, "/var/log/auth.log"),
        cp!(System, "/var/log/lastlog"),
        cp!(System, "/var/log/wtmp"),
        cp!(System, "/var/log/btmp"),
        cp!(System, "/var/log/faillog"),
        cp!(System, "/var/log/wtmp.1"),
        cp!(System, "/var/log/btmp.1"),
        cp!(System, "/var/log/dmesg"),
        cp!(System, "/var/log/daemon.log"),
        cp!(System, "/var/log/kern.log"),
        cp!(System, "/var/log/secure"),
        cp!(System, "/var/log/sudo.log"),
        cp!(System, "/var/log/cron"),
        cp!(System, "/var/log/cron.log"),
        cp!(System, "/var/run/utmp"),
        // SSH
        cp!(System, "/var/log/sshd.log"),
        cp!(System, "/var/log/sshd/*"),
        // 审计
        cp!(Audit, "/var/log/audit/audit.log"),
        cp!(Audit, "/var/log/audit/*"),
        // 安全工具
        cp!(Security, "/var/log/rkhunter/rkhunter.log"),
        cp!(Security, "/var/log/chkrootkit/chkrootkit.log"),
        cp!(Security, "/var/log/clamav/*"),
        cp!(Security, "/var/log/fail2ban.log"),
        // Web 服务
        cp!(Web, "/var/log/apache2/access.log"),
        cp!(Web, "/var/log/apache2/error.log"),
        cp!(Web, "/var/log/httpd/access_log"),
        cp!(Web, "/var/log/httpd/error_log"),
        cp!(Web, "/var/log/nginx/access.log"),
        cp!(Web, "/var/log/nginx/error.log"),
        cp!(Web, "/var/log/tomcat/*"),
        cp!(Web, "/var/log/glassfish/*"),
        // 数据库
        cp!(Database, "/var/log/mysql/error.log"),
        cp!(Database, "/var/log/mysql/mysql.log"),
        cp!(Database, "/var/log/mysql/*"),
        cp!(Database, "/var/log/postgresql/*"),
        cp!(Database, "/var/log/mongodb/mongod.log"),
        cp!(Database, "/var/log/redis/redis.log"),
        // 邮件
        cp!(Mail, "/var/log/mail.log"),
        cp!(Mail, "/var/log/mail.err"),
        cp!(Mail, "/var/log/maillog"),
        // 容器
        cp!(Container, "/var/log/docker.log"),
        cp!(Container, "/var/log/kubernetes/*"),
        cp!(Container, "/var/log/containers/*"),
        cp!(Container, "/var/log/pods/*"),
        // 防火墙 & 网络
        cp!(Network, "/var/log/firewalld"),
        cp!(Network, "/var/log/iptables.log"),
        cp!(Network, "/var/log/samba/*"),
        cp!(Network, "/var/log/rsyncd.log"),
        cp!(Network, "/var/log/openvpn/*"),
        cp!(Network, "/var/log/wireguard/*"),
        // 包管理器 (归入系统)
        cp!(System, "/var/log/yum.log"),
        cp!(System, "/var/log/apt/history.log"),
        cp!(System, "/var/log/apt/term.log"),
        cp!(System, "/var/log/dnf.log"),
        cp!(System, "/var/log/pacman.log"),
        // 临时文件
        cp!(Temp, "/tmp/*"),
        cp!(Temp, "/var/tmp/*"),
    ];

    let mut paths: Vec<String> = all
        .into_iter()
        .filter(|cp| categories.contains(&cp.category))
        .map(|cp| cp.path.to_string())
        .collect();

    // Shell 历史 (动态)
    if categories.contains(&Category::Shell) {
        let history_files = [
            ".bash_history", ".zsh_history", ".python_history",
            ".node_repl_history", ".mysql_history", ".psql_history",
            ".rediscli_history", ".lesshst", ".viminfo", ".wget-hsts",
        ];

        // root
        for f in &history_files {
            paths.push(format!("/root/{}", f));
        }

        // /home/*
        if let Ok(entries) = std::fs::read_dir("/home") {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let base = entry.path().display().to_string();
                    for f in &history_files {
                        paths.push(format!("{}/{}", base, f));
                    }
                }
            }
        }
    }

    // 浏览器缓存 (Linux)
    if categories.contains(&Category::Browser) {
        if let Ok(entries) = std::fs::read_dir("/home") {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let base = entry.path().display().to_string();
                    paths.push(format!("{}/.cache/mozilla/firefox/*/cache2/*", base));
                    paths.push(format!("{}/.cache/google-chrome/Default/Cache/*", base));
                    paths.push(format!("{}/.cache/chromium/Default/Cache/*", base));
                    paths.push(format!("{}/.mozilla/firefox/*/places.sqlite", base));
                }
            }
        }
    }

    paths
}

#[cfg(windows)]
pub fn log_paths(categories: &[Category]) -> Vec<String> {
    let username = std::env::var("USERNAME").unwrap_or_default();
    let user_dir = format!(r"C:\Users\{}", username);

    let all: Vec<CategorizedPath> = vec![
        // 系统事件日志
        cp!(System, r"C:\Windows\System32\winevt\Logs\Security.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\Application.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\System.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\Setup.evtx"),
        cp!(Audit, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-Security-Auditing.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-User Profile Service%4Operational.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-DNS-Client%4Operational.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-DNS-Server%4Analytical.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-Windows Firewall With Advanced Security%4Firewall.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-TaskScheduler%4Operational.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-SMBClient\Operational.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-RemoteDesktopServices-RdpCoreTS\Operational.evtx"),
        cp!(Network, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-TerminalServices-LocalSessionManager\Operational.evtx"),
        cp!(Security, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-Windows Defender\Operational.evtx"),
        cp!(System, r"C:\Windows\System32\winevt\Logs\Microsoft-Windows-Sysmon\Operational.evtx"),
        // 防火墙 & HTTP 日志
        cp!(Network, r"C:\Windows\System32\LogFiles\Firewall\pfirewall.log"),
        cp!(Web, r"C:\Windows\System32\LogFiles\HTTPERR\httperr1.log"),
        cp!(Web, r"C:\Windows\System32\LogFiles\W3SVC1\*"),
        // IIS
        cp!(Web, r"C:\inetpub\logs\LogFiles\W3SVC1\*"),
        cp!(Web, r"C:\inetpub\logs\FailedReqLogFiles\*"),
        // Apache
        cp!(Web, r"C:\Program Files\Apache Group\Apache2\logs\access.log"),
        cp!(Web, r"C:\Program Files\Apache Group\Apache2\logs\error.log"),
        cp!(Web, r"C:\Program Files (x86)\IIS Express\Logs\IISExpress.log"),
    ];

    let mut paths: Vec<String> = all
        .into_iter()
        .filter(|cp| categories.contains(&cp.category))
        .map(|cp| cp.path.to_string())
        .collect();

    // 临时文件
    if categories.contains(&Category::Temp) {
        paths.push(r"C:\Windows\Temp\*".into());
        paths.push(format!(r"{}\AppData\Local\Temp\*", user_dir));
        paths.push(format!(r"{}\AppData\LocalLow\Temp\*", user_dir));
    }

    // 用户痕迹/浏览器
    if categories.contains(&Category::Browser) {
        paths.push(format!(r"{}\AppData\Roaming\Microsoft\Windows\Recent\*", user_dir));
        paths.push(format!(r"{}\AppData\Local\Microsoft\Windows\INetCache\*", user_dir));
        paths.push(format!(r"{}\AppData\Local\Microsoft\Windows\History\*", user_dir));

        // Firefox
        let firefox_profiles = format!(r"{}\AppData\Local\Mozilla\Firefox\Profiles", user_dir);
        if let Ok(entries) = std::fs::read_dir(&firefox_profiles) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let base = entry.path().display().to_string();
                    paths.push(format!(r"{}\cache2", base));
                    paths.push(format!(r"{}\places.sqlite", base));
                }
            }
        }

        // Chrome
        let chrome_cache = format!(r"{}\AppData\Local\Google\Chrome\User Data\Default\Cache", user_dir);
        paths.push(format!(r"{}\*", chrome_cache));

        // Edge
        let edge_cache = format!(r"{}\AppData\Local\Microsoft\Edge\User Data\Default\Cache", user_dir);
        paths.push(format!(r"{}\*", edge_cache));
    }

    paths
}
