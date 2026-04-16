use crate::save::path::{format_path, get, get_mut, Seg};
use eframe::egui;
use serde_json::Value;

use Seg::{I, K};

pub fn show(ui: &mut egui::Ui, json: &mut Value, changed: &mut bool) {
    // ===== 顶层 =====
    section(ui, "🎯 运行信息");
    field_i64(ui, json, &[K("ascension")], "升天等级", changed);
    field_i64(ui, json, &[K("current_act_index")], "当前章节索引 (0=第1章)", changed);
    field_i64(ui, json, &[K("schema_version")], "存档版本", changed);
    field_string(ui, json, &[K("platform_type")], "平台", changed);

    ui.add_space(8.0);

    // ===== 玩家基础 =====
    section(ui, "👤 玩家");
    field_string(ui, json, &[K("players"), I(0), K("character_id")], "角色", changed);
    field_i64(ui, json, &[K("players"), I(0), K("current_hp")], "当前生命", changed);
    field_i64(ui, json, &[K("players"), I(0), K("max_hp")], "最大生命", changed);
    field_i64(ui, json, &[K("players"), I(0), K("gold")], "金币", changed);
    field_i64(ui, json, &[K("players"), I(0), K("max_energy")], "最大能量", changed);
    field_i64(ui, json, &[K("players"), I(0), K("max_potion_slot_count")], "药水槽数量", changed);

    ui.add_space(8.0);

    // ===== 派生信息（只读）=====
    section(ui, "📊 派生信息");
    derived_info(ui, json);

    ui.add_space(8.0);

    // ===== 时间 =====
    section(ui, "⏱ 时间");
    field_unix_time(ui, json, &[K("start_time")], "开始时间", changed);
    field_unix_time(ui, json, &[K("save_time")], "上次保存", changed);
    field_i64(ui, json, &[K("run_time")], "已用时长 (秒)", changed);

    ui.add_space(8.0);

    // ===== 简要清单（只读，后续做增删改）=====
    section(ui, "📦 简要清单 (只读预览)");
    counts_summary(ui, json);
}

// ============================================================
// 派生信息计算
// ============================================================

fn derived_info(ui: &mut egui::Ui, json: &Value) {
    egui::Grid::new("derived_grid")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            // 当前楼层 ≈ visited_map_coords 的长度
            if let Some(arr) = get(json, &[K("visited_map_coords")]).and_then(|v| v.as_array()) {
                ui.label("已访问节点数");
                ui.label(format!("{}", arr.len()));
                ui.end_row();
            }

            // 牌组大小
            if let Some(arr) = get(json, &[K("players"), I(0), K("deck")]).and_then(|v| v.as_array()) {
                ui.label("牌组数量");
                ui.label(format!("{} 张", arr.len()));
                ui.end_row();
            }

            // 遗物数
            if let Some(arr) = get(json, &[K("players"), I(0), K("relics")]).and_then(|v| v.as_array()) {
                ui.label("遗物数量");
                ui.label(format!("{} 件", arr.len()));
                ui.end_row();
            }

            // 药水数
            if let Some(arr) = get(json, &[K("players"), I(0), K("potions")]).and_then(|v| v.as_array()) {
                ui.label("药水数量");
                ui.label(format!("{} 瓶", arr.len()));
                ui.end_row();
            }

            // 升级牌数
            if let Some(arr) = get(json, &[K("players"), I(0), K("deck")]).and_then(|v| v.as_array()) {
                let upgraded = arr.iter()
                    .filter(|c| c.get("current_upgrade_level").is_some())
                    .count();
                let enchanted = arr.iter()
                    .filter(|c| c.get("enchantment").is_some())
                    .count();
                ui.label("已升级 / 已附魔");
                ui.label(format!("{} / {}", upgraded, enchanted));
                ui.end_row();
            }
        });
}

fn counts_summary(ui: &mut egui::Ui, json: &Value) {
    // 牌组列表
    if let Some(arr) = get(json, &[K("players"), I(0), K("deck")]).and_then(|v| v.as_array()) {
        egui::CollapsingHeader::new(format!("🃏 牌组 ({})", arr.len()))
            .id_salt("deck_preview")
            .show(ui, |ui| {
                for card in arr {
                    let id = card.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let upgrade = card.get("current_upgrade_level")
                        .and_then(|v| v.as_i64())
                        .map(|n| format!("+{}", n))
                        .unwrap_or_default();
                    let enchant = card.get("enchantment")
                        .and_then(|e| e.get("id"))
                        .and_then(|v| v.as_str())
                        .map(|s| format!(" 🔮{}", strip_prefix(s, "ENCHANTMENT.")))
                        .unwrap_or_default();
                    ui.label(format!("• {} {}{}", strip_prefix(id, "CARD."), upgrade, enchant));
                }
            });
    }

    // 遗物列表
    if let Some(arr) = get(json, &[K("players"), I(0), K("relics")]).and_then(|v| v.as_array()) {
        egui::CollapsingHeader::new(format!("💎 遗物 ({})", arr.len()))
            .id_salt("relic_preview")
            .show(ui, |ui| {
                for relic in arr {
                    let id = relic.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    ui.label(format!("• {}", strip_prefix(id, "RELIC.")));
                }
            });
    }

    // 药水
    if let Some(arr) = get(json, &[K("players"), I(0), K("potions")]).and_then(|v| v.as_array()) {
        egui::CollapsingHeader::new(format!("🧪 药水 ({})", arr.len()))
            .id_salt("potion_preview")
            .show(ui, |ui| {
                for p in arr {
                    let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let slot = p.get("slot_index").and_then(|v| v.as_i64()).unwrap_or(-1);
                    ui.label(format!("• [槽{}] {}", slot, strip_prefix(id, "POTION.")));
                }
            });
    }
}

fn strip_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
}

// ============================================================
// 字段编辑器
// ============================================================

fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).strong().size(15.0));
    ui.separator();
}

fn field_i64(
    ui: &mut egui::Ui,
    json: &mut Value,
    path: &[Seg],
    label: &str,
    changed: &mut bool,
) {
    let Some(v) = get_mut(json, path) else {
        missing(ui, label, path);
        return;
    };
    let Some(mut n) = v.as_i64() else {
        type_mismatch(ui, label, "整数");
        return;
    };
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        if ui.add(egui::DragValue::new(&mut n).speed(1.0)).changed() {
            *v = Value::Number(n.into());
            *changed = true;
        }
    });
}

fn field_string(
    ui: &mut egui::Ui,
    json: &mut Value,
    path: &[Seg],
    label: &str,
    changed: &mut bool,
) {
    let Some(v) = get_mut(json, path) else {
        missing(ui, label, path);
        return;
    };
    let Some(s) = v.as_str() else {
        type_mismatch(ui, label, "字符串");
        return;
    };
    let mut buf = s.to_string();
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        if ui.add(egui::TextEdit::singleline(&mut buf).desired_width(260.0)).changed() {
            *v = Value::String(buf);
            *changed = true;
        }
    });
}

fn field_unix_time(
    ui: &mut egui::Ui,
    json: &mut Value,
    path: &[Seg],
    label: &str,
    changed: &mut bool,
) {
    let Some(v) = get_mut(json, path) else {
        missing(ui, label, path);
        return;
    };
    let Some(mut n) = v.as_i64() else {
        type_mismatch(ui, label, "整数时间戳");
        return;
    };
    let human = chrono::DateTime::<chrono::Local>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(n.max(0) as u64)
    ).format("%Y-%m-%d %H:%M:%S").to_string();

    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        if ui.add(egui::DragValue::new(&mut n).speed(1.0)).changed() {
            *v = Value::Number(n.into());
            *changed = true;
        }
        ui.weak(format!("({})", human));
    });
}

fn missing(ui: &mut egui::Ui, label: &str, path: &[Seg]) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.weak(format!("(未找到: {})", format_path(path)));
    });
}

fn type_mismatch(ui: &mut egui::Ui, label: &str, expected: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("{}:", label));
        ui.weak(format!("(类型不是{})", expected));
    });
}