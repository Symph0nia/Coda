package module

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

// 文件夹大小限制，单位为字节（10MB）
const folderSizeLimit = 100 * 1024 * 1024

// GetOSType 确定操作系统类型
func GetOSType() string {
	osType := runtime.GOOS
	fmt.Printf("操作系统类型: %s\n", osType)
	return osType
}

// CheckPermissions 确认是否具有足够的权限
func CheckPermissions() bool {
	osType := GetOSType()
	if osType == "windows" {
		return checkWindowsPermissions()
	} else if osType == "linux" {
		return checkLinuxPermissions()
	} else {
		fmt.Println("不支持的操作系统类型。")
		return false
	}
}

// checkWindowsPermissions 检查Windows系统权限
func checkWindowsPermissions() bool {
	if isWindowsAdmin() {
		fmt.Println("权限确认: 当前用户具有管理员权限。")
		return true
	} else {
		fmt.Println("权限确认: 当前用户不具有管理员权限。")
		return false
	}
}

// checkLinuxPermissions 检查Linux系统权限
func checkLinuxPermissions() bool {
	if os.Geteuid() == 0 {
		fmt.Println("权限确认: 当前用户具有root权限。")
		return true
	} else {
		fmt.Println("权限确认: 当前用户不具有root权限。")
		return false
	}
}

// isWindowsAdmin 检查是否具有Windows管理员权限
func isWindowsAdmin() bool {
	cmd := exec.Command("net", "session")
	err := cmd.Run()
	if err != nil {
		// 进一步检查错误信息
		exitError, ok := err.(*exec.ExitError)
		if ok {
			switch exitError.ExitCode() {
			case 5:
				fmt.Println("权限确认: 访问被拒绝。可能没有管理员权限。")
			case 53:
				fmt.Println("权限确认: 找不到网络路径。")
			default:
				fmt.Printf("权限确认: 检查管理员权限时发生错误，错误代码: %d\n", exitError.ExitCode())
			}
		} else {
			fmt.Println("权限确认: 检查管理员权限时发生未知错误。")
		}
		return false
	}
	return true
}

// deleteLargeKeepSmall 删除大文件夹，备份小文件夹并记录原始路径
func deleteLargeKeepSmall(smallFolders, largeFolders []string) {
	tempDir := filepath.Join(os.TempDir(), "folder_backups")
	if err := os.MkdirAll(tempDir, 0755); err != nil {
		fmt.Printf("创建备份目录失败: %v\n", err)
		return
	}

	// 创建文件记录备份路径
	pathRecordFile := filepath.Join(tempDir, "backup_paths.txt")
	file, err := os.Create(pathRecordFile)
	if err != nil {
		fmt.Printf("创建备份路径记录文件失败: %v\n", err)
		return
	}
	defer file.Close()

	// 删除大文件夹
	for _, folder := range largeFolders {
		if err := os.RemoveAll(folder); err != nil {
			fmt.Printf("删除大文件夹失败: %s, 错误: %v\n", folder, err)
		} else {
			fmt.Printf("成功删除大文件夹: %s\n", folder)
		}
	}

	// 备份小文件夹并删除原文件夹
	for _, folder := range smallFolders {
		backupFolder := filepath.Join(tempDir, filepath.Base(folder))
		if err := os.Rename(folder, backupFolder); err != nil {
			fmt.Printf("备份文件夹失败: %s, 错误: %v\n", folder, err)
		} else {
			fmt.Fprintf(file, "%s,%s\n", backupFolder, folder) // 记录备份文件夹和原始路径
			fmt.Printf("成功备份并删除小文件夹: %s 到 %s\n", folder, backupFolder)
		}
	}
}

// restoreSmallFolders 从Temp目录恢复备份的小文件夹到原位
func restoreSmallFolders() {
	tempDir := filepath.Join(os.TempDir(), "folder_backups")
	pathRecordFile := filepath.Join(tempDir, "backup_paths.txt")

	// 读取备份路径记录
	content, err := ioutil.ReadFile(pathRecordFile)
	if err != nil {
		fmt.Printf("读取备份路径记录文件失败: %v\n", err)
		return
	}

	lines := strings.Split(string(content), "\n")
	for _, line := range lines {
		if line == "" {
			continue
		}
		parts := strings.Split(line, ",")
		if len(parts) != 2 {
			fmt.Println("备份路径记录格式错误")
			continue
		}
		backupFolder := parts[0]
		originalPath := parts[1]

		// 恢复文件夹
		if err := os.Rename(backupFolder, originalPath); err != nil {
			fmt.Printf("恢复文件夹失败: %s 到 %s, 错误: %v\n", backupFolder, originalPath, err)
		} else {
			fmt.Printf("成功恢复文件夹: %s 到 %s\n", backupFolder, originalPath)
		}
	}
}

// deleteAllFolders 删除所有文件夹
func deleteAllFolders(smallFolders, largeFolders []string) {
	allFolders := append(smallFolders, largeFolders...)
	for _, folder := range allFolders {
		if err := os.RemoveAll(folder); err != nil {
			fmt.Printf("删除文件夹失败: %s, 错误: %v\n", folder, err)
		} else {
			fmt.Printf("成功删除文件夹: %s\n", folder)
		}
	}
}

// checkPathsExist
func checkPathsExist(paths []string) []string {
	existingPaths := []string{}
	for _, path := range paths {
		if _, err := os.Stat(path); err == nil {
			existingPaths = append(existingPaths, path)
		} else {
			fmt.Printf("路径不存在: %s\n", path)
		}
	}
	return existingPaths
}

// checkFoldersHaveFiles
func checkPathContents(paths []string) []string {
	nonEmptyPaths := []string{}
	for _, path := range paths {
		info, err := os.Stat(path)
		if err != nil {
			continue
		}
		if info.IsDir() {
			files, err := ioutil.ReadDir(path)
			if err == nil && len(files) > 0 {
				nonEmptyPaths = append(nonEmptyPaths, path)
			}
		} else {
			nonEmptyPaths = append(nonEmptyPaths, path) // 文件总是"非空"
		}
	}
	return nonEmptyPaths
}

// checkFoldersPermissions 检查文件夹的删除权限
func checkFoldersPermissions(folders []string) []string {
	deletableFolders := []string{}
	for _, folder := range folders {
		tempFile := filepath.Join(folder, "tempfile.tmp")
		if file, err := os.Create(tempFile); err == nil {
			file.Close()
			os.Remove(tempFile)
			deletableFolders = append(deletableFolders, folder)
		} else {
			fmt.Printf("无法在文件夹中创建临时文件，无权限: %s\n", folder)
		}
	}
	return deletableFolders
}

// classifyPathsBySize
func classifyPathsBySize(paths []string) ([]string, []string) {
	smallPaths := []string{}
	largePaths := []string{}
	for _, path := range paths {
		size := calculatePathSize(path)
		if size < folderSizeLimit {
			smallPaths = append(smallPaths, path)
		} else {
			largePaths = append(largePaths, path)
		}
	}
	return smallPaths, largePaths
}

// calculatePathSize 现在处理文件和文件夹
func calculatePathSize(path string) int64 {
	var size int64
	filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
		if err == nil {
			size += info.Size()
		}
		return nil
	})
	return size
}
