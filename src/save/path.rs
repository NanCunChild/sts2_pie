use serde_json::Value;

/// JSON 路径中的一段：要么是对象 key，要么是数组下标
#[derive(Clone, Copy, Debug)]
pub enum Seg<'a> {
    K(&'a str),
    I(usize),
}

/// 按路径深入 JSON，返回不可变引用
pub fn get<'a>(json: &'a Value, path: &[Seg]) -> Option<&'a Value> {
    let mut cur = json;
    for seg in path {
        cur = match seg {
            Seg::K(k) => cur.as_object()?.get(*k)?,
            Seg::I(i) => cur.as_array()?.get(*i)?,
        };
    }
    Some(cur)
}

/// 按路径深入 JSON，返回可变引用
pub fn get_mut<'a>(json: &'a mut Value, path: &[Seg]) -> Option<&'a mut Value> {
    let mut cur = json;
    for seg in path {
        cur = match seg {
            Seg::K(k) => cur.as_object_mut()?.get_mut(*k)?,
            Seg::I(i) => cur.as_array_mut()?.get_mut(*i)?,
        };
    }
    Some(cur)
}

/// 把路径格式化成可读字符串，用于错误提示
pub fn format_path(path: &[Seg]) -> String {
    let mut s = String::new();
    for (idx, seg) in path.iter().enumerate() {
        match seg {
            Seg::K(k) => {
                if idx > 0 { s.push('.'); }
                s.push_str(k);
            }
            Seg::I(i) => {
                s.push_str(&format!("[{}]", i));
            }
        }
    }
    s
}