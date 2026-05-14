# Wheel IME 技术文档

## 1. 项目概述

Wheel IME 是一个基于 Rust + Slint 的中文输入法引擎，带自定义虚拟键盘。目标平台为 Allwinner T113-S3 嵌入式 ARM 板（Cortex-A7, Linux 6.18, simple-framebuffer + evdev + ALSA），同时支持 Ubuntu 桌面端开发调试。

### 技术栈

| 组件 | 版本 / 说明 |
|------|-------------|
| Slint | 1.15.1（UI 框架） |
| Rust | MSRV 1.92 |
| 桌面渲染 | winit + femtovg |
| 嵌入式渲染 | linuxkms-noseat + software renderer |
| 目标硬件 | Allwinner T113-S3, 800×480 LCD, Tina Linux |

### 功能特性

- **全拼输入**：标准汉语拼音，最长匹配音节解析
- **双拼输入**：自然码（Natural Code）方案，两键一音节
- **英文 / 符号模式**：直接文本输入和符号数字输入
- **光标编辑**：插入/删除光标定位，自定义视觉光标
- **GB2312 字库**：完整一级 3755 字 + 二级 3008 字 = 共 6763 字
- **词组输入**：1536 条常用词组，支持精确匹配和前缀增量匹配
- **用户词典**：频率学习，JSON 持久化至 `user_data/user_dict.json`
- **联想词**：基于上屏字符的静态联想推荐
- **四框独立缓冲**：英文、符号、全拼、双拼各自独立文本区

---

## 2. 架构概览

```
┌─────────────────────────────────────────────┐
│  Host Window (ui/main.slint)                │
│  ┌───────────────────────────────────────┐  │
│  │  Main layout                          │  │
│  │  - 顶部输出行                          │  │
│  │  - 四个文本框 (per-box buffers)        │  │
│  └───────────────────────────────────────┘  │
│  ┌───────────────────────────────────────┐  │
│  │  Keyboard overlay (绝对定位浮层, 800×360) │
│  │  KeyboardPanel ← ImeState global      │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
         ↕ 属性双向绑定 (<=> ImeState.xxx)
         → 回调 (key-pressed, candidate-selected, ...)
```

- **KeyboardPanel**：自包含输入法 UI 控件，不参与主屏布局流
- **ImeState**：Slint `global` 单例，承载所有 IME 共享状态
- **Host Window**：通过 `<=> ImeState.xxx` 绑定属性；Rust 端操作这些属性即可驱动 UI

### 模块职责 (`src/ime/`)

| 模块 | 职责 |
|------|------|
| `engine.rs` | 核心状态机：模式切换、输入缓冲、候选词管理、光标、翻页、按键分发 |
| `pinyin.rs` | 全拼解析：贪心最长匹配，返回已解析音节 + 未完成前缀 |
| `double_pinyin.rs` | 自然码双拼解码：两键一音节，含声韵母消歧规则 |
| `syllable_table.rs` | `VALID_SYLLABLES` 常量数组，`is_valid_syllable()` / `is_valid_prefix()` |
| `dict.rs` | `Dictionary`：单字 + 词组 HashMap 查询 |
| `dict_core.rs` | 通过 `include_str!` 加载嵌入字典 |
| `association.rs` | 静态联想引擎：上屏字 → 推荐词 |
| `user_dict.rs` | JSON 持久化用户词频，提供 `freq_boost()` |
| `layout.rs` | `update_ui()`：将引擎状态推送到 Slint 属性 |
| `ui/keyboard.slint` | 可复用组件：`KeyButton`, `CandidateBar`, `KeyboardRows`, `KeyboardPanel`, `ImeState` |

---

## 3. 数据流

### 3.1 按键 → 候选词 → 上屏

```
TouchArea clicked (Slint)
  → app.rs on_key_pressed 回调
    → engine.process_key(key_str)
      → dispatch by InputMode
        ChineseFull  → process_chinese_full()
        ChineseDouble → process_chinese_double()
        English      → process_english()
        Symbols      → process_symbol()
      → 中文模式：字母追加到 input_buffer → update_candidates()
        → parse_pinyin_buffer() 或 double_pinyin_to_syllables()
          → 返回 (parsed_syllables, remaining_buffer)
        → dict.lookup_chars()     — 单字检索
        → dict.lookup_phrases_exact()    — 词组精确匹配
        → dict.lookup_phrases_prefix()   — 词组前缀匹配
        → user_dict.freq_boost()  — 用户频率加成
        → 合并排序 → candidates
      → 数字键选择候选词：select_candidate(idx)
        → insert_at_cursor(text)
        → record_usage() → 更新用户词典
        → association_engine.get_suggestions() → 联想模式
  → layout::update_ui() 推送状态到 Slint 属性
```

### 3.2 写入回写策略

| 操作 | 主屏文本框行为 |
|------|----------------|
| 按键输入 / 选词 / 联想 | **不回写**，仅更新键方面板 |
| 按回车 | 将 `engine.output_text` 写入当前活跃框缓冲区，隐藏键盘 |
| 按放弃 / 底部空白区 | 从缓冲区恢复 `engine.output_text`（丢弃编辑），隐藏键盘 |
| 点击其他模式框 | 保存至前一个框（等同确认），切换活跃框 |

---

## 4. 候选词检索算法（2026-05-14 改进）

### 4.1 拼音解析

全拼使用贪心最长匹配算法的变体：

```rust
pub fn parse_pinyin_buffer(input: &str) -> (Vec<String>, String)
```

- 从输入开头尝试最长音节匹配
- 检查剩余部分是否能构成有效拼音前缀（避免过早切割）
- 返回已确认音节列表 + 无法继续匹配的剩余前缀

示例：
- `"jisuan"` → `(["ji", "suan"], "")`
- `"jis"` → `(["ji"], "s")`
- `"xian"` → `(["xian"], "")`

### 4.2 候选词生成流程 (`update_candidates`)

```
输入缓冲 → 音节解析 → 分组检索 → 频率排序 → 合并排序 → 候选列表
```

#### 第一步：单字检索（所有已解析音节）

```rust
for syl in &syllables {
    chars.extend(dict.lookup_chars(syl));
}
```

- 从**每一个**已解析音节查单字，而非仅最后一个
- 按频率排序去重

#### 第二步：词组检索（含未完成前缀）

构建完整拼音键：

```rust
let pinyin_key = if remaining.is_empty() {
    syllables.join("'")                 // "ji'suan"
} else if base_key.is_empty() {
    remaining                           // "j"
} else {
    format!("{}'{}", base_key, remaining)  // "ji's"
};
```

- **精确匹配**：仅所有音节完整解析 (`remaining.is_empty()`) 时触发
- **前缀匹配**：始终使用完整拼音键。当 `pinyin_key` 已含 `'`（跨音节）时，直接用键本身做前缀；否则追加 `'`

#### 第三步：频率排序

- 组内按频率降序排序（含用户词典加成）
- 词组与单字交叉去重

#### 第四步：合并排序

```
if 有未完成音节 || 多音节：
    词组优先 → 单字
else（纯单音节）：
    单字优先 → 词组
```

### 4.3 增量输入效果

| 输入 | 音节解析 | 候选词排序 |
|------|----------|-----------|
| `j` | `([], "j")` | 无（前缀过短） |
| `ji` | `(["ji"], "")` | `ji` 单字 → `ji'` 前缀词组 |
| `jis` | `(["ji"], "s")` | `ji's` 前缀词组（计算、技术…） → `ji` 单字 |
| `jisu` | `(["ji"], "su")` | `ji'su` 前缀词组 → `ji` 单字 |
| `jisuan` | `(["ji","suan"], "")` | `ji'suan` 精确词组 → 前缀词组 → `ji`+`suan` 单字 |

关键改进：`remaining` 缓冲区全程参与词组前缀匹配，确保增量输入时候选词精准窄化，而非退化为仅匹配已解析部分的宽泛查询。

---

## 5. 光标系统

- `engine.cursor_pos: usize` — 字符偏移（非字节偏移）
- `insert_at_cursor(text)`：在光标位置插入文本，光标前移
- `delete_before_cursor()`：删除光标前一字符（pos 0 无操作）
- 方向键左右移动光标（Row 4）
- Slint 1.15.1 `TextInput` 不暴露字符级光标位置；视觉光标通过自定义 `HorizontalLayout( Text + Rectangle(2px) + Text )` 实现

---

## 6. 键盘布局

```
Row 1:  q  w  e  r  t  y  u  i  o  p
Row 2:  a  s  d  f  g  h  j  k  l  ⌫
Row 3:  z  x  c  v  b  n  m  .  ⏎(2x宽)
Row 4:  英文  数字  全拼/双拼  空格  ←  →  [spacer]  放弃
```

- Row 2 右侧为退格键
- Row 3 回车宽度 = `key-w * 2`，与 Row 2 退格右对齐
- Row 2/3 左对齐 (`start`)
- Row 4 "放弃" 按钮通过 spacer 右对齐

---

## 7. 词典格式

### 单字字典 (`pinyin_chars.txt`)

```
pinyin hanzi frequency
```
- GB2312 一级 3755 字（频次 500→10）
- GB2312 二级 3008 字（频次 200→5）
- 共计 6763 个唯一汉字

### 词组字典 (`phrases.txt`)

```
ji'suan 计算 1000
```
- 多音节用 `'` 分隔
- 1536 条常用词组
- 频次由 pypinyin 自动标注（去声调）

### 汉字标注示例

```
ji 机 500
ji 几 490
ji 及 480
...
ji'suan 计算 1000
ji'suan 就算 600
ji'shu 技术 800
```

---

## 8. 自然码双拼方案

`double_pinyin.rs` 实现自然码双拼编码：

### 声韵母映射

- 特殊声母映射：`v`→zh, `i`→ch, `u`→sh
- `combine_initial_final()` 处理一键多韵母消歧：
  - `d` 键：uang/iang（依声母 j/q/x 判断）
  - `w` 键：ia/ua（依声母 g/k/h/zh/ch/sh 判断）
  - `y` 键：ing/uai（依声母 g/k/h/zh/ch/sh 判断）
  - `s` 键：ong/iong（依声母 j/q/x 判断）
  - `o` 键：uo/o（依声母 b/p/m/f 判断）
- j/q/x + u/ue/uan/un → ü 变体
- n/l + u/ue → nv/lv

### 零声母处理

零声母音节首键直接输入韵母，如 `a`→a, `e`→e, `o`→o。

---

## 9. 用户词典

`user_dict.rs` 维护用户使用频率，持久化至 `user_data/user_dict.json`：

```json
{
  "计算": 12,
  "技术": 8,
  "机": 25
}
```

- `record_usage(text)`：每次选词递增计数
- `freq_boost(text)`：返回对数频率加成值
- 选词后自动保存
- 频率加成作用于候选词排序，使常用词逐步前置

---

## 10. 构建与部署

### Desktop (Ubuntu)

```bash
cargo build
cargo run              # 可传参：cargo run -- 8（候选词数量）
```

### Embedded ARM

```bash
cargo build --no-default-features --features embedded \
    --target armv7-unknown-linux-musleabihf --release
```

在设备上运行：
```bash
SLINT_BACKEND_LINUXFB=1 ./wheel-rust     # fbdev
SLINT_BACKEND=linuxkms ./wheel-rust       # DRM/KMS
```

### Feature Flags

`Cargo.toml` 定义两个互斥 feature：

| Feature | 后端 | 渲染器 | 场景 |
|---------|------|--------|------|
| `desktop`（默认） | winit | femtovg | Ubuntu 开发 |
| `embedded` | linuxkms-noseat | software | T113-S3 ARM |

### 测试

```bash
cargo test                              # 全量测试
cargo test test_final_z_ei              # 指定双拼测试
```

---

## 11. 文件结构

```
ime.rust/
├── Cargo.toml
├── build.rs                    # Slint 编译，fluent-light 样式
├── rust-toolchain.toml
├── ui/
│   └── main.slint              # 主窗口：文本区 + 键盘浮层
├── assets/
│   └── bg.png                  # 背景图
├── src/
│   ├── main.rs
│   ├── app.rs                  # 回调注册，引擎与 Slint 桥接
│   └── ime/
│       ├── mod.rs
│       ├── engine.rs           # 核心状态机
│       ├── pinyin.rs           # 全拼解析
│       ├── double_pinyin.rs    # 自然码双拼
│       ├── syllable_table.rs   # 音节表
│       ├── dict.rs             # 字典查询
│       ├── dict_core.rs        # 嵌入字典加载
│       ├── association.rs      # 联想引擎
│       ├── user_dict.rs        # 用户词频
│       ├── layout.rs           # UI 状态推送
│       ├── dicts/
│       │   ├── pinyin_chars.txt
│       │   ├── phrases.txt
│       │   ├── phrases_expanded.txt
│       │   └── phrases_geo.txt
│       └── ui/
│           ├── keyboard.slint   # KeyboardPanel 控件
│           └── KEYBOARD_PANEL.md
└── user_data/
    └── user_dict.json           # 用户词频持久化
```

---

## 12. API 参考

### ImeEngine 关键方法

| 方法 | 说明 |
|------|------|
| `process_key(key: &str) -> Option<String>` | 处理按键，返回 `Some("enter")` 表示回车确认 |
| `select_candidate(index: usize) -> Option<String>` | 选择候选词，触发上屏和联想 |
| `insert_at_cursor(text: &str)` | 在光标位置插入文本 |
| `delete_before_cursor()` | 删除光标前一字符 |
| `toggle_mode(mode: InputMode)` | 切换输入模式 |
| `set_dictionary(dict: Dictionary)` | 设置字典 |
| `next_page()` / `prev_page()` | 翻页 |

### InputMode 枚举

```rust
pub enum InputMode {
    ChineseFull,    // 全拼
    ChineseDouble,  // 双拼
    English,        // 英文
    Symbols,        // 符号
}
```

### DictEntry

```rust
pub struct DictEntry {
    pub text: String,  // 汉字
    pub freq: f64,     // 频率
}
```

### Dictionary 查询方法

| 方法 | 说明 |
|------|------|
| `lookup_chars(pinyin)` | 单字精确匹配 |
| `lookup_phrases_exact(pinyin_key)` | 词组精确匹配 |
| `lookup_phrases_prefix(pinyin_key)` | 词组前缀匹配 |

---

*最后更新：2026-05-14*
