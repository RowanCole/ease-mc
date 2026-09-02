// 运行路径：应用数据统一放在可执行文件同级的目录中（绿色便携、自包含）。
use std::path::PathBuf;

/// 可执行文件所在目录
fn exe_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?;
    let parent = exe
        .parent()
        .ok_or_else(|| "获取可执行文件目录失败".to_string())?;
    Ok(parent.to_path_buf())
}

/// 游戏安装目录：与可执行文件同级的 game 目录
pub fn game_path() -> Result<PathBuf, String> {
    Ok(exe_dir()?.join("game"))
}

/// 运行时配置文件路径（首次运行由打包资源初始化到此处）
pub fn config_path() -> Result<PathBuf, String> {
    Ok(exe_dir()?.join("config.json"))
}
