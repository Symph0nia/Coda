package main

import (
	"Coda/module"
	"flag"
	"fmt"
)

// 文件夹大小限制，单位为字节（10MB）
const folderSizeLimit = 100 * 1024 * 1024

func main() {
	deleteFlag := flag.Bool("D", false, "删除全部文件")
	backupDeleteFlag := flag.Bool("B", false, "执行备份并删除操作")
	restoreFlag := flag.Bool("R", false, "恢复备份")
	flag.Parse()

	// 获取操作系统类型
	osType := module.GetOSType()

	// 检查是否具有足够的权限
	if !module.CheckPermissions() {
		fmt.Println("权限不足，无法执行清理操作。")
		return
	}

	// 根据操作系统类型调用相应的清理函数
	if osType != "linux" && osType != "windows" {
		fmt.Println("不支持的操作系统类型。")
		return
	}

	if osType == "linux" {
		fmt.Println("检测到Linux系统，开始清理入侵痕迹...")
		if *deleteFlag {
			fmt.Println("正在执行全部删除操作...")
			module.DeleteLinuxIntrusionTraces() // 假设这个函数内部会调用 deleteAllFolders
		} else if *backupDeleteFlag {
			fmt.Println("正在执行备份并删除操作...")
			module.DeleteLinuxLargeKeepSmall() // 你需要确保这个函数可从module包访问
		} else if *restoreFlag {
			fmt.Println("正在执行恢复操作...")
			module.RestoreLinuxSmallFolders() // 你需要确保这个函数可从module包访问
		} else {
			fmt.Println("未指定有效的操作模式。使用 -D, -B, 或 -R.")
		}
	} else if osType == "windows" {
		fmt.Println("检测到Windows系统，开始清理入侵痕迹...")
		if *deleteFlag {
			fmt.Println("正在执行全部删除操作...")
			module.DeleteWindowsIntrusionTraces() // 假设这个函数内部会调用 deleteAllFolders
		} else if *backupDeleteFlag {
			fmt.Println("正在执行备份并删除操作...")
			module.DeleteWindowsLargeKeepSmall() // 你需要确保这个函数可从module包访问
		} else if *restoreFlag {
			fmt.Println("正在执行恢复操作...")
			module.RestoreWindowsSmallFolders() // 你需要确保这个函数可从module包访问
		} else {
			fmt.Println("未指定有效的操作模式。使用 -D, -B, 或 -R.")
		}
	}
}
