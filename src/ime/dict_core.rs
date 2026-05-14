use crate::ime::dict::Dictionary;

/// 加载核心字典（单字 + 原始词组 + 扩展词组）
pub fn load_core_dict() -> Dictionary 
{
    let chars_data = include_str!("dicts/pinyin_chars.txt");
    let phrases_data = include_str!("dicts/phrases.txt");
    let expanded_data = include_str!("dicts/phrases_expanded.txt");
    Dictionary::new_with_expanded(chars_data, phrases_data, expanded_data)
}
