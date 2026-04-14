use crate::app::SaveParserApp;
use crate::discovery::{SaveKind, SaveSource};
use eframe::egui;

pub fn show(ui: &mut egui::Ui, app: &mut SaveParserApp) {
    ui.heading("发现的存档");
    ui.label(format!("共 {} 个", app.discovered.len()));
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 按 source 分组
        let mut by_source: std::collections::BTreeMap<String, Vec<_>> = Default::default();
        for entry in &app.discovered {
            by_source
                .entry(source_label(&entry.source).to_string())
                .or_default()
                .push(entry.clone());
        }

        for (label, entries) in by_source {
            egui::CollapsingHeader::new(label)
                .default_open(true)
                .show(ui, |ui| {
                    for entry in entries {
                        let kind_icon = match entry.kind {
                            SaveKind::SinglePlayer => "👤",
                            SaveKind::Multiplayer => "👥",
                        };
                        let filename = entry
                            .path
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "?".to_string());
                        let display = format!("{} {} - {}", kind_icon, entry.profile, filename);

                        let selected = app.selected_path.as_ref() == Some(&entry.path);
                        if ui.selectable_label(selected, &display).clicked() {
                            app.pending_load = Some(entry.path.clone());
                        }

                        ui.label(
                            egui::RichText::new(entry.path.display().to_string())
                                .small()
                                .weak(),
                        );
                    }
                });
        }
    });
}

fn source_label(s: &SaveSource) -> &'static str {
    match s {
        SaveSource::WindowsNative => "Windows 原生",
        SaveSource::ProtonOfficial => "Proton (正版)",
        SaveSource::ProtonPirated => "Proton (盗版?)",
        SaveSource::Wine => "Wine",
    }
}
