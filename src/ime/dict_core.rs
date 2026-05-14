use crate::ime::dict::Dictionary;

/// 加载核心字典
pub fn load_core_dict() -> Dictionary 
{
    let chars_data = include_str!("dicts/pinyin_chars.txt");
    let phrases_data = include_str!("dicts/phrases.txt");
    Dictionary::new(chars_data, phrases_data)
}
