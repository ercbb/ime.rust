# Wheel IME — 输入法控件集成指南

Wheel IME 是一个基于 Slint 的中文输入法控件（KeyboardPanel），支持全拼、双拼、英文、符号四种输入模式。控件与主程序高度解耦——键盘作为浮层覆盖在主屏上方，不参与主屏布局流，不修改主屏显示。

## 1. 架构概览

```
┌─────────────────────────────────────────────┐
│  Host Window (main.slint)                   │
│  ┌───────────────────────────────────────┐  │
│  │  Main layout (always full height)     │  │
│  │  - 顶部输出行                          │  │
│  │  - 四个文本框 (用户业务数据)           │  │
│  └───────────────────────────────────────┘  │
│  ┌───────────────────────────────────────┐  │
│  │  Keyboard overlay (绝对定位浮层)       │  │
│  │  KeyboardPanel ← ImeState global      │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
         ↕ 属性双向绑定 (<=> ImeState.xxx)
         → 回调 (key-pressed, candidate-selected, …)
```

- **KeyboardPanel**：自包含的输入法 UI 控件，自有布局参数默认值，零外部依赖
- **ImeState**：Slint `global` 单例，承载所有 IME 共享状态（候选字、拼音缓冲、模式等）
- **Host Window**：通过 `<=> ImeState.xxx` 声明绑定属性，Rust 端操作这些属性即可驱动键盘

## 2. 必需文件

将以下文件复制到你的项目中：

```
src/ime/ui/keyboard.slint     ← KeyboardPanel + ImeState + CandidateItem + KeyButton
src/ime/engine.rs              ← ImeEngine 状态机（全拼/双拼/英文/符号）
src/ime/pinyin.rs              ← 全拼解析
src/ime/double_pinyin.rs       ← 自然码双拼解析
src/ime/syllable_table.rs      ← 音节表
src/ime/dict.rs                ← 字典查询
src/ime/dict_core.rs           ← 内嵌字典加载
src/ime/association.rs         ← 联想词
src/ime/user_dict.rs           ← 用户词频
src/ime/layout.rs              ← update_ui() 状态推送
src/ime/dicts/phrases.txt      ← 词组字典
src/ime/dicts/pinyin_chars.txt ← 单字字典
```

## 3. Slint 端集成（最小模板）

在你的 `main.slint` 中按以下步骤集成：

### 3.1 导入

```slint
import { CandidateItem, KeyboardPanel, ImeState } from "path/to/keyboard.slint";
```

### 3.2 声明 IME 属性（与 ImeState 双向绑定）

```slint
export component MainWindow inherits Window {
    // --- 键盘显隐 ---
    in-out property <bool> show-ime: false;

    // --- IME 属性（<=> ImeState.xxx 自动同步到键盘）---
    in-out property <string> output-text <=> ImeState.output-text;
    in-out property <string> text-before-cursor <=> ImeState.text-before-cursor;
    in-out property <string> text-after-cursor <=> ImeState.text-after-cursor;
    in-out property <string> input-buffer <=> ImeState.input-buffer;
    in-out property <string> mode-indicator <=> ImeState.mode-indicator;
    in-out property <color> mode-color <=> ImeState.mode-color;
    in-out property <[CandidateItem]> candidates <=> ImeState.candidates;
    in-out property <[CandidateItem]> associations <=> ImeState.associations;
    in-out property <bool> show-associations <=> ImeState.show-associations;
    in-out property <bool> has-prev-page <=> ImeState.has-prev-page;
    in-out property <bool> has-next-page <=> ImeState.has-next-page;
    in-out property <bool> symbol-mode <=> ImeState.symbol-mode;
    in-out property <bool> caps-lock <=> ImeState.caps-lock;
    in-out property <string> mode-tag <=> ImeState.mode-tag;
    in-out property <string> cn-method <=> ImeState.cn-method;
```

### 3.3 声明回调

```slint
    callback key-pressed(string);
    callback candidate-selected(int);
    callback association-selected(string);
    callback prev-page();
    callback next-page();
    callback show-keyboard-with-mode(string);
    callback show-keyboard();
    callback hide-keyboard();
```

### 3.4 放置键盘覆盖层

键盘应作为绝对定位的浮层，放在窗口根层级，**不要**放在主布局流中：

```slint
    // 主布局 (全高，不被键盘挤压)
    VerticalLayout {
        // ... 你的业务 UI ...
    }

    // 键盘覆盖层 (绝对定位，浮在主屏上方)
    if root.show-ime: Rectangle {
        x: 0;
        y: root.height - 384px;   // 384 = KeyboardPanel 高度(360) + 底部边距(24)
        width: root.width;
        height: 384px;
        background: #00000050;     // 半透明遮罩，遮挡主屏内容

        // 兜底触摸区：消费空白处点击，防止穿透到主屏
        TouchArea { }

        VerticalLayout {
            KeyboardPanel {
                // 所有数据属性从 ImeState 读取
                output-text <=> ImeState.output-text;
                input-buffer: ImeState.input-buffer;
                mode-indicator: ImeState.mode-indicator;
                mode-color: ImeState.mode-color;
                candidates: ImeState.candidates;
                associations: ImeState.associations;
                show-associations: ImeState.show-associations;
                has-prev-page: ImeState.has-prev-page;
                has-next-page: ImeState.has-next-page;
                text-before-cursor: ImeState.text-before-cursor;
                text-after-cursor: ImeState.text-after-cursor;
                symbol-mode: ImeState.symbol-mode;
                caps-lock: ImeState.caps-lock;
                mode-tag: ImeState.mode-tag;
                cn-method: ImeState.cn-method;

                // 回调 → 转发到 Host 的回调
                key-pressed(id) => { root.key-pressed(id); }
                candidate-selected(idx) => { root.candidate-selected(idx); }
                association-selected(text) => { root.association-selected(text); }
                prev-page() => { root.prev-page(); }
                next-page() => { root.next-page(); }
                hide-keyboard() => { root.hide-keyboard(); }
            }

            // 底部边距 (点击关闭键盘)
            Rectangle {
                height: 24px;
                background: transparent;
                TouchArea { clicked => { root.hide-keyboard(); } }
            }
        }
    }
```

> **注意：** `KeyboardPanel` 自有完整的布局默认值（键宽 68px、键高 58px、面板 780×360px 等），无需传入布局参数。如需自定义，可传入 `kb-width`、`kb-height`、`key-height`、`key-gap` 等 in 属性覆盖。

## 4. Rust 端集成

### 4.1 初始化

```rust
use crate::ime::dict_core;
use crate::ime::engine::{ImeEngine, InputMode};
use crate::ime::layout;
use crate::MainWindow;

pub fn run(candidate_count: usize, cn_double: bool) {
    let main_window = MainWindow::new().unwrap();

    // 设置初始中文方法 (通过 <=> 自动同步到 ImeState)
    let initial_cn = if cn_double { "double" } else { "full" };
    main_window.set_cn_method(initial_cn.into());

    // 初始化 IME 引擎
    let mut engine = ImeEngine::new();
    if cn_double {
        engine.toggle_mode(InputMode::ChineseDouble);
    }
    engine.page_size = candidate_count.max(1);
    engine.set_dictionary(dict_core::load_core_dict());

    // ⚠️ 首次推送状态到 UI
    layout::update_ui(&main_window, &engine);

    // ... 注册回调 ...
    main_window.run().unwrap();
}
```

### 4.2 核心回调：key-pressed

```rust
main_window.on_key_pressed(move |key_id: SharedString| {
    let mut engine = engine_rc.borrow_mut();
    let key_str = key_id.as_str();

    // 模式切换键
    if key_str == "mode_en" {
        engine.toggle_mode(InputMode::English);
    } else if key_str == "mode_num" {
        engine.toggle_mode(InputMode::Symbols);
    } else if key_str == "mode_cn" {
        let target = if cn_pref == "double" { ChineseDouble } else { ChineseFull };
        engine.toggle_mode(target);
    } else if key_str == "caps_lock" {
        engine.toggle_caps_lock();
    } else if key_str == "cursor_left" || key_str == "cursor_right" {
        // 光标移动
    } else {
        engine.process_key(key_str);  // 字母、数字、回车、退格等
    }

    if let Some(win) = window_weak.upgrade() {
        layout::update_ui(&win, &engine);  // 推送状态到 Slint

        // 回车：确认写入，隐藏键盘
        if key_str == "enter" { win.set_show_ime(false); }
    }
});
```

### 4.3 其他回调

```rust
// 候选字选择
main_window.on_candidate_selected(move |idx: i32| {
    engine.process_key(&(idx + 1).to_string());
    layout::update_ui(&win, &engine);
});

// 联想词选择
main_window.on_association_selected(move |text: SharedString| {
    engine.select_association(text.as_str());
    layout::update_ui(&win, &engine);
});

// 翻页
main_window.on_prev_page(move || { engine.prev_page(); layout::update_ui(&win, &engine); });
main_window.on_next_page(move || { engine.next_page(); layout::update_ui(&win, &engine); });

// 显示/隐藏键盘
main_window.on_show_keyboard(move || { win.set_show_ime(true); });
main_window.on_hide_keyboard(move || { win.set_show_ime(false); });
```

### 4.4 layout::update_ui

`layout::update_ui(&window, &engine)` 是状态推送的核心函数，负责将 `ImeEngine` 的所有状态同步到 Slint 属性。调用时机：

| 事件 | 调用 |
|------|------|
| 任意按键 | ✅ |
| 候选字选择 | ✅ |
| 联想词选择 | ✅ |
| 翻页 | ✅ |
| 显示键盘 | ✅ |
| 隐藏键盘 | 不需要（键盘不可见） |

## 5. KeyboardPanel 接口参考

### 5.1 in 属性（从外部接收）

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `kb-width` | length | 780px | 键盘面板宽度 |
| `kb-height` | length | 360px | 键盘面板高度 |
| `key-height` | length | 58px | 按键高度 |
| `key-gap` | length | 8px | 按键间隙 |
| `key-w` | length | 68px | 标准按键宽度 |
| `mode-w` | length | 136px | 模式键宽度 |
| `space-w` | length | 204px | 空格键宽度 |
| `ctrl-w` | length | 78px | 控制键宽度 |
| `kb-opacity` | float | 0.85 | 键盘整体不透明度 |
| `symbol-mode` | bool | false | 符号模式切换 |
| `caps-lock` | bool | false | 大写锁定 |
| `mode-tag` | string | "full" | 当前输入模式标签 |
| `cn-method` | string | "full" | 中文输入方案 |
| `input-buffer` | string | "" | 拼音输入缓冲 |
| `mode-indicator` | string | "中" | 模式指示字符 |
| `mode-color` | color | #0078d7 | 模式指示颜色 |
| `candidates` | [CandidateItem] | [] | 候选字列表 |
| `associations` | [CandidateItem] | [] | 联想词列表 |
| `show-associations` | bool | false | 显示联想栏 |
| `has-prev-page` | bool | false | 有上一页 |
| `has-next-page` | bool | false | 有下一页 |
| `text-before-cursor` | string | "" | 光标前文本 |
| `text-after-cursor` | string | "" | 光标后文本 |

### 5.2 in-out 属性

| 属性 | 说明 |
|------|------|
| `output-text` | 编辑区完整文本（双向绑定） |

### 5.3 回调

| 回调 | 参数 | 说明 |
|------|------|------|
| `key-pressed` | string (key-id) | 按键按下 |
| `candidate-selected` | int (index) | 候选字选中 |
| `association-selected` | string (text) | 联想词选中 |
| `prev-page` | — | 上一页候选 |
| `next-page` | — | 下一页候选 |
| `hide-keyboard` | — | 放弃编辑，关闭键盘 |

### 5.4 ImeState 全局属性

| 属性 | 类型 | 说明 |
|------|------|------|
| `output-text` | string | 编辑区文本 |
| `input-buffer` | string | 拼音缓冲 |
| `mode-indicator` | string | 模式指示 |
| `mode-color` | color | 模式颜色 |
| `candidates` | [CandidateItem] | 候选字 |
| `associations` | [CandidateItem] | 联想词 |
| `show-associations` | bool | 显示联想 |
| `has-prev-page` | bool | 有上一页 |
| `has-next-page` | bool | 有下一页 |
| `text-before-cursor` | string | 光标前文本 |
| `text-after-cursor` | string | 光标后文本 |
| `symbol-mode` | bool | 符号模式 |
| `caps-lock` | bool | 大写锁定 |
| `mode-tag` | string | 模式标签 |
| `cn-method` | string | 中文方案 |

## 6. 注意事项

1. **键盘覆盖层定位**：覆盖层的 `y` 应为 `root.height - 384px`（384 = 键盘高度 360 + 底部边距 24）。`x` 为 0，`width` 为 `root.width`。

2. **事件隔离**：覆盖层内必须有一个空的 `TouchArea { }` 在 VerticalLayout 之前，否则点击键盘空白处会穿透到主屏控件。

3. **半透明遮罩**：覆盖层 `background` 建议设 `#00000050`，确保主屏内容在键盘下方不可见。

4. **布局参数**：KeyboardPanel 自有默认值，通常无需传入。如需适配不同屏幕，传入 `kb-width`、`kb-height`、`key-height`、`key-gap` 即可。

5. **首次状态推送**：`layout::update_ui()` 必须在注册回调之前调用一次，否则初始候选栏为空。

6. **`show-ime` 切换**：键盘显隐只改变覆盖层的 `visible`，主屏布局始终占据全高，不会被挤压。

7. **中文方案**：`cn-method` 属性控制双拼/全拼切换。在 Rust 端通过 `main_window.set_cn_method("double")` 设置；Slint 端通过 `<=> ImeState.cn-method` 自动同步。

8. **候选字数**：`engine.page_size` 控制每页候选字数，默认可通过命令行参数传入（`cargo run -- 5`）。

## 7. 双拼（自然码）快速参考

### 7.1 声母映射

| 按键 | 声母 | 按键 | 声母 |
|------|------|------|------|
| v | zh | i | ch |
| u | sh | 其他 | 同键位 |

### 7.2 韵母映射

| 按键 | 韵母 | 按键 | 韵母 | 按键 | 韵母 |
|------|------|------|------|------|------|
| a | a | j | an | s | ong |
| b | ou | k | ao | t | ue |
| c | iao | l | ai | u | u |
| d | uang/iang | m | ian | v | ui |
| e | e | n | in | w | ia/ua |
| f | en | o | uo/o | x | ie |
| g | eng | p | un | y | ing/uai |
| h | ang | q | iu | z | ei |
| i | i | r | uan | | |

### 7.3 消歧规则

| 按键 | 默认韵母 | 条件 | 实际韵母 |
|------|----------|------|----------|
| d | uang | j/q/x 后 | iang |
| w | ia | g/k/h/zh/ch/sh 后 | ua |
| y | ing | g/k/h/zh/ch/sh 后 | uai |
| s | ong | j/q/x 后 | iong |
| o | uo | b/p/m/f 后 | o |

### 7.4 零声母

对于 a/e/o 开头的零声母字，首键用 a/e/o 作为韵母标记，第二键为韵母键：

| 汉字 | 拼音 | 编码 | 规则 |
|------|------|------|------|
| 阿 | a | aa | a + a |
| 爱 | ai | al | a + l(ai) |
| 安 | an | aj | a + j(an) |
| 奥 | ao | ak | a + k(ao) |
| 昂 | ang | ah | a + h(ang) |
| 恩 | en | ef | e + f(en) |
| 诶 | ei | ez | e + z(ei) |
| 欧 | ou | ob | o + b(ou) |

### 7.5 示例

| 汉字 | 双拼 | 说明 |
|------|------|------|
| 你 | ni | n + i |
| 好 | hk | h + k(ao) |
| 中国 | vsgo | v(zh)+s(ong), g+o(uo) |
| 江 | jd | j + d(iang) |
| 安 | aj | a + j(an) |
| 恩爱 | efal | e+f(en), a+l(ai) |

## 8. 完整示例

参见本仓库 `ui/main.slint`（Slint 端）和 `src/app.rs`（Rust 端）。
