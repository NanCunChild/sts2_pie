#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 

mod app;
mod discovery;
mod save;
mod ui;

use app::SaveParserApp;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "linux")]
    check_wayland_warning();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "STS2 Profile Info Editor",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(SaveParserApp::new()))
        }),
    )
}

#[cfg(target_os = "linux")]
fn check_wayland_warning() {
    if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland") 
        && std::env::var("WAYLAND_DISPLAY").is_ok()
    {
        eprintln!("⚠ 检测到 Wayland 会话。文件拖拽可能无法工作。");
        eprintln!("  解决方案 1: 使用左侧列表或「📂 浏览」按钮选择文件");
        eprintln!("  解决方案 2: 使用 X11 后端启动: WAYLAND_DISPLAY= ./sts2-pie");
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_data = load_system_cjk_font();
    if let Some(data) = font_data {
        fonts
            .font_data
            .insert("cjk".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "cjk".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("cjk".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_system_cjk_font() -> Option<Vec<u8>> {
    let candidates = [
        #[cfg(target_os = "windows")]
        "C:/Windows/Fonts/msyh.ttc",
        #[cfg(target_os = "windows")]
        "C:/Windows/Fonts/simhei.ttf",
        #[cfg(target_os = "linux")]
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        #[cfg(target_os = "linux")]
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        #[cfg(target_os = "linux")]
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ];
    candidates.iter().find_map(|p| std::fs::read(p).ok())
}
