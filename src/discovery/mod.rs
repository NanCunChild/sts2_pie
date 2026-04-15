use glob::glob;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub profile: String,
    pub kind: SaveKind,
    pub source: SaveSource,
}

#[derive(Debug, Clone)]
pub enum SaveKind {
    SinglePlayer,
    Multiplayer,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SaveSource {
    WindowsNative,
    ProtonOfficial,
    ProtonPirated,
    Wine,
    GoldbergEmu,
}

pub fn discover_all() -> Vec<SaveEntry> {
    scan(build_patterns())
}

fn build_patterns() -> Vec<(String, SaveSource)> {
    let mut patterns = vec![];

    #[cfg(target_os = "windows")]
    if let Some(appdata) = dirs::config_dir() {
        let base = appdata.to_string_lossy();
        patterns.push((
            format!("{base}/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::WindowsNative,
        ));
    }

    #[cfg(target_os = "linux")]
    {
        // 将 home 变量的作用域限制在 Linux 专属块内
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/2868840/pfx/drive_c/users/steamuser/AppData/Roaming/SlayTheSpire 2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::ProtonOfficial,
        ));
        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/pfx/drive_c/users/steamuser/AppData/Roaming/SlayTheSpire*2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::ProtonPirated,
        ));
        patterns.push((
            format!("{home}/.wine/drive_c/users/*/AppData/Roaming/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::Wine,
        ));
        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/2868840/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile*/saves/current_run*.save"),
            SaveSource::GoldbergEmu,
        ));
        patterns.push((
            format!("{home}/.local/share/Steam/steamapps/compatdata/[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]/pfx/drive_c/users/steamuser/AppData/Roaming/GSE Saves/2868840/remote/profile*/saves/current_run*.save"),
            SaveSource::GoldbergEmu,
        ));
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

// #[cfg(target_os = "windows")]
// fn discover_windows() -> Vec<SaveEntry> { ... }
// #[cfg(target_os = "linux")]
// fn discover_linux() -> Vec<SaveEntry> { ... }