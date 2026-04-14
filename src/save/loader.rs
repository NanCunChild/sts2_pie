use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_save(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("读取失败: {}", path.display()))?;

    // TODO: 如果发现 .save 是加密的，在这里加解密层
    // 先尝试当作明文 UTF-8 JSON
    let json: Value =
        serde_json::from_slice(&bytes).context("JSON 解析失败 —— 文件可能被加密或损坏")?;
    Ok(json)
}

pub fn save_with_backup(path: &Path, value: &Value) -> Result<PathBuf> {
    // 1. 备份原文件
    let backup_path = make_backup_path(path);
    if path.exists() {
        fs::copy(path, &backup_path)
            .with_context(|| format!("备份失败: {}", backup_path.display()))?;
    }

    // 2. 序列化（pretty 便于游戏读取调试，但有些游戏只接受紧凑格式 —— 按需调整）
    let serialized = serde_json::to_vec_pretty(value).context("序列化失败")?;

    // 3. 原子写入：先写 .tmp 再 rename
    let tmp_path = path.with_extension("save.tmp");
    fs::write(&tmp_path, &serialized)
        .with_context(|| format!("写入临时文件失败: {}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).with_context(|| format!("替换原文件失败: {}", path.display()))?;

    Ok(backup_path)
}

fn make_backup_path(path: &Path) -> PathBuf {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".bak.{}", ts));
    path.with_file_name(name)
}
