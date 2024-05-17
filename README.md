# Coda
## 0x01 简介

Coda是一款支持Windows/Linux系统的入侵痕迹抹除工具，可以帮助攻击者迅速消除入侵痕迹，使用Golang语言编写。

## 0x02 编译方法

```bash
git clone https://github.com/Symph0nia/Coda.git
```

```
cd Coda
```

```
go build ./main.go
```

## 0x03 使用方法

Coda接收三个参数，分别为：

```
./coda -D # 删除所有的日志信息
./coda -B # 删除大型的的日志信息，将小型的日志信息备份到/TEMP或/tmp文件夹下
./coda -R # 恢复备份的日志信息到原位置
```

## 0x04 原理

Coda的基本用法即直接删除所有的日志，从而实现对溯源的打击。

Coda的进阶用法为Backup 2 Restore

在渗透成功后，运行Coda -B，对当前的日志信息进行镜像保存，接下来可以进行敏感操作，例如数据获取，数据删除一类。

在渗透结束阶段，运行Coda -R，对已经保存的日志信息进行恢复，不对日志系统进行完全清除，从而实现一个优雅的空白监控时间。

## 0x05 注意事项

> [!WARNING]
>
> Coda对系统文件所造成的伤害是不可逆的，慎用。

目前的日志分类规则：大于100MB

目前的清除日志列表：

Windows：

| 日志文件路径                                                 | 作用                                                         |
| ------------------------------------------------------------ | ------------------------------------------------------------ |
| `C:\\Windows\\System32\\winevt\\Logs\\Security.evtx`         | 记录安全相关事件，例如登录尝试、权限使用等。                 |
| `C:\\Windows\\System32\\winevt\\Logs\\Application.evtx`      | 记录应用程序相关事件，由应用程序生成的日志信息。             |
| `C:\\Windows\\System32\\winevt\\Logs\\System.evtx`           | 记录系统级别事件，包括驱动程序加载、系统组件的启动和停止等。 |
| `C:\\Program Files\\Apache Group\\Apache2\\logs\\access.log` | 记录Apache服务器的访问日志，包括每个请求的详细信息。         |
| `C:\\Program Files\\Apache Group\\Apache2\\logs\\error.log`  | 记录Apache服务器的错误日志，包括启动、运行时错误和异常。     |
| `C:\\Program Files (x86)\\IIS Express\\Logs\\IISExpress.log` | 记录IIS Express的日志，包括访问和错误信息。                  |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-User Profile Service%4Operational.evtx` | 记录用户配置文件服务的操作日志。                             |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-DNS-Client%4Operational.evtx` | 记录DNS客户端操作日志。                                      |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-DNS-Server%4Analytical.evtx` | 记录DNS服务器分析日志，用于诊断DNS问题。                     |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-Windows Firewall With Advanced Security%4Firewall.evtx` | 记录高级安全Windows防火墙的操作日志。                        |
| `C:\\Windows\\System32\\LogFiles\\Firewall\\pfirewall.log`   | 记录Windows防火墙日志，包括被允许或被阻止的网络连接。        |
| `C:\\Windows\\System32\\LogFiles\\W3SVC1\\`                  | 记录IIS Web服务的日志，包括访问和错误日志。                  |
| `C:\\Windows\\System32\\LogFiles\\HTTPERR\\httperr1.log`     | 记录IIS HTTP错误日志，包括无法处理的HTTP请求。               |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-Security-Auditing.evtx` | 记录安全审计日志，包括成功和失败的安全事件。                 |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-TaskScheduler%4Operational.evtx` | 记录任务计划程序的操作日志。                                 |
| `C:\\Windows\\Temp`                                          | 存储临时文件，通常用于短期存储。                             |
| `filepath.Join("C:\\Users", username, "AppData\\Local\\Temp")` | 当前用户的本地临时文件夹，存储临时文件。                     |
| `filepath.Join("C:\\Users", username, "AppData\\LocalLow\\Temp")` | 当前用户的本地低权限临时文件夹，存储低权限临时文件。         |
| `filepath.Join("C:\\Users", username, "AppData\\Roaming\\Microsoft\\Windows\\Recent")` | 记录当前用户最近访问的文件和文件夹。                         |
| `filepath.Join("C:\\Users", username, "AppData\\Local\\Microsoft\\Windows\\INetCache")` | 存储浏览器缓存文件。                                         |
| `filepath.Join("C:\\Users", username, "AppData\\Local\\Microsoft\\Windows\\History")` | 存储浏览器历史记录。                                         |
| `filepath.Join("C:\\Users", username, "Documents")`          | 当前用户的文档文件夹，存储用户的文档文件。                   |
| `filepath.Join("C:\\Users", username, "Downloads")`          | 当前用户的下载文件夹，存储用户下载的文件。                   |
| `C:\\inetpub\\logs\\LogFiles\\W3SVC1\\`                      | 记录IIS Web服务的访问日志。                                  |
| `C:\\inetpub\\logs\\FailedReqLogFiles\\`                     | 记录IIS失败的请求日志，用于诊断失败的HTTP请求。              |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-SMBClient\\Operational.evtx` | 记录SMB客户端操作日志。                                      |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-RemoteDesktopServices-RdpCoreTS\\Operational.evtx` | 记录远程桌面服务操作日志。                                   |
| `C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-TerminalServices-LocalSessionManager\\Operational.evtx` | 记录终端服务本地会话管理器操作日志。                         |

Linux：

| 日志文件路径                         | 作用                                                         |
| ------------------------------------ | ------------------------------------------------------------ |
| `/var/log/syslog`                    | 记录系统日志，包括内核消息、服务启动和停止、各种系统事件。   |
| `/var/log/messages`                  | 记录一般系统消息和非关键系统错误。                           |
| `/var/log/auth.log`                  | 记录认证相关的日志，包括登录尝试、sudo使用等。               |
| `/var/log/lastlog`                   | 记录上次登录的信息。                                         |
| `/var/log/wtmp`                      | 记录登录和注销事件的永久日志。                               |
| `/var/log/btmp`                      | 记录失败的登录尝试。                                         |
| `/var/log/faillog`                   | 记录失败的用户登录信息。                                     |
| `/var/log/apache2/access.log`        | 记录Apache服务器的访问日志，包括每个请求的详细信息。         |
| `/var/log/apache2/error.log`         | 记录Apache服务器的错误日志，包括启动、运行时错误和异常。     |
| `/var/log/nginx/access.log`          | 记录Nginx服务器的访问日志，包括每个请求的详细信息。          |
| `/var/log/nginx/error.log`           | 记录Nginx服务器的错误日志，包括启动、运行时错误和异常。      |
| `/var/log/mysql/error.log`           | 记录MySQL数据库的错误日志，包括启动、运行时错误和异常。      |
| `/var/log/mysql/mysql.log`           | 记录MySQL数据库的一般操作日志。                              |
| `/var/log/daemon.log`                | 记录守护进程的日志，包括系统服务的启动和停止。               |
| `/var/log/kern.log`                  | 记录内核日志，包括内核启动信息和运行时的错误。               |
| `/var/log/mail.log`                  | 记录邮件系统的日志，包括邮件传输信息。                       |
| `/var/log/mail.err`                  | 记录邮件系统的错误日志。                                     |
| `/var/log/secure`                    | 记录安全相关的日志，包括登录、认证和授权信息。               |
| `/var/log/audit/audit.log`           | 记录系统审计日志，包括SELinux和其他安全模块的事件。          |
| `/var/log/sudo.log`                  | 记录sudo命令的使用情况。                                     |
| `/tmp/`                              | 临时文件目录，存储系统和用户的临时文件。                     |
| `/var/tmp/`                          | 临时文件目录，存储系统和用户的临时文件，通常比/tmp的生命周期更长。 |
| `/home/*/.bash_history`              | 记录每个用户的bash历史命令。                                 |
| `/home/*/.zsh_history`               | 记录每个用户的zsh历史命令。                                  |
| `/root/.bash_history`                | 记录root用户的bash历史命令。                                 |
| `/root/.zsh_history`                 | 记录root用户的zsh历史命令。                                  |
| `/var/log/sshd.log`                  | 记录SSH守护进程的日志。                                      |
| `/var/run/utmp`                      | 记录当前登录用户的信息。                                     |
| `/var/log/dmesg`                     | 记录内核环形缓冲区的消息，通常包括系统启动信息。             |
| `/var/log/yum.log`                   | 记录Yum包管理器的操作日志。                                  |
| `/var/log/apt/history.log`           | 记录APT包管理器的历史日志。                                  |
| `/var/log/apt/term.log`              | 记录APT包管理器的终端日志，包括包安装和卸载的信息。          |
| `/var/log/journal/`                  | systemd的日志目录，记录所有使用systemd管理的服务和系统事件。 |
| `/var/log/rkhunter/rkhunter.log`     | 记录Rootkit Hunter的扫描日志。                               |
| `/var/log/chkrootkit/chkrootkit.log` | 记录Chkrootkit工具的扫描日志。                               |
| `/var/log/docker.log`                | 记录Docker的日志，包括容器的启动和停止信息。                 |
| `/var/log/kubernetes/`               | 记录Kubernetes的日志，包括集群中各组件的事件。               |
| `/var/log/containers/`               | 记录容器的日志，通常包括Docker和Kubernetes管理的容器。       |
| `/var/log/postgresql/`               | 记录PostgreSQL数据库的日志。                                 |
| `/var/log/mongodb/mongod.log`        | 记录MongoDB数据库的日志，包括启动和运行时信息。              |
| `/var/log/redis/redis.log`           | 记录Redis数据库的日志。                                      |
| `/var/log/tomcat/`                   | 记录Tomcat应用服务器的日志，包括访问和错误日志。             |
| `/var/log/glassfish/`                | 记录GlassFish应用服务器的日志，包括访问和错误日志。          |
| `/var/log/maillog`                   | 记录邮件传输代理（MTA）的日志，包括邮件传输信息。            |
| `/var/log/mail.err`                  | 记录邮件传输代理（MTA）的错误日志。                          |
| `/var/log/firewalld`                 | 记录Firewalld的日志，包括防火墙规则的应用和事件。            |
| `/var/log/iptables.log`              | 记录Iptables的日志，包括防火墙规则的应用和事件。             |
| `/var/log/samba/*`                   | 记录Samba服务的日志，包括文件共享和打印服务的事件。          |
| `/var/log/rsyncd.log`                | 记录Rsync守护进程的日志，包括文件同步和传输的事件。          |
| `/var/log/wtmp.1`                    | 记录历史登录和注销事件的归档日志。                           |
| `/var/log/btmp.1`                    | 记录历史失败的登录尝试的归档日志。                           |
