# Coda

跨平台入侵痕迹清除工具，支持 Windows / Linux，Rust 编写。

## 特性

- **精确清除**：解析 utmp/wtmp/btmp/lastlog 二进制结构，按用户名/IP/TTY 精确删除特定登录记录，而非暴力删文件
- **安全覆写**：多轮随机数据覆写 + 零填充，防止数据恢复
- **Dry-run 模式**：默认可预览所有操作目标，确认后再执行
- **分类选择**：按类别（系统/Web/数据库/Shell/网络/浏览器/容器/审计等）选择清理范围
- **日志截断**：截断日志文件为零字节而非删除，保持文件存在避免服务报错
- **自毁机制**：执行完成后覆写并删除自身二进制

### Linux

| 功能 | 说明 |
|---|---|
| utmp/wtmp/btmp 精确清除 | 按用户/IP/TTY 筛选删除特定条目 |
| lastlog 精确清零 | 按 UID 偏移清零对应记录 |
| systemd journal | rotate + vacuum + 清理目录 |
| auditd | 清空规则 + 删除日志 + 停止服务 |
| Shell 历史 | bash/zsh/python/mysql/redis 等 10 种历史文件 |
| 轮转日志 | 递归处理 .1/.gz/.bz2/.xz/.zst/.old |
| 进程隐藏 | hidepid=2 |
| 网络痕迹 | ARP/DNS/NetworkManager/DHCP |
| 容器痕迹 | Docker/Kubernetes/Podman |

### Windows

| 功能 | 说明 |
|---|---|
| Event Log 批量清除 | wevtutil 枚举并清除所有 channel |
| 注册表痕迹 | AmCache/ShimCache/BAM/DAM/UserAssist/ShellBags/MRU/MUICache |
| RDP 连接历史 | 注册表 + Default.rdp |
| Prefetch / Superfetch | 删除 .pf 文件 |
| USN Journal | fsutil 清除 NTFS 变更日志 |
| SRUM 数据库 | 进程资源使用监控记录 |
| PowerShell 日志 | ScriptBlock/Transcription/ConsoleHost_history |
| Thumbcache / Jump Lists | 缩略图缓存 + 任务栏记录 |
| 回收站 / VSS | $Recycle.Bin + 卷影副本 |
| Windows Defender | 隔离文件 + 扫描历史 + 日志 |

## 编译

```bash
# Linux
cargo build --release

# Windows 交叉编译
cargo build --release --target x86_64-pc-windows-gnu
```

## 使用

```bash
# 预览模式（不执行任何操作）
coda -D --dry-run

# 删除全部日志
coda -D

# 仅清理特定类别
coda -D -c system,shell,web

# 安全覆写删除（3轮随机+零填充）
coda -D --shred

# 截断模式（文件清零但保留）
coda -D --truncate

# 选择性清除：精确删除指定用户的登录记录
coda -S --user attacker
coda -S --ip 192.168.1.100
coda -S --user attacker --ip 10.0.0.1

# 备份后删除
coda -B

# 恢复备份
coda -R

# 执行完后自毁
coda -D --shred --self-destruct
```

### 可用类别

| 类别 | 缩写 | 覆盖内容 |
|---|---|---|
| system | - | 系统日志、SSH、包管理器、cron |
| web | - | Apache/Nginx/IIS/Tomcat |
| database | db | MySQL/PostgreSQL/MongoDB/Redis |
| shell | - | bash/zsh/python 等历史文件 |
| temp | tmp | 临时文件目录 |
| network | net | 防火墙/VPN/Samba/ARP/DNS |
| browser | - | Firefox/Chrome/Edge 缓存 |
| container | docker | Docker/Kubernetes/Podman |
| audit | - | auditd/systemd journal |
| security | sec | rkhunter/ClamAV/fail2ban/Defender |
| mail | - | 邮件系统日志 |
| all | - | 以上全部 |

## 注意事项

> [!WARNING]
> 仅用于授权的渗透测试和安全研究。Coda 对系统文件的操作不可逆。

> [!TIP]
> 建议先使用 `--dry-run` 预览操作目标，确认无误后再执行。
