# KeyboardPanel 控件说明

自包含的中文输入法键盘控件，可直接嵌入任意 Slint 窗口。

## 引入

```slint
import { KeyboardPanel } from "ime/ui/keyboard.slint";
```

## 属性

### 布局 (Layout)

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `kb-width` | `length` | `600px` | 键盘面板宽度 |
| `kb-height` | `length` | `320px` | 键盘面板高度（不含关闭按钮） |
| `key-height` | `length` | `52px` | 按键高度 |
| `key-gap` | `length` | `4px` | 按键间距 |
| `key-w` | `length` | `56px` | 普通按键宽度 |
| `mode-w` | `length` | `112px` | 模式切换键宽度 |
| `space-w` | `length` | `168px` | 空格键宽度 |
| `ctrl-w` | `length` | `64px` | 控制键宽度（退格、收起等） |
| `kb-opacity` | `float` | `0.85` | 键盘透明度 |

### 模式 (Mode)

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `symbol-mode` | `bool` | `false` | `true` 显示符号键盘，`false` 显示字母键盘 |
| `caps-lock` | `bool` | `false` | 大写锁定状态（影响 Row3 首键标签） |
| `mode-tag` | `string` | `"full"` | 当前输入模式，取值：`"full"` (全拼)、`"double"` (双拼)、`"english"` (英文)、`"symbols"` (符号)。影响 Row4 模式键的标签文字 |

### 数据 (Data) — 由 IME 引擎推送

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `output-text` | `in-out string` | `""` | 键盘编辑区显示的文本（双向绑定，引擎端修改即同步） |
| `input-buffer` | `string` | `""` | 当前输入的拼音/编码字符 |
| `mode-indicator` | `string` | `"中"` | 模式指示文字（显示在候选栏左侧） |
| `mode-color` | `color` | `#0078d7` | 模式指示背景色 |
| `candidates` | `[CandidateItem]` | `[]` | 候选词列表 |
| `associations` | `[CandidateItem]` | `[]` | 联想词列表 |
| `show-associations` | `bool` | `false` | `true` 显示联想词，`false` 显示候选词 |
| `has-prev-page` | `bool` | `false` | 是否有上一页候选 |
| `has-next-page` | `bool` | `false` | 是否有下一页候选 |

### 内部状态 (Internal)

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `cursor-visible` | `in-out bool` | `true` | 光标闪烁状态（内部 Timer 每 530ms 翻转） |
| `active-key` | `in-out string` | `""` | 当前按下的按键 ID（内部 120ms 后自动清除） |

## 回调 (Callbacks)

| 回调 | 参数 | 说明 |
|------|------|------|
| `key-pressed(string)` | 按键 ID | 用户按下虚拟键盘按键（不含 `hide_key`） |
| `candidate-selected(int)` | 序号 | 用户点击候选词 |
| `association-selected(string)` | 联想词 | 用户点击联想词 |
| `prev-page()` | — | 用户点击上一页 |
| `next-page()` | — | 用户点击下一页 |
| `hide-keyboard()` | — | 用户点击"收起"键 |

## 按键 ID 参考

### 字母键盘模式 (`symbol-mode: false`)

| 按键 | ID |
|------|-----|
| 字母键 | `q` `w` `e` `r` `t` `y` `u` `i` `o` `p` `a` `s` `d` `f` `g` `h` `j` `k` `l` `z` `x` `c` `v` `b` `n` `m` |
| 功能键 | `caps_lock` `enter` `mode_en` `mode_num` `mode_cn` `space` `backspace` `hide_key` |

### cn-method — 固定中文输入方式

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `cn-method` | `in string` | `"full"` | 启动时锁定，运行时不可切换。`"full"` 为全拼，`"double"` 为双拼 |

mode_cn 按钮标签始终显示当前锁定的方式（`"全拼"` 或 `"双拼"`），按下后从英文/符号模式**返回**中文模式。不会在全拼和双拼之间切换。

启动示例：
```rust
// 启动全拼
app::run(5, false);
// 启动双拼
app::run(5, true);

// 或命令行：
// cargo run -- 5         → 全拼
// cargo run -- 5 double  → 双拼
```

### 符号键盘模式 (`symbol-mode: true`)

| 按键 | ID |
|------|-----|
| 数字键 | `0` `1` `2` `3` `4` `5` `6` `7` `8` `9` |
| 中文标点 | `，` `。` `？` `！` `、` `：` `；` `"` `@` |
| 符号键 | `~` `￥` `（` `）` `【` `】` `《` `》` |
| 功能键 | `enter` `mode_lang` `mode_cn` `space` `backspace` `hide_key` |

## 最小使用示例

```slint
import { KeyboardPanel } from "ime/ui/keyboard.slint";

export component MyApp inherits Window {
    in-out property <string> output-text: "";
    in property <bool> show-ime: false;

    callback key-pressed(string);
    callback hide-keyboard();

    if show-ime: KeyboardPanel {
        output-text <=> root.output-text;

        key-pressed(id) => { root.key-pressed(id); }
        hide-keyboard() => { root.hide-keyboard(); }
    }
}
```

## 完整对接示例（Rust 端）

参考本项目中 `src/app.rs` 和 `src/ime/layout.rs` 的实现：

1. 用 `KbLayout::new(width, height)` 计算按键尺寸
2. 调用 `update_keyboard_config(&window, &layout)` 推送尺寸属性
3. 每次引擎状态变化后调用 `update_ui(&window, &engine)` 推送数据
4. 实现 `key-pressed` 回调，将按键 ID 传给 IME 引擎处理
5. 实现 `hide-keyboard` 回调，设置 `show_ime: false`

## 布局计算参考

控件内部结构（从上到下）：

```
┌─────────────────────────────────────┐
│ 编辑区 (40px + padding)             │  ← 显示 output-text + 闪烁光标
├─────────────────────────────────────┤
│ 候选栏 (36px)                       │  ← mode-indicator + candidates
├─────────────────────────────────────┤
│ Row 1: 10 个普通键                  │
│ Row 2: 9 个普通键                   │
│ Row 3: 控制键 + 7 普通键 + 控制键   │
│ Row 4: 模式键×2 + 空格 + 退格 + 收起│
└─────────────────────────────────────┘
```

总高度 = 编辑区 (~62px) + 候选栏 (36px) + 4 行按键

`KbLayout::new()` 根据 `kb-height` 和 `kb-width` 自动计算 `key-h`、`key-w`、`mode-w`、`space-w`、`ctrl-w`。
