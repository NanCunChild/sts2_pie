use glob::glob;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub profile: String,    // "profile1" / "profile2" ...
    pub kind: SaveKind,     // 单人 / 多人
    pub source: SaveSource, // Steam 原生 / Proton / Wine
}

#[derive(Debug, Clone)]
pub enum SaveKind {
    SinglePlayer,
    Multiplayer,
}

#[derive(Debug, Clone)]
pub enum SaveSource {
    WindowsNative,
    ProtonOfficial,
    ProtonPirated,
    Wine,
    GoldbergEmu,
}

pub fn discover_all() -> Vec<SaveEntry> {
    let mut results = Vec::new();
    #[cfg(target_os = "windows")]
    results.extend(discover_windows());
    #[cfg(target_os = "linux")]
    results.extend(discover_linux());
    results
}

fn build_patterns() -> Vec<(String, SaveSource)> {
    // 注意：glob 的 ** 可以匹配任意层；* 匹配单层任意字符
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut patterns = vec![];

    #[cfg(target_os = "windows")]
    if let Some(appdata) = dirs::config_dir() {
        // config_dir() 在 Windows 上返回 %APPDATA% (Roaming)
        let base = appdata.to_string_lossy();
        patterns.push((
            format!("{base}/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::WindowsNative,
        ));
    }

    #[cfg(target_os = "linux")]
    {
        // Proton 正版
        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/2868840/pfx/drive_c/users/steamuser/AppData/Roaming/SlayTheSpire 2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::ProtonOfficial,
        ));
        // Proton 盗版（10 位随机 ID）
        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/pfx/drive_c/users/steamuser/AppData/Roaming/SlayTheSpire*2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::ProtonPirated,
        ));
        // Wine
        patterns.push((
            format!("{home}/.wine/drive_c/users/*/AppData/Roaming/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::Wine,
        ));
        
        patterns.push((
        format!("{home}/.local/share/Steam/steamapps/compatdata/2868840/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile*/saves/current_run*.save"),
        SaveSource::GoldbergEmu,
    ));
        // 盗版 10 位随机 ID compatdata
        patterns.push((
        format!("{home}/.local/share/Steam/steamapps/compatdata/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile*/saves/current_run*.save"),
        SaveSource::GoldbergEmu,
    ));
        // Wine prefix
        patterns.push((
        format!("{home}/.wine/drive_c/users/*/AppData/Roaming/GSE Saves/2868840/remote/profile*/saves/current_run*.save"),
        SaveSource::GoldbergEmu,
    ));
    }

    patterns
}

fn scan(patterns: Vec<(String, SaveSource)>) -> Vec<SaveEntry> {
    let mut out = Vec::new();
    for (pat, source) in patterns {
        if let Ok(paths) = glob(&pat) {
            for entry in paths.flatten() {
                let kind = if entry
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains("_mp"))
                    .unwrap_or(false)
                {
                    SaveKind::Multiplayer
                } else {
                    SaveKind::SinglePlayer
                };

                let profile = entry
                    .components()
                    .filter_map(|c| c.as_os_str().to_str())
                    .find(|s| s.starts_with("profile"))
                    .unwrap_or("?")
                    .to_string();

                out.push(SaveEntry {
                    path: entry,
                    profile,
                    kind,
                    source: source.clone(),
                });
            }
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn discover_windows() -> Vec<SaveEntry> {
    scan(build_patterns())
}

#[cfg(target_os = "linux")]
fn discover_linux() -> Vec<SaveEntry> {
    scan(build_patterns())
}
