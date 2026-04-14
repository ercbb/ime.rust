use crate::ime::syllable_table::{is_valid_prefix, is_valid_syllable};

/// 尝试将输入缓冲区解析为拼音音节列表
/// 返回 (已确认的音节列表, 剩余未完成的输入)
pub fn parse_pinyin_buffer(input: &str) -> (Vec<String>, String) {
    if input.is_empty() {
        return (vec![], String::new());
    }

    let input = input.to_lowercase();
    let mut result = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let remaining: String = chars[pos..].iter().collect();
        let mut matched: Option<(String, usize)> = None;

        // 从最长到最短尝试匹配
        for end in (1..=remaining.len()).rev() {
            let candidate: String = remaining[..end].to_string();
            if is_valid_syllable(&candidate) {
                // 检查剩余部分是否也能构成有效前缀
                let after = &remaining[end..];
                if after.is_empty() || is_valid_prefix(after) {
                    matched = Some((candidate, pos + end));
                    pos += end;
                    break;
                } else {
                    // 可能后面能组合成更长的匹配，继续尝试更短的
                    matched = Some((candidate, pos + end));
                }
            }
        }

        if let Some((syl, new_pos)) = matched {
            // 检查是否有更好的分割
            result.push(syl);
            pos = new_pos;
        } else if is_valid_prefix(&chars[pos..].iter().collect::<String>()) {
            // 未完成的音节
            let remaining: String = chars[pos..].iter().collect();
            return (result, remaining);
        } else {
            // 无效输入
            pos += 1;
        }
    }

    // 检查最后是否有未完成的音节
    let remaining: String = chars[pos..].iter().collect();
    if remaining.is_empty() {
        (result, String::new())
    } else {
        (result, remaining)
    }
}
