package module

import (
	"fmt"
	"io/ioutil"
	"os/user"
	"path/filepath"
)

// DeleteWindowsIntrusionTraces 删除Windows系统的入侵痕迹
func DeleteWindowsIntrusionTraces() {
	folders := GetWindowsFilePaths()

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

	// 第五阶段：提供不同的处理方式
	deleteAllFolders(smallFolders, largeFolders)
}

// DeleteWindowsLargeKeepSmall 删除大文件夹，保留小文件夹
func DeleteWindowsLargeKeepSmall() {
	folders := GetWindowsFilePaths()

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

// RestoreWindowsSmallFolders 恢复小文件夹
func RestoreWindowsSmallFolders() {
	restoreSmallFolders()
}

// GetWindowsFilePaths 动态获取重要文件夹和文件的路径
func GetWindowsFilePaths() []string {
	currentUser, err := user.Current()
	if err != nil {
		fmt.Printf("获取当前用户失败: %v\n", err)
		return nil
	}
	username := currentUser.Username

	userProfilePath := filepath.Join("C:\\Users", username, "AppData\\Local\\Mozilla\\Firefox\\Profiles")
	profiles, err := ioutil.ReadDir(userProfilePath)
	if err != nil {
		fmt.Printf("读取Firefox配置文件夹失败: %v\n", err)
		return nil
	}

	var firefoxProfilePaths []string
	for _, profile := range profiles {
		if profile.IsDir() {
			profilePath := filepath.Join(userProfilePath, profile.Name(), "cache2")
			placesPath := filepath.Join(userProfilePath, profile.Name(), "places.sqlite")
			firefoxProfilePaths = append(firefoxProfilePaths, profilePath, placesPath)
		}
	}

	return []string{
		"C:\\Windows\\System32\\winevt\\Logs\\Security.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Application.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\System.evtx",
		"C:\\Program Files\\Apache Group\\Apache2\\logs\\access.log",
		"C:\\Program Files\\Apache Group\\Apache2\\logs\\error.log",
		"C:\\Program Files (x86)\\IIS Express\\Logs\\IISExpress.log",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-User Profile Service%4Operational.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-DNS-Client%4Operational.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-DNS-Server%4Analytical.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-Windows Firewall With Advanced Security%4Firewall.evtx",
		"C:\\Windows\\System32\\LogFiles\\Firewall\\pfirewall.log",
		"C:\\Windows\\System32\\LogFiles\\W3SVC1\\",
		"C:\\Windows\\System32\\LogFiles\\HTTPERR\\httperr1.log",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-Security-Auditing.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-TaskScheduler%4Operational.evtx",
		"C:\\Windows\\Temp",
		filepath.Join("C:\\Users", username, "AppData\\Local\\Temp"),
		filepath.Join("C:\\Users", username, "AppData\\LocalLow\\Temp"),
		filepath.Join("C:\\Users", username, "AppData\\Roaming\\Microsoft\\Windows\\Recent"),
		filepath.Join("C:\\Users", username, "AppData\\Local\\Microsoft\\Windows\\INetCache"),
		filepath.Join("C:\\Users", username, "AppData\\Local\\Microsoft\\Windows\\History"),
		filepath.Join("C:\\Users", username, "Documents"),
		filepath.Join("C:\\Users", username, "Downloads"),
		"C:\\inetpub\\logs\\LogFiles\\W3SVC1\\",
		"C:\\inetpub\\logs\\FailedReqLogFiles\\",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-SMBClient\\Operational.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-RemoteDesktopServices-RdpCoreTS\\Operational.evtx",
		"C:\\Windows\\System32\\winevt\\Logs\\Microsoft-Windows-TerminalServices-LocalSessionManager\\Operational.evtx",
	}
}
