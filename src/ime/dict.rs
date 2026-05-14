use std::collections::HashMap;

/// 字典中的候选词条目
#[derive(Clone, Debug)]
pub struct DictEntry 
{
    pub text: String,
    pub freq: f64,
}

/// 字典管理器
pub struct Dictionary 
{
    /// 单字字典: 拼音 → 候选字列表
    chars: HashMap<String, Vec<DictEntry>>,
    /// 词组字典: 拼音(带引号分隔) → 候选词列表
    phrases: HashMap<String, Vec<DictEntry>>,
}

impl Dictionary 
{
    /// 从嵌入的字典数据创建字典
    pub fn new(chars_data: &str, phrases_data: &str) -> Self 
    {
        let chars = parse_dict_data(chars_data);
        let phrases = parse_dict_data(phrases_data);
        Self { chars, phrases }
    }

    /// 查询单字
    pub fn lookup_chars(&self, pinyin: &str) -> Vec<DictEntry> 
    {
        self.chars
            .get(&pinyin.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// 查找完全匹配给定音节序列的候选词
    pub fn lookup_exact(&self, syllables: &[String]) -> Vec<DictEntry> 
    {
        let mut results = Vec::new();

        if syllables.is_empty() 
        {
            return results;
        }

        // 词组精确匹配
        if syllables.len() > 1 
        {
            let pinyin_key = syllables.join("'");
            if let Some(entries) = self.phrases.get(&pinyin_key) 
            {
                results.extend(entries.iter().cloned());
            }
        }

        // 单字匹配（当只有一个音节时）
        if syllables.len() == 1 
        {
            if let Some(entries) = self.chars.get(syllables[0].as_str()) 
            {
                results.extend(entries.iter().cloned());
            }
        }

        results.sort_by(|a, b| {
            b.freq
                .partial_cmp(&a.freq)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// 查找以给定音节开头的词组（用于智能提示）
    pub fn lookup_prefix(&self, syllables: &[String]) -> Vec<DictEntry> 
    {
        let mut results = Vec::new();
        if syllables.is_empty() 
        {
            return results;
        }

        let pinyin_key = syllables.join("'");

        // 精确匹配的词组排在前面
        if let Some(entries) = self.phrases.get(&pinyin_key) 
        {
            results.extend(entries.iter().cloned());
        }

        // 前缀匹配
        for (key, entries) in &self.phrases 
        {
            if key.starts_with(&format!("{}'", pinyin_key)) {
                results.extend(entries.iter().cloned());
            }
        }

        // 单字匹配
        if syllables.len() == 1 
        {
            if let Some(entries) = self.chars.get(syllables[0].as_str()) 
            {
                results.extend(entries.iter().cloned());
            }
        }

        results.sort_by(|a, b| {
            b.freq
                .partial_cmp(&a.freq)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.dedup_by(|a, b| a.text == b.text);
        results
    }

    /// 查找精确匹配该拼音键的词组（不含单字）
    pub fn lookup_phrases_exact(&self, pinyin_key: &str) -> Vec<DictEntry> 
    {
        self.phrases.get(pinyin_key).cloned().unwrap_or_default()
    }

    /// 查找前缀匹配该拼音键的词组（不含单字）
    pub fn lookup_phrases_prefix(&self, pinyin_key: &str) -> Vec<DictEntry> 
    {
        let mut results = Vec::new();
        if let Some(entries) = self.phrases.get(pinyin_key) 
        {
            results.extend(entries.iter().cloned());
        }
        let prefix = format!("{}'", pinyin_key);
        for (key, entries) in &self.phrases 
        {
            if key.starts_with(&prefix) 
            {
                results.extend(entries.iter().cloned());
            }
        }
        results.sort_by(|a, b| {
            b.freq
                .partial_cmp(&a.freq)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.dedup_by(|a, b| a.text == b.text);
        results
    }
}

fn parse_dict_data(data: &str) -> HashMap<String, Vec<DictEntry>> 
{
    let mut map: HashMap<String, Vec<DictEntry>> = HashMap::new();

    for line in data.lines() 
    {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') 
        {
            continue;
        }

        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() >= 2 
        {
            let pinyin = parts[0].to_lowercase();
            let text = parts[1].to_string();
            let freq: f64 = if parts.len() >= 3 {
                parts[2].parse().unwrap_or(1.0)
            } else {
                1.0
            };

            map.entry(pinyin)
                .or_insert_with(Vec::new)
                .push(DictEntry { text, freq });
        }
    }

    // 每个拼音的候选按频率排序
    for entries in map.values_mut() 
    {
        entries.sort_by(|a, b| {
            b.freq
                .partial_cmp(&a.freq)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    map
}
