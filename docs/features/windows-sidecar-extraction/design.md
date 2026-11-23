# 设计：Windows sidecar 解压修复

- 状态：designed
- Windows 使用系统 tar 命令并以参数数组传递 ZIP 和目标目录；解压目录预先创建，非零退出码由 execFileSync 抛出。其他平台的 unzip 路径不变，现有的二进制与 drawtext 能力验证继续执行。
