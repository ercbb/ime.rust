use crate::ime::association::AssociationEngine;
use crate::ime::dict::{DictEntry, Dictionary};
use crate::ime::double_pinyin;
use crate::ime::pinyin;
use crate::ime::user_dict::UserDict;

use std::path::PathBuf;

/// 输入模式
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputMode {
    ChineseFull,
    ChineseDouble,
    English,
    Symbols,
}

/// 候选词
#[derive(Clone, Debug)]
pub struct Candidate {
    pub text: String,
}

/// IME 引擎
pub struct ImeEngine {
    pub mode: InputMode,
    pub input_buffer: String,
    pub candidates: Vec<Candidate>,
    pub candidate_page: usize,
    pub page_size: usize,
    pub output_text: String,
    pub cursor_pos: usize,
    pub caps_lock: bool,

    // 内部状态
    parsed_syllables: Vec<String>,
    remaining_buffer: String,
    association_engine: AssociationEngine,
    user_dict: UserDict,
    user_dict_path: PathBuf,
    dict: Option<Dictionary>,

    // 联想模式
    pub association_mode: bool,
    association_candidates: Vec<String>,
    last_input: String,
}

impl ImeEngine {
    pub fn new() -> Self {
        let user_dict_path = dirs_data_path();
        let user_dict = UserDict::load(&user_dict_path);

        Self {
            mode: InputMode::ChineseFull,
            input_buffer: String::new(),
            candidates: Vec::new(),
            candidate_page: 0,
            page_size: 6,
            output_text: String::new(),
            cursor_pos: 0,
            caps_lock: false,
            parsed_syllables: Vec::new(),
            remaining_buffer: String::new(),
            association_engine: AssociationEngine::new(),
            user_dict,
            user_dict_path,
            dict: None,
            association_mode: false,
            association_candidates: Vec::new(),
            last_input: String::new(),
        }
    }

    /// 设置字典
    pub fn set_dictionary(&mut self, dict: Dictionary) {
        self.dict = Some(dict);
    }

    /// 处理键盘输入
    pub fn process_key(&mut self, key: &str) -> Option<String> {
        // 联想模式下的输入处理
        if self.association_mode {
            self.association_mode = false;
            self.association_candidates.clear();
        }

        match self.mode {
            InputMode::ChineseFull => self.process_chinese_full(key),
            InputMode::ChineseDouble => self.process_chinese_double(key),
            InputMode::English => self.process_english(key),
            InputMode::Symbols => self.process_symbol(key),
        }
    }

    /// 处理中文全拼输入
    fn process_chinese_full(&mut self, key: &str) -> Option<String> {
        match key {
            // 数字键选择候选词
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let idx = key.parse::<usize>().unwrap() - 1;
                self.select_candidate(idx)
            }
            "0" => self.select_candidate(9),
            // 退格
            "backspace" => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    self.update_candidates();
                    None
                } else {
                    self.delete_before_cursor();
                    None
                }
            }
            // 空格：选择第一个候选词
            "space" => {
                if !self.candidates.is_empty() {
                    self.select_candidate(0)
                } else if self.input_buffer.is_empty() {
                    self.insert_at_cursor(" ");
                    None
                } else {
                    // 无候选词，直接将输入作为英文字符输出
                    self.insert_at_cursor(&self.input_buffer.clone());
                    self.input_buffer.clear();
                    self.candidates.clear();
                    None
                }
            }
            // 回车：直接输出输入缓冲区内容
            "enter" => {
                if !self.input_buffer.is_empty() {
                    let text = self.input_buffer.clone();
                    self.insert_at_cursor(&text);
                    self.input_buffer.clear();
                    self.candidates.clear();
                    None
                } else {
                    Some("enter".to_string())
                }
            }
            // 翻页
            "pagedown" | "=" => {
                self.next_page();
                None
            }
            "pageup" | "-" => {
                self.prev_page();
                None
            }
            // 字母输入
            c if c.len() == 1 && c.chars().next().map_or(false, |ch| ch.is_ascii_lowercase()) => {
                self.input_buffer.push_str(c);
                self.update_candidates();
                None
            }
            // 非 ASCII 单字符直接上屏（如中文标点符号）
            c if c.chars().count() == 1 && !c.chars().next().unwrap().is_ascii() => {
                self.insert_at_cursor(c);
                None
            }
            _ => None,
        }
    }

    /// 处理中文双拼输入
    fn process_chinese_double(&mut self, key: &str) -> Option<String> {
        match key {
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                let idx = key.parse::<usize>().unwrap() - 1;
                self.select_candidate(idx)
            }
            "0" => self.select_candidate(9),
            "backspace" => {
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    self.update_candidates();
                    None
                } else {
                    self.delete_before_cursor();
                    None
                }
            }
            "space" => {
                if !self.candidates.is_empty() {
                    self.select_candidate(0)
                } else if self.input_buffer.is_empty() {
                    self.insert_at_cursor(" ");
                    None
                } else {
                    self.insert_at_cursor(&self.input_buffer.clone());
                    self.input_buffer.clear();
                    self.candidates.clear();
                    None
                }
            }
            "enter" => {
                if !self.input_buffer.is_empty() {
                    let text = self.input_buffer.clone();
                    self.insert_at_cursor(&text);
                    self.input_buffer.clear();
                    self.candidates.clear();
                    None
                } else {
                    Some("enter".to_string())
                }
            }
            "pagedown" | "=" => {
                self.next_page();
                None
            }
            "pageup" | "-" => {
                self.prev_page();
                None
            }
            c if c.len() == 1 && c.chars().next().map_or(false, |ch| ch.is_ascii_lowercase()) => {
                self.input_buffer.push_str(c);
                self.update_candidates();
                None
            }
            // 非 ASCII 单字符直接上屏（如中文标点符号）
            c if c.chars().count() == 1 && !c.chars().next().unwrap().is_ascii() => {
                self.insert_at_cursor(c);
                None
            }
            _ => None,
        }
    }

    /// 处理英文输入
    fn process_english(&mut self, key: &str) -> Option<String> {
        match key {
            "space" => {
                self.insert_at_cursor(" ");
                None
            }
            "backspace" => {
                self.delete_before_cursor();
                None
            }
            "enter" => Some("enter".to_string()),
            c if c.len() == 1 && c.chars().next().map_or(false, |ch| ch.is_ascii_alphabetic()) => {
                if self.caps_lock {
                    self.insert_at_cursor(&c.to_uppercase());
                } else {
                    self.insert_at_cursor(c);
                }
                None
            }
            c => {
                self.insert_at_cursor(c);
                None
            }
        }
    }

    /// 处理数字/符号输入
    fn process_symbol(&mut self, key: &str) -> Option<String> {
        match key {
            "backspace" => {
                self.delete_before_cursor();
                None
            }
            "space" => {
                self.insert_at_cursor(" ");
                None
            }
            "enter" => Some("enter".to_string()),
            _ => {
                self.insert_at_cursor(key);
                None
            }
        }
    }

    /// 获取按键对应的符号（用于符号模式显示和输入）
    pub fn symbol_for_key(key: &str) -> Option<&'static str> {
        let symbol_map: [(&str, &str); 26] = [
            ("q", "1"), ("w", "2"), ("e", "3"), ("r", "4"), ("t", "5"), ("y", "6"), ("u", "7"), ("i", "8"), ("o", "9"), ("p", "0"),
            ("a", "!"), ("s", "@"), ("d", "#"), ("f", "$"), ("g", "%"), ("h", "/"), ("j", "&"), ("k", "*"), ("l", "+"),
            ("z", "<"), ("x", ">"), ("c", "("), ("v", ")"), ("b", ","), ("n", "."), ("m", "-"),
        ];
        symbol_map.iter().find(|(k, _)| *k == key).map(|(_, sym)| *sym)
    }

    /// 切换大小写锁定
    pub fn toggle_caps_lock(&mut self) {
        self.caps_lock = !self.caps_lock;
    }

    /// 更新候选词列表
    fn update_candidates(&mut self) {
        self.candidates.clear();
        self.candidate_page = 0;

        if self.input_buffer.is_empty() {
            return;
        }

        let (syllables, remaining) = match self.mode {
            InputMode::ChineseFull => pinyin::parse_pinyin_buffer(&self.input_buffer),
            InputMode::ChineseDouble => double_pinyin::double_pinyin_to_syllables(&self.input_buffer),
            _ => return,
        };

        self.parsed_syllables = syllables.clone();
        self.remaining_buffer = remaining.clone();

        if let Some(ref dict) = self.dict {
            let mut chars: Vec<DictEntry> = Vec::new();
            let mut phrases: Vec<DictEntry> = Vec::new();

            // 1. 单字（从最后一个音节查）
            if let Some(last) = syllables.last() {
                chars = dict.lookup_chars(last);
            }

            // 2. 词组
            if !syllables.is_empty() {
                let pinyin_key = syllables.join("'");

                // 精确匹配词组（仅多音节）
                if syllables.len() > 1 {
                    phrases.extend(dict.lookup_phrases_exact(&pinyin_key));
                }

                // 前缀匹配词组
                let prefix = dict.lookup_phrases_prefix(&pinyin_key);
                for entry in prefix {
                    if !phrases.iter().any(|e| e.text == entry.text) {
                        phrases.push(entry);
                    }
                }
            }

            // 应用用户频率（组内排序前）
            for entry in chars.iter_mut().chain(phrases.iter_mut()) {
                let boost = self.user_dict.freq_boost(&entry.text);
                if boost > 0.0 {
                    entry.freq += boost;
                }
            }

            // 组内按频率排序
            chars.sort_by(|a, b| b.freq.partial_cmp(&a.freq).unwrap_or(std::cmp::Ordering::Equal));
            phrases.sort_by(|a, b| b.freq.partial_cmp(&a.freq).unwrap_or(std::cmp::Ordering::Equal));

            // 去重：词组中移除已在单字中出现的条目
            phrases.retain(|p| !chars.iter().any(|c| c.text == p.text));

            // 合并：多音节时精确匹配词组排最前，单音节时单字在前
            let mut entries: Vec<DictEntry> = Vec::new();
            if syllables.len() > 1 {
                // 精确匹配词组 → 单字 → 其他词组
                let pinyin_key = syllables.join("'");
                let exact = dict.lookup_phrases_exact(&pinyin_key);
                for e in exact {
                    if !entries.iter().any(|x| x.text == e.text) {
                        entries.push(e);
                    }
                }
                entries.extend(chars);
                for e in phrases {
                    if !entries.iter().any(|x| x.text == e.text) {
                        entries.push(e);
                    }
                }
            } else {
                entries.extend(chars);
                entries.extend(phrases);
            }

            // 转换为候选词
            self.candidates = entries
                .into_iter()
                .take(50) // 最多保留50个候选
                .map(|e| Candidate {
                    text: e.text,
                })
                .collect();
        }
    }

    /// 选择候选词
    fn select_candidate(&mut self, index: usize) -> Option<String> {
        let page_start = self.candidate_page * self.page_size;
        let actual_index = page_start + index;

        if actual_index < self.candidates.len() {
            let selected = self.candidates[actual_index].text.clone();

            // 记录使用频率
            self.user_dict.record_usage(&selected);

            // 输出到文本
            self.insert_at_cursor(&selected);

            // 清空输入状态
            self.input_buffer.clear();
            self.candidates.clear();
            self.candidate_page = 0;
            self.last_input = selected.clone();

            // 更新联想
            self.association_candidates = self.association_engine.get_suggestions(&self.output_text);
            if !self.association_candidates.is_empty() {
                self.association_mode = true;
            }

            // 保存用户词典
            self.save_user_dict();

            None
        } else {
            None
        }
    }

    /// 选择联想建议
    pub fn select_association(&mut self, text: &str) {
        self.insert_at_cursor(text);
        self.user_dict.record_usage(text);
        self.last_input = text.to_string();

        // 继续联想
        self.association_candidates = self.association_engine.get_suggestions(&self.output_text);
        if self.association_candidates.is_empty() {
            self.association_mode = false;
        }

        self.save_user_dict();
    }

    /// 下一页候选词
    pub fn next_page(&mut self) {
        let total_pages = (self.candidates.len() + self.page_size - 1) / self.page_size;
        if self.candidate_page + 1 < total_pages {
            self.candidate_page += 1;
        }
    }

    /// 上一页候选词
    pub fn prev_page(&mut self) {
        if self.candidate_page > 0 {
            self.candidate_page -= 1;
        }
    }

    /// 获取当前页的候选词
    pub fn get_current_page_candidates(&self) -> Vec<&Candidate> {
        let start = self.candidate_page * self.page_size;
        self.candidates[start..].iter().take(self.page_size).collect()
    }

    /// 获取联想候选
    pub fn get_association_candidates(&self) -> &[String] {
        &self.association_candidates
    }

    /// 设置光标位置（按字符偏移，自动 clamp 到合法范围）
    pub fn set_cursor(&mut self, pos: usize) {
        let len = self.output_text.chars().count();
        self.cursor_pos = pos.min(len);
    }

    /// 在光标位置插入文本，光标后移至插入文本末尾
    fn insert_at_cursor(&mut self, text: &str) {
        let byte_pos = self
            .output_text
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.output_text.len());
        self.output_text.insert_str(byte_pos, text);
        self.cursor_pos += text.chars().count();
    }

    /// 删除光标前一个字符（若光标在开头则不操作）
    fn delete_before_cursor(&mut self) {
        if self.cursor_pos > 0 {
            let byte_pos = self
                .output_text
                .char_indices()
                .nth(self.cursor_pos - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.output_text.remove(byte_pos);
            self.cursor_pos -= 1;
        }
    }

    /// 切换输入模式
    pub fn toggle_mode(&mut self, new_mode: InputMode) {
        // 切换前清空状态
        if !self.input_buffer.is_empty() {
            self.input_buffer.clear();
            self.candidates.clear();
        }
        self.mode = new_mode;
        self.association_mode = false;
        self.association_candidates.clear();
    }

    /// 切换到下一个输入模式
    pub fn cycle_mode(&mut self) {
        let next = match self.mode {
            InputMode::ChineseFull => InputMode::ChineseDouble,
            InputMode::ChineseDouble => InputMode::English,
            InputMode::English => InputMode::Symbols,
            InputMode::Symbols => InputMode::ChineseFull,
        };
        self.toggle_mode(next);
    }

    /// 英文/数字模式切换
    pub fn cycle_lang_mode(&mut self) {
        let next = match self.mode {
            InputMode::English => InputMode::Symbols,
            InputMode::Symbols => InputMode::English,
            _ => InputMode::English,
        };
        self.toggle_mode(next);
    }

    /// 全拼/双拼模式切换
    pub fn cycle_cn_mode(&mut self) {
        let next = match self.mode {
            InputMode::ChineseFull => InputMode::ChineseDouble,
            InputMode::ChineseDouble => InputMode::ChineseFull,
            _ => InputMode::ChineseFull,
        };
        self.toggle_mode(next);
    }

    /// 保存用户词典
    fn save_user_dict(&self) {
        self.user_dict.save(&self.user_dict_path);
    }
}

/// 获取用户数据文件路径
fn dirs_data_path() -> PathBuf {
    let mut path = PathBuf::from(".");
    path.push("user_data");
    path.push("user_dict.json");
    path
}

impl Default for ImeEngine {
    fn default() -> Self {
        Self::new()
    }
}
