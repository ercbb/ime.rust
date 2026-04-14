use std::collections::HashMap;

/// 自然码双拼方案映射
/// 自然码规则：
/// - 每个汉字用两个键表示
/// - 第一键 = 声母（zh=v, ch=i, sh=u）
/// - 第二键 = 韵母
/// - 零声母以 o 开头，后面跟韵母键

/// 声母映射表：按键 → 声母
fn get_initial_map() -> HashMap<char, &'static str> {
    let mut m = HashMap::new();
    // 标准声母
    m.insert('b', "b");
    m.insert('p', "p");
    m.insert('m', "m");
    m.insert('f', "f");
    m.insert('d', "d");
    m.insert('t', "t");
    m.insert('n', "n");
    m.insert('l', "l");
    m.insert('g', "g");
    m.insert('k', "k");
    m.insert('h', "h");
    m.insert('j', "j");
    m.insert('q', "q");
    m.insert('x', "x");
    m.insert('r', "r");
    m.insert('z', "z");
    m.insert('c', "c");
    m.insert('s', "s");
    m.insert('y', "y");
    m.insert('w', "w");
    // 特殊声母映射
    m.insert('v', "zh"); // v → zh
    m.insert('i', "ch"); // i → ch
    m.insert('u', "sh"); // u → sh
    m
}

/// 韵母映射表：按键 → 韵母（自然码方案）
fn get_final_map() -> HashMap<char, &'static str> {
    let mut m = HashMap::new();
    // 自然码韵母映射
    m.insert('q', "iu");
    m.insert('w', "ia");    // ia/ua
    m.insert('e', "e");
    m.insert('r', "uan");   // uan/üan
    m.insert('t', "ue");    // ue/üe
    m.insert('y', "uai");
    m.insert('u', "u");
    m.insert('i', "i");
    m.insert('o', "o");     // o/uo
    m.insert('p', "un");    // un/ün
    m.insert('a', "a");
    m.insert('s', "ong");   // ong/iong
    m.insert('d', "uang");  // uang/iang
    m.insert('f', "en");
    m.insert('g', "eng");
    m.insert('h', "ang");
    m.insert('j', "an");
    m.insert('k', "ao");
    m.insert('l', "ai");
    m.insert('z', "ou");
    m.insert('x', "ia");    // ia/ua → 实际上自然码 x→ie
    m.insert('c', "iao");
    m.insert('v', "zh");    // 在韵母位置时 v→ui (自然码特例)
    m.insert('b', "in");
    m.insert('n', "in");    // 自然码 n→in
    m.insert('m', "ian");
    // 补充映射
    m
}

// 重新按照正确的自然码方案定义
/// 自然码双拼方案（完整版）
/// 第一键声母、第二键韵母

/// 声母按键对照表
const INITIAL_TABLE: &[(char, &str)] = &[
    ('b', "b"), ('p', "p"), ('m', "m"), ('f', "f"),
    ('d', "d"), ('t', "t"), ('n', "n"), ('l', "l"),
    ('g', "g"), ('k', "k"), ('h', "h"),
    ('j', "j"), ('q', "q"), ('x', "x"),
    ('z', "z"), ('c', "c"), ('s', "s"), ('r', "r"),
    ('v', "zh"), ('i', "ch"), ('u', "sh"),
    ('y', "y"), ('w', "w"),
];

/// 韵母按键对照表（自然码方案）
const FINAL_TABLE: &[(char, &str)] = &[
    ('a', "a"),   ('b', "ou"),  ('c', "iao"), ('d', "iang"), ('e', "e"),
    ('f', "en"),  ('g', "eng"), ('h', "ang"), ('i', "i"),    ('j', "an"),
    ('k', "ao"),  ('l', "ai"),  ('m', "ian"), ('n', "in"),   ('o', "uo"),
    ('p', "un"),  ('q', "iu"),  ('r', "uan"), ('s', "ong"),  ('t', "uang"),
    ('u', "u"),   ('v', "ui"),  ('w', "ua"),  ('x', "ie"),   ('y', "uai"),
    ('z', "iu"),  // 自然码 z→iu
];

/// 获取声母
pub fn get_initial(key: char) -> Option<&'static str> {
    INITIAL_TABLE.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// 获取韵母
pub fn get_final(key: char) -> Option<&'static str> {
    FINAL_TABLE.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// 将自然码双拼编码转换为拼音音节
/// input: 双拼编码序列（每两个字母代表一个音节）
/// 返回: 拼音音节列表
pub fn parse_double_pinyin(input: &str) -> (Vec<String>, String) {
    let input = input.to_lowercase();
    let chars: Vec<char> = input.chars().collect();
    let mut result = Vec::new();

    let mut pos = 0;
    while pos + 1 < chars.len() {
        let initial_key = chars[pos];
        let final_key = chars[pos + 1];

        // 零声母情况：首键为 'o' 或声母键即韵母
        let initial = get_initial(initial_key);

        if let Some(init) = initial {
            let final_str = get_final(final_key);
            if let Some(fin) = final_str {
                let pinyin = combine_initial_final(init, fin);
                result.push(pinyin);
            } else {
                // 韵母键无效，可能是输入尚未完成
                let remaining: String = chars[pos..].iter().collect();
                return (result, remaining);
            }
        } else {
            // 不是有效的声母键
            let remaining: String = chars[pos..].iter().collect();
            return (result, remaining);
        }

        pos += 2;
    }

    // 处理剩余的单个字符（未完成输入）
    let remaining: String = chars[pos..].iter().collect();
    (result, remaining)
}

/// 将自然码双拼输入转换为拼音音节（用于实时输入）
/// 支持未完成的双拼编码
pub fn double_pinyin_to_syllables(input: &str) -> (Vec<String>, String) {
    parse_double_pinyin(input)
}

/// 获取双拼编码对应的拼音（两个字母一组）
pub fn double_pinyin_to_pinyin(code: &str) -> Option<String> {
    let code = code.to_lowercase();
    let chars: Vec<char> = code.chars().collect();

    if chars.len() != 2 {
        return None;
    }

    // 零声母：以 o 开头
    if chars[0] == 'o' {
        let fin = get_final(chars[1])?;
        return Some(fin.to_string());
    }

    let initial = get_initial(chars[0])?;
    let fin = get_final(chars[1])?;

    Some(combine_initial_final(initial, fin))
}

/// 获取双拼编码第一个键可能的韵母（用于联想/提示）
pub fn get_final_for_key(key: char) -> Option<&'static str> {
    get_final(key)
}

/// 获取双拼编码第一个键对应的声母提示
pub fn get_initial_for_key(key: char) -> Option<&'static str> {
    get_initial(key)
}

/// 合并声母和韵母
fn combine_initial_final(initial: &str, final_: &str) -> String {
    // 特殊处理规则
    match (initial, final_) {
        // j/q/x + u → j/q/x + ü
        ("j", "u") => "ju".to_string(),
        ("q", "u") => "qu".to_string(),
        ("x", "u") => "xu".to_string(),
        // j/q/x + un → j/q/x + ün (通过拼音规则处理)
        ("j", "un") => "jun".to_string(),
        ("q", "un") => "qun".to_string(),
        ("x", "un") => "xun".to_string(),
        // j/q/x + uan → j/q/x + üan
        ("j", "uan") => "juan".to_string(),
        ("q", "uan") => "quan".to_string(),
        ("x", "uan") => "xuan".to_string(),
        // j/q/x + ue → j/q/x + üe
        ("j", "ue") => "jue".to_string(),
        ("q", "ue") => "que".to_string(),
        ("x", "ue") => "xue".to_string(),
        // n/l + u → n/l + ü
        ("n", "u") => "nv".to_string(),
        ("l", "u") => "lv".to_string(),
        ("n", "ue") => "nve".to_string(),
        ("l", "ue") => "lve".to_string(),
        // 声母和韵母相同的情况
        ("", _) => final_.to_string(),
        (_, "i") if initial.len() > 1 => format!("{}i", initial), // zhi, chi, shi
        // 默认：声母+韵母
        (_, _) => format!("{}{}", initial, final_),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_pinyin_nihao() {
        // 自然码: ni → ni, hk → hao
        // n + i → ni, h + k → hao
        let (syllables, remaining) = double_pinyin_to_syllables("nihk");
        assert_eq!(syllables, vec!["ni", "hao"]);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_double_pinyin_zhongguo() {
        // 自然码: vs → zhong, go → guo
        // v + s → zh + ong = zhong, g + o → g + uo = guo
        let (syllables, remaining) = double_pinyin_to_syllables("vsgo");
        assert_eq!(syllables, vec!["zhong", "guo"]);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_double_pinyin_partial() {
        // 只输入一个字母
        let (syllables, remaining) = double_pinyin_to_syllables("n");
        assert!(syllables.is_empty());
        assert_eq!(remaining, "n");
    }

    #[test]
    fn test_double_pinyin_shuru() {
        // 自然码: uuiu → shuru? 不对，sh+u=shu, r+u=ru
        // u + u → sh + u = shu, r + u → r + u = ru
        let (syllables, _) = double_pinyin_to_syllables("uuru");
        assert_eq!(syllables, vec!["shu", "ru"]);
    }

    #[test]
    fn test_single_code() {
        // ni → ni (n声母 + i韵母=i → ni)
        assert_eq!(double_pinyin_to_pinyin("ni"), Some("ni".to_string()));
        // hk → h + ao = hao
        assert_eq!(double_pinyin_to_pinyin("hk"), Some("hao".to_string()));
    }
}
