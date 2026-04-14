use crate::ime::syllable_table::{is_valid_prefix, is_valid_syllable};

/// 全拼解析器：将连续的英文字母序列切分为拼音音节
pub fn parse_full_pinyin(input: &str) -> Vec<String> {
    let input = input.to_lowercase();
    if input.is_empty() {
        return vec![];
    }

    // 贪心匹配：从每个位置尝试匹配最长合法音节
    let mut result = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let remaining: String = chars[pos..].iter().collect();
        let mut matched: Option<String> = None;

        // 从最长到最短尝试匹配
        for end in (1..=remaining.len()).rev() {
            let candidate: String = remaining[..end].to_string();
            if is_valid_syllable(&candidate) {
                matched = Some(candidate);
                pos += end;
                break;
            }
        }

        if let Some(syl) = matched {
            result.push(syl);
        } else {
            // 尝试找到下一个有效前缀的起始位置
            // 如果当前位置不是有效前缀，跳过一个字符
            if !is_valid_prefix(&remaining) {
                pos += 1;
            } else {
                // 当前是有效前缀但不是完整音节，等待更多输入
                break;
            }
        }
    }

    result
}

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

/// 智能分词：尝试找到所有可能的音节切分方案，返回最优方案
pub fn smart_parse(input: &str) -> Vec<String> {
    let input = input.to_lowercase();
    if input.is_empty() {
        return vec![];
    }

    // 使用贪心最长匹配
    greedy_parse(&input)
}

fn greedy_parse(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = input.chars().collect();

    while pos < chars.len() {
        let remaining: String = chars[pos..].iter().collect();
        let mut best_match: Option<String> = None;

        // 从最长到最短匹配
        for end in (1..=remaining.len()).rev() {
            let candidate = &remaining[..end];
            if is_valid_syllable(candidate) {
                best_match = Some(candidate.to_string());
                pos += end;
                break;
            }
        }

        if let Some(syl) = best_match {
            result.push(syl);
        } else {
            pos += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nihao() {
        let result = parse_full_pinyin("nihao");
        assert_eq!(result, vec!["ni", "hao"]);
    }

    #[test]
    fn test_parse_zhongguo() {
        let result = parse_full_pinyin("zhongguo");
        assert_eq!(result, vec!["zhong", "guo"]);
    }

    #[test]
    fn test_parse_women() {
        let result = parse_full_pinyin("women");
        assert_eq!(result, vec!["wo", "men"]);
    }

    #[test]
    fn test_parse_xian() {
        // "xian" 可以是 "xi"+"an" 或 "xian"
        let result = parse_full_pinyin("xian");
        assert_eq!(result, vec!["xian"]);
    }

    #[test]
    fn test_smart_parse() {
        let result = smart_parse("nihaozhongguo");
        assert_eq!(result, vec!["ni", "hao", "zhong", "guo"]);
    }
}
