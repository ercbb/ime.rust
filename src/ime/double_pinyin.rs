/// 自然码双拼方案映射
/// 自然码规则：
/// - 每个汉字用两个键表示
/// - 第一键 = 声母（zh=v, ch=i, sh=u）
/// - 第二键 = 韵母
/// - 零声母以 o 开头，后面跟韵母键

/// 声母按键对照表
const INITIAL_TABLE: &[(char, &str)] = &[
    ('b', "b"),
    ('p', "p"),
    ('m', "m"),
    ('f', "f"),
    ('d', "d"),
    ('t', "t"),
    ('n', "n"),
    ('l', "l"),
    ('g', "g"),
    ('k', "k"),
    ('h', "h"),
    ('j', "j"),
    ('q', "q"),
    ('x', "x"),
    ('z', "z"),
    ('c', "c"),
    ('s', "s"),
    ('r', "r"),
    ('v', "zh"),
    ('i', "ch"),
    ('u', "sh"),
    ('y', "y"),
    ('w', "w"),
];

/// 韵母按键对照表（自然码方案）
/// 部分键位对应双韵母，由 combine_initial_final() 根据声母自动消歧：
///   d → uang (j/q/x 后取 iang)
///   w → ia   (g/k/h/zh/ch/sh 后取 ua)
///   y → ing  (g/k/h/zh/ch/sh 后取 uai)
///   s → ong  (j/q/x 后取 iong)
///   o → uo   (b/p/m/f 后取 o)
const FINAL_TABLE: &[(char, &str)] = &[
    ('a', "a"),
    ('b', "ou"),
    ('c', "iao"),
    ('d', "uang"),
    ('e', "e"),
    ('f', "en"),
    ('g', "eng"),
    ('h', "ang"),
    ('i', "i"),
    ('j', "an"),
    ('k', "ao"),
    ('l', "ai"),
    ('m', "ian"),
    ('n', "in"),
    ('o', "uo"),
    ('p', "un"),
    ('q', "iu"),
    ('r', "uan"),
    ('s', "ong"),
    ('t', "ue"),
    ('u', "u"),
    ('v', "ui"),
    ('w', "ia"),
    ('x', "ie"),
    ('y', "ing"),
    ('z', "ei"),
];

/// 获取声母
pub fn get_initial(key: char) -> Option<&'static str> 
{
    INITIAL_TABLE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// 获取韵母
pub fn get_final(key: char) -> Option<&'static str> 
{
    FINAL_TABLE.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// 将自然码双拼编码转换为拼音音节
/// input: 双拼编码序列（每两个字母代表一个音节）
/// 返回: 拼音音节列表
pub fn parse_double_pinyin(input: &str) -> (Vec<String>, String) 
{
    let input = input.to_lowercase();
    let chars: Vec<char> = input.chars().collect();
    let mut result = Vec::new();

    let mut pos = 0;
    while pos + 1 < chars.len() 
    {
        let initial_key = chars[pos];
        let final_key = chars[pos + 1];

        // 零声母情况：首键为 'o' 或声母键即韵母
        let initial = get_initial(initial_key);

        if let Some(init) = initial 
        {
            let final_str = get_final(final_key);
            if let Some(fin) = final_str 
            {
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
pub fn double_pinyin_to_syllables(input: &str) -> (Vec<String>, String) 
{
    parse_double_pinyin(input)
}

/// 合并声母和韵母，处理自然码双韵母消歧及拼音拼写规则
fn combine_initial_final(initial: &str, final_: &str) -> String 
{
    match (initial, final_) 
    {
        // === d 键消歧: uang / iang ===
        // j/q/x + uang → iang
        ("j", "uang") => "jiang".to_string(),
        ("q", "uang") => "qiang".to_string(),
        ("x", "uang") => "xiang".to_string(),

        // === w 键消歧: ia / ua ===
        // g/k/h + ia → ua
        ("g", "ia") => "gua".to_string(),
        ("k", "ia") => "kua".to_string(),
        ("h", "ia") => "hua".to_string(),
        // zh/ch/sh + ia → ua
        ("zh", "ia") => "zhua".to_string(),
        ("ch", "ia") => "chua".to_string(),
        ("sh", "ia") => "shua".to_string(),

        // === y 键消歧: ing / uai ===
        // g/k/h + ing → uai
        ("g", "ing") => "guai".to_string(),
        ("k", "ing") => "kuai".to_string(),
        ("h", "ing") => "huai".to_string(),
        // zh/ch/sh + ing → uai
        ("zh", "ing") => "zhuai".to_string(),
        ("ch", "ing") => "chuai".to_string(),
        ("sh", "ing") => "shuai".to_string(),

        // === s 键消歧: ong / iong ===
        // j/q/x + ong → iong
        ("j", "ong") => "jiong".to_string(),
        ("q", "ong") => "qiong".to_string(),
        ("x", "ong") => "xiong".to_string(),

        // === o 键消歧: uo / o ===
        // b/p/m/f + uo → o (拼音拼写规则: 唇音后 uo 写作 o)
        ("b", "uo") => "bo".to_string(),
        ("p", "uo") => "po".to_string(),
        ("m", "uo") => "mo".to_string(),
        ("f", "uo") => "fo".to_string(),

        // === j/q/x 拼写规则: u → ü ===
        ("j", "u") => "ju".to_string(),
        ("q", "u") => "qu".to_string(),
        ("x", "u") => "xu".to_string(),
        ("j", "un") => "jun".to_string(),
        ("q", "un") => "qun".to_string(),
        ("x", "un") => "xun".to_string(),
        ("j", "uan") => "juan".to_string(),
        ("q", "uan") => "quan".to_string(),
        ("x", "uan") => "xuan".to_string(),
        ("j", "ue") => "jue".to_string(),
        ("q", "ue") => "que".to_string(),
        ("x", "ue") => "xue".to_string(),

        // === n/l + u/ue → nü/lü/nüe/lüe ===
        ("n", "u") => "nv".to_string(),
        ("l", "u") => "lv".to_string(),
        ("n", "ue") => "nve".to_string(),
        ("l", "ue") => "lve".to_string(),

        // === 其他规则 ===
        ("", _) => final_.to_string(),
        (_, "i") if initial.len() > 1 => format!("{}i", initial), // zhi, chi, shi
        (_, _) => format!("{}{}", initial, final_),
    }
}

#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn test_double_pinyin_nihao() 
    {
        // n + i → ni, h + k → hao
        let (syllables, remaining) = double_pinyin_to_syllables("nihk");
        assert_eq!(syllables, vec!["ni", "hao"]);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_double_pinyin_zhongguo() 
    {
        // v + s → zh + ong = zhong, g + o → g + uo = guo
        let (syllables, remaining) = double_pinyin_to_syllables("vsgo");
        assert_eq!(syllables, vec!["zhong", "guo"]);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_double_pinyin_partial() 
    {
        let (syllables, remaining) = double_pinyin_to_syllables("n");
        assert!(syllables.is_empty());
        assert_eq!(remaining, "n");
    }

    #[test]
    fn test_double_pinyin_shuru() 
    {
        // u + u → sh + u = shu, r + u → r + u = ru
        let (syllables, _) = double_pinyin_to_syllables("uuru");
        assert_eq!(syllables, vec!["shu", "ru"]);
    }

    #[test]
    fn test_single_code() 
    {
        let (s, _) = double_pinyin_to_syllables("ni");
        assert_eq!(s, vec!["ni"]);
        let (s, _) = double_pinyin_to_syllables("hk");
        assert_eq!(s, vec!["hao"]);
    }

    // === 修正后的自然码映射测试 ===

    #[test]
    fn test_final_z_ei() 
    {
        // z → ei: fz = f + ei = fei
        let (s, _) = double_pinyin_to_syllables("fz");
        assert_eq!(s, vec!["fei"]);
        let (s, _) = double_pinyin_to_syllables("lz");
        assert_eq!(s, vec!["lei"]);
    }

    #[test]
    fn test_final_t_ue() 
    {
        // t → ue: jt = jue (j/q/x 后 üe)
        let (s, _) = double_pinyin_to_syllables("jt");
        assert_eq!(s, vec!["jue"]);
        let (s, _) = double_pinyin_to_syllables("qt");
        assert_eq!(s, vec!["que"]);
        let (s, _) = double_pinyin_to_syllables("xt");
        assert_eq!(s, vec!["xue"]);
        // n/l 后 üe → nve/lve
        let (s, _) = double_pinyin_to_syllables("nt");
        assert_eq!(s, vec!["nve"]);
        let (s, _) = double_pinyin_to_syllables("lt");
        assert_eq!(s, vec!["lve"]);
    }

    #[test]
    fn test_final_d_uang_iang() 
    {
        // d → uang: gd = guang
        let (s, _) = double_pinyin_to_syllables("gd");
        assert_eq!(s, vec!["guang"]);
        let (s, _) = double_pinyin_to_syllables("kd");
        assert_eq!(s, vec!["kuang"]);
        let (s, _) = double_pinyin_to_syllables("hd");
        assert_eq!(s, vec!["huang"]);
        // j/q/x + uang → iang
        let (s, _) = double_pinyin_to_syllables("jd");
        assert_eq!(s, vec!["jiang"]);
        let (s, _) = double_pinyin_to_syllables("qd");
        assert_eq!(s, vec!["qiang"]);
        let (s, _) = double_pinyin_to_syllables("xd");
        assert_eq!(s, vec!["xiang"]);
    }

    #[test]
    fn test_final_w_ia_ua() 
    {
        // w → ia: jw = jia
        let (s, _) = double_pinyin_to_syllables("jw");
        assert_eq!(s, vec!["jia"]);
        let (s, _) = double_pinyin_to_syllables("qw");
        assert_eq!(s, vec!["qia"]);
        let (s, _) = double_pinyin_to_syllables("xw");
        assert_eq!(s, vec!["xia"]);
        let (s, _) = double_pinyin_to_syllables("dw");
        assert_eq!(s, vec!["dia"]);
        let (s, _) = double_pinyin_to_syllables("lw");
        assert_eq!(s, vec!["lia"]);
        // g/k/h/zh/ch/sh + ia → ua
        let (s, _) = double_pinyin_to_syllables("gw");
        assert_eq!(s, vec!["gua"]);
        let (s, _) = double_pinyin_to_syllables("kw");
        assert_eq!(s, vec!["kua"]);
        let (s, _) = double_pinyin_to_syllables("hw");
        assert_eq!(s, vec!["hua"]);
        let (s, _) = double_pinyin_to_syllables("vw");
        assert_eq!(s, vec!["zhua"]);
        let (s, _) = double_pinyin_to_syllables("iw");
        assert_eq!(s, vec!["chua"]);
        let (s, _) = double_pinyin_to_syllables("uw");
        assert_eq!(s, vec!["shua"]);
    }

    #[test]
    fn test_final_y_ing_uai() 
    {
        // y → ing: jy = jing
        let (s, _) = double_pinyin_to_syllables("jy");
        assert_eq!(s, vec!["jing"]);
        let (s, _) = double_pinyin_to_syllables("qy");
        assert_eq!(s, vec!["qing"]);
        let (s, _) = double_pinyin_to_syllables("xy");
        assert_eq!(s, vec!["xing"]);
        let (s, _) = double_pinyin_to_syllables("by");
        assert_eq!(s, vec!["bing"]);
        // g/k/h/zh/ch/sh + ing → uai
        let (s, _) = double_pinyin_to_syllables("gy");
        assert_eq!(s, vec!["guai"]);
        let (s, _) = double_pinyin_to_syllables("ky");
        assert_eq!(s, vec!["kuai"]);
        let (s, _) = double_pinyin_to_syllables("hy");
        assert_eq!(s, vec!["huai"]);
        let (s, _) = double_pinyin_to_syllables("vy");
        assert_eq!(s, vec!["zhuai"]);
        let (s, _) = double_pinyin_to_syllables("iy");
        assert_eq!(s, vec!["chuai"]);
        let (s, _) = double_pinyin_to_syllables("uy");
        assert_eq!(s, vec!["shuai"]);
    }

    #[test]
    fn test_final_s_ong_iong() 
    {
        // s → ong: gs = gong
        let (s, _) = double_pinyin_to_syllables("gs");
        assert_eq!(s, vec!["gong"]);
        let (s, _) = double_pinyin_to_syllables("ds");
        assert_eq!(s, vec!["dong"]);
        // j/q/x + ong → iong
        let (s, _) = double_pinyin_to_syllables("js");
        assert_eq!(s, vec!["jiong"]);
        let (s, _) = double_pinyin_to_syllables("qs");
        assert_eq!(s, vec!["qiong"]);
        let (s, _) = double_pinyin_to_syllables("xs");
        assert_eq!(s, vec!["xiong"]);
    }

    #[test]
    fn test_final_o_uo_o() 
    {
        // o → uo: go = guo
        let (s, _) = double_pinyin_to_syllables("go");
        assert_eq!(s, vec!["guo"]);
        // b/p/m/f + uo → o
        let (s, _) = double_pinyin_to_syllables("bo");
        assert_eq!(s, vec!["bo"]);
        let (s, _) = double_pinyin_to_syllables("po");
        assert_eq!(s, vec!["po"]);
        let (s, _) = double_pinyin_to_syllables("mo");
        assert_eq!(s, vec!["mo"]);
        let (s, _) = double_pinyin_to_syllables("fo");
        assert_eq!(s, vec!["fo"]);
    }
}
