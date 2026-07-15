use clap::{ArgGroup, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    System,
    Web,
    Database,
    Shell,
    Temp,
    Network,
    Browser,
    Container,
    Audit,
    Security,
    Mail,
}

impl Category {
    pub fn parse_list(s: &str) -> Vec<Category> {
        if s == "all" {
            return Self::all();
        }
        s.split(',')
            .filter_map(|c| match c.trim() {
                "system" => Some(Category::System),
                "web" => Some(Category::Web),
                "database" | "db" => Some(Category::Database),
                "shell" => Some(Category::Shell),
                "temp" | "tmp" => Some(Category::Temp),
                "network" | "net" => Some(Category::Network),
                "browser" => Some(Category::Browser),
                "container" | "docker" => Some(Category::Container),
                "audit" => Some(Category::Audit),
                "security" | "sec" => Some(Category::Security),
                "mail" => Some(Category::Mail),
                _ => None,
            })
            .collect()
    }

    pub fn all() -> Vec<Category> {
        vec![
            Category::System,
            Category::Web,
            Category::Database,
            Category::Shell,
            Category::Temp,
            Category::Network,
            Category::Browser,
            Category::Container,
            Category::Audit,
            Category::Security,
            Category::Mail,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all() {
        assert_eq!(Category::parse_list("all").len(), 11);
    }

    #[test]
    fn parse_aliases() {
        let v = Category::parse_list("db,net,tmp,sec,docker");
        assert!(v.contains(&Category::Database));
        assert!(v.contains(&Category::Network));
        assert!(v.contains(&Category::Temp));
        assert!(v.contains(&Category::Security));
        assert!(v.contains(&Category::Container));
    }

    #[test]
    fn parse_ignores_unknown() {
        let v = Category::parse_list("shell,nope,web");
        assert_eq!(v, vec![Category::Shell, Category::Web]);
    }

    #[test]
    fn parse_empty_unknown() {
        assert!(Category::parse_list("foo,bar").is_empty());
    }
}

/// Shell 历史文件名（相对用户家目录），全项目唯一来源。
#[cfg(unix)]
pub const HISTORY_FILES: &[&str] = &[
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

#[derive(Parser)]
#[command(name = "coda", version, about = "入侵痕迹清除工具")]
#[command(group(
    ArgGroup::new("mode")
        .required(true)
        .args(["delete", "backup", "restore", "selective"])
))]
pub struct Cli {
    #[arg(short = 'D', long = "delete", help = "删除全部日志")]
    pub delete: bool,

    #[arg(short = 'B', long = "backup", help = "备份小文件后删除全部")]
    pub backup: bool,

    #[arg(short = 'R', long = "restore", help = "恢复备份")]
    pub restore: bool,

    #[arg(short = 'S', long = "selective", help = "选择性清除 (utmp/wtmp/lastlog 精确模式)")]
    pub selective: bool,

    #[arg(long = "dry-run", help = "只展示目标，不执行任何操作")]
    pub dry_run: bool,

    #[arg(long = "shred", help = "安全覆写删除 (默认3轮随机+1轮零填充)")]
    pub shred: bool,

    #[arg(long = "shred-passes", default_value = "3", help = "安全覆写轮数")]
    pub shred_passes: u32,

    #[arg(long = "truncate", help = "截断文件为零字节而非删除")]
    pub truncate: bool,

    #[arg(long = "timestomp", help = "截断后还原文件时间戳")]
    pub timestomp: bool,

    #[arg(long = "aggressive", help = "允许破坏性操作 (断网配置/hidepid/容器运行时目录等)")]
    pub aggressive: bool,

    #[arg(long = "self-destruct", help = "执行完成后删除自身二进制")]
    pub self_destruct: bool,

    #[arg(
        short = 'c',
        long = "categories",
        default_value = "all",
        help = "清理类别: system,web,db,shell,temp,net,browser,container,audit,sec,mail,all"
    )]
    pub categories: String,

    #[arg(long = "user", help = "选择性清除: 按用户名筛选")]
    pub user: Option<String>,

    #[arg(long = "ip", help = "选择性清除: 按 IP 筛选")]
    pub ip: Option<String>,

    #[arg(long = "tty", help = "选择性清除: 按 TTY 筛选")]
    pub tty: Option<String>,
}
