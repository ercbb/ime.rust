use crate::ime::dict::Dictionary;

/// 加载核心字典（单字 + 通用词组 + 专业词组）
pub fn load_core_dict() -> Dictionary 
{
    let chars_data = include_str!("dicts/pinyin_chars.txt");
    let general_phrases = include_str!("dicts/phrases_expanded.txt");
    let geo_phrases = include_str!("dicts/phrases_geo.txt");
    Dictionary::new_with_expanded(chars_data, general_phrases, geo_phrases)
}
