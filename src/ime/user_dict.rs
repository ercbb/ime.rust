use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 用户词典条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserEntry 
{
    pub text: String,
    pub pinyin: String,
    pub freq: u32,
}

/// 用户词典：保存用户高频词和自定义词组
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDict 
{
    /// 自定义词组: pinyin → entries
    #[serde(default)]
    entries: HashMap<String, Vec<UserEntry>>,
    /// 用户词频记录: text → count
    #[serde(default)]
    freq_map: HashMap<String, u32>,
}

impl UserDict 
{
    pub fn new() -> Self 
    {
        Self {
            entries: HashMap::new(),
            freq_map: HashMap::new(),
        }
    }

    /// 从文件加载用户词典
    pub fn load(path: &PathBuf) -> Self 
    {
        if let Ok(data) = fs::read_to_string(path) 
        {
            serde_json::from_str(&data).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// 保存用户词典到文件
    pub fn save(&self, path: &PathBuf) 
    {
        if let Some(parent) = path.parent() 
        {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) 
        {
            let _ = fs::write(path, data);
        }
    }

    /// 记录使用频率
    pub fn record_usage(&mut self, text: &str) 
    {
        *self.freq_map.entry(text.to_string()).or_insert(0) += 1;
    }

    /// 获取频率加权值（用于候选排序）
    pub fn freq_boost(&self, text: &str) -> f64 
    {
        let count = self.freq_map.get(text).copied().unwrap_or(0);
        if count > 0 
        {
            100.0 + count as f64 * 10.0
        } else {
            0.0
        }
    }
}

impl Default for UserDict 
{
    fn default() -> Self 
    {
        Self::new()
    }
}
