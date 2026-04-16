use glob::glob;
use serde::Deserialize;
use std::fs;
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
    WindowsSteamCloud,
    MacNative, 
    MacSteamCloud,   // macOS Steam 云存档
    ProtonOfficial,
    ProtonPirated,
    Wine,            // 用于 Linux 的 Wine 和 macOS 的 CrossOver/Whisky
    GoldbergEmu,
    Custom,
}

impl SaveSource {
    pub fn label(&self) -> &'static str {
        match self {
            SaveSource::WindowsNative => "Windows 原生",
            SaveSource::WindowsSteamCloud => "Windows Steam 云存档",
            SaveSource::MacNative => "macOS 原生",
            SaveSource::MacSteamCloud => "macOS Steam 云存档",
            SaveSource::ProtonOfficial => "Proton (正版)",
            SaveSource::ProtonPirated => "Proton (盗版?)",
            SaveSource::Wine => "Wine / CrossOver",
            SaveSource::GoldbergEmu => "Goldberg (Steam 模拟)",
            SaveSource::Custom => "自定义配置路径",
        }
    }
}

#[derive(Deserialize, Default, Debug)]
struct Config {
    #[serde(default)]
    extra_scan_paths: Vec<String>,
}

fn load_config() -> Config {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop(); 
        exe_path.push("config.toml");
        
        if exe_path.exists() {
            if let Ok(content) = fs::read_to_string(&exe_path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    eprintln!("[Config] Loaded portable config from: {}", exe_path.display());
                    return config;
                }
            }
        }
    }

    if let Some(mut path) = dirs::config_dir() {
        path.push("sts2-pie");
        let config_file = path.join("config.toml");

        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        } else {
            if fs::create_dir_all(&path).is_ok() {
                let default_template = r#"# STS2-PIE (Profile Info Editor) 配置文件

# 额外扫描的存档路径列表 (支持通配符 * 和 ?)
# 可以使用 ~/ 代表当前系统用户的主目录
extra_scan_paths = [
    # 取消下面两行的注释来添加自定义路径（编译的时候这么做太怪了。。。真的需要这个功能吗？）
    # "~/Games/custom_sts2_prefix/drive_c/users/steamuser/AppData/Roaming/SlayTheSpire2/steam/*/profile*/saves/current_run*.save",
    # "/mnt/d/Backup/STS2_Saves/*/profile*/saves/current_run*.save",
]
"#;
                if fs::write(&config_file, default_template).is_ok() {
                    eprintln!("[Config] Generated default config at: {}", config_file.display());
                }
            }
        }
    }

    Config::default()
}

pub fn discover_all() -> Vec<SaveEntry> {
    scan(build_patterns())
}

fn build_patterns() -> Vec<(String, SaveSource)> {
    let mut patterns = vec![];
    let config = load_config();
    
    let home_str = dirs::home_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    // =============== Windows 平台 ===============
    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = dirs::config_dir() {
            let base = appdata.to_string_lossy();
            patterns.push((
                format!("{base}/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
                SaveSource::WindowsNative,
            ));
        }
        
        patterns.push((
            "C:/Program Files (x86)/Steam/userdata/*/2868840/remote/profile*/saves/current_run*.save".to_string(),
            SaveSource::WindowsSteamCloud,
        ));
    }

    // =============== macOS 平台 ===============
    #[cfg(target_os = "macos")]
    {
        let home = &home_str;

        // macOS Steam 云存档
        patterns.push((
            format!("{home}/Library/Application Support/Steam/userdata/*/2868840/remote/profile*/saves/current_run*.save"),
            SaveSource::MacSteamCloud,
        ));
        
        // macOS 原生本地应用数据
        patterns.push((
            format!("{home}/Library/Application Support/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::MacNative,
        ));

        // 兼容 Mac 玩家使用 CrossOver / Whisky 跑 Windows 游戏的情况
        patterns.push((
            format!("{home}/Library/Application Support/CrossOver/Bottles/*/drive_c/users/*/AppData/Roaming/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::Wine,
        ));
        patterns.push((
            format!("{home}/Library/Containers/com.isaacmarovitz.Whisky/Bottles/*/drive_c/users/*/AppData/Roaming/SlayTheSpire2/steam/*/profile*/saves/current_run*.save"),
            SaveSource::Wine,
        ));
    }

    // =============== Linux 平台 ===============
    #[cfg(target_os = "linux")]
    {
        let home = &home_str;

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

    for extra_path in config.extra_scan_paths {
        let expanded_path = if extra_path.starts_with("~/") {
            extra_path.replacen("~", &home_str, 1)
        } else {
            extra_path
        };
        patterns.push((expanded_path, SaveSource::Custom));
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