use crate::discovery::{self, SaveEntry};
use crate::save::loader;
use eframe::egui;
use serde_json::Value;
use std::path::PathBuf;

pub struct SaveParserApp {
    pub discovered: Vec<SaveEntry>,
    pub selected_path: Option<PathBuf>,
    pub current_json: Option<Value>,
    pub path_input: String,
    pub status: String,
    pub dirty: bool,
    pub pending_load: Option<PathBuf>,
}

impl SaveParserApp {
    pub fn new() -> Self {
        Self {
            discovered: discovery::discover_all(),
            selected_path: None,
            current_json: None,
            path_input: String::new(),
            status: "就绪".to_string(),
            dirty: false,
            pending_load: None,
        }
    }

    fn load_path(&mut self, path: PathBuf) {
        eprintln!("[load_path] called with: {}", path.display());
        eprintln!(
            "[load_path] exists: {}, is_file: {}",
            path.exists(),
            path.is_file()
        );

        match loader::load_save(&path) {
            Ok(v) => {
                let preview = match &v {
                    Value::Object(m) => format!("Object with {} keys", m.len()),
                    Value::Array(a) => format!("Array with {} items", a.len()),
                    _ => format!("{:?}", v),
                };
                eprintln!("[load_path] OK: {}", preview);

                self.current_json = Some(v);
                self.selected_path = Some(path.clone());
                self.path_input = path.display().to_string();
                self.dirty = false;
                self.status = format!("已加载 ({}): {}", preview, path.display());
            }
            Err(e) => {
                eprintln!("[load_path] ERR: {:#}", e);
                self.status = format!("加载失败: {:#}", e);
                // 即使失败也更新路径输入，方便看到点击触发了
                self.path_input = format!("[失败] {}", path.display());
            }
        }
    }

    fn save_current(&mut self) {
        let (Some(path), Some(json)) = (&self.selected_path, &self.current_json) else {
            self.status = "没有可保存的内容".to_string();
            return;
        };
        match loader::save_with_backup(path, json) {
            Ok(bak) => {
                self.dirty = false;
                self.status = format!("已保存，备份: {}", bak.display());
            }
            Err(e) => {
                self.status = format!("保存失败: {:#}", e);
            }
        }
    }
}

impl eframe::App for SaveParserApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 不可变借用 ctx.input，所以先拿出来
        ctx.input(|i| {
            if !i.raw.hovered_files.is_empty() {
                eprintln!("[update] hovered_files: {}", i.raw.hovered_files.len());
            }
            if !i.raw.dropped_files.is_empty() {
                eprintln!("[update] dropped_files: {}", i.raw.dropped_files.len());
                for f in &i.raw.dropped_files {
                    eprintln!(
                        "  - path={:?}, name={:?}, bytes={:?}",
                        f.path,
                        f.name,
                        f.bytes.as_ref().map(|b| b.len())
                    );
                }
            }
        });

        let dropped: Option<PathBuf> =
            ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone()));
        if let Some(p) = dropped {
            eprintln!("[update] queueing dropped path: {}", p.display());
            self.pending_load = Some(p);
        }

        if let Some(p) = self.pending_load.take() {
            eprintln!("[update] processing pending_load: {}", p.display());
            self.load_path(p);
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("🔄 重新扫描").clicked() {
                    self.discovered = crate::discovery::discover_all();
                    self.status = format!("发现 {} 个存档", self.discovered.len());
                }
                let save_btn = egui::Button::new(if self.dirty {
                    "💾 保存*"
                } else {
                    "💾 保存"
                });
                if ui
                    .add_enabled(self.current_json.is_some(), save_btn)
                    .clicked()
                {
                    self.save_current();
                }
                ui.separator();
                ui.label(&self.status);
            });
        });

        egui::SidePanel::left("discovered")
            .default_width(360.0)
            .show(ctx, |ui| {
                crate::ui::sidebar::show(ui, self);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::ui::viewer::show(ui, self);
        });
    }
}
