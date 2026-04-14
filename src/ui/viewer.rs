use crate::app::SaveParserApp;
use eframe::egui;
use serde_json::Value;
use std::path::PathBuf;

pub fn show(ui: &mut egui::Ui, app: &mut SaveParserApp) {
    // === 顶部工具条：路径输入 + 浏览 ===
    ui.horizontal(|ui| {
        ui.label("路径:");
        let avail = ui.available_width();
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.path_input)
                .desired_width((avail - 100.0).max(100.0)),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let p = PathBuf::from(app.path_input.trim());
            if p.exists() {
                app.pending_load = Some(p);
            } else {
                app.status = "路径不存在".to_string();
            }
        }
        if ui.button("📂 浏览").clicked() {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Save files", &["save"])
                .add_filter("All files", &["*"])
                .pick_file()
            {
                app.pending_load = Some(p);
            }
        }
    });

    // === 拖拽提示条（紧凑高度）===
    let drop_text = if app.current_json.is_none() {
        "⬇ 拖拽 .save 文件到此处，或从左侧选择"
    } else {
        "⬇ 拖入新文件可替换"
    };
    egui::Frame::none()
        .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY))
        .rounding(4.0)
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(drop_text).weak());
            });
        });

    ui.separator();

    if app.current_json.is_none() {
        ui.label("未加载文件");
        return;
    }

    // === 左右分栏：右侧固定宽，左侧填充 ===
    // 用 child_ui + 手动切割比 SidePanel 嵌套更可控
    let total = ui.available_size();
    let right_width = (total.x * 0.42).clamp(280.0, 560.0);
    let left_width = total.x - right_width - 8.0;

    ui.horizontal_top(|ui| {
        // ----- 左：原始 JSON 树 -----
        ui.allocate_ui_with_layout(
            egui::vec2(left_width, total.y),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new("📄 原始 JSON").strong());
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("json_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if let Some(json) = app.current_json.as_mut() {
                            let mut changed = false;
                            render_value(ui, "root", json, &mut changed, 0);
                            if changed {
                                app.dirty = true;
                            }
                        }
                    });
            },
        );

        ui.separator();

        // ----- 右：结构化视图 -----
        ui.allocate_ui_with_layout(
            egui::vec2(right_width, total.y),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new("🎮 结构化数据").strong());
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("struct_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut changed = false;
                        if let Some(json) = app.current_json.as_mut() {
                            crate::ui::structured::show(ui, json, &mut changed);
                        }
                        if changed {
                            app.dirty = true;
                        }
                    });
            },
        );
    });
}

// render_value 不变
fn render_value(ui: &mut egui::Ui, key: &str, value: &mut Value, changed: &mut bool, depth: usize) {
    match value {
        Value::Object(map) => {
            egui::CollapsingHeader::new(format!("📦 {} {{{}}}", key, map.len()))
                .id_salt(format!("{}-{}", depth, key))
                .show(ui, |ui| {
                    for (k, v) in map.iter_mut() {
                        render_value(ui, k, v, changed, depth + 1);
                    }
                });
        }
        Value::Array(arr) => {
            egui::CollapsingHeader::new(format!("📋 {} [{}]", key, arr.len()))
                .id_salt(format!("{}-{}-arr", depth, key))
                .show(ui, |ui| {
                    for (i, v) in arr.iter_mut().enumerate() {
                        render_value(ui, &format!("[{}]", i), v, changed, depth + 1);
                    }
                });
        }
        Value::String(s) => {
            ui.horizontal(|ui| {
                ui.label(format!("🔤 {}:", key));
                if ui
                    .add(egui::TextEdit::singleline(s).desired_width(300.0))
                    .changed()
                {
                    *changed = true;
                }
            });
        }
        Value::Number(_) => {
            let original = if let Value::Number(n) = &value {
                n.to_string()
            } else {
                unreachable!()
            };
            let mut buf = original.clone();
            let mut response_changed = false;

            ui.horizontal(|ui| {
                ui.label(format!("🔢 {}:", key));
                if ui
                    .add(egui::TextEdit::singleline(&mut buf).desired_width(150.0))
                    .changed()
                {
                    response_changed = true;
                }
            });

            if response_changed && buf != original {
                if let Ok(parsed) = buf.parse::<i64>() {
                    *value = Value::Number(parsed.into());
                    *changed = true;
                } else if let Ok(parsed) = buf.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(parsed) {
                        *value = Value::Number(num);
                        *changed = true;
                    }
                }
            }
        }
        Value::Bool(b) => {
            ui.horizontal(|ui| {
                ui.label(format!("☑ {}:", key));
                if ui.checkbox(b, "").changed() {
                    *changed = true;
                }
            });
        }
        Value::Null => {
            ui.label(format!("∅ {}: null", key));
        }
    }
}