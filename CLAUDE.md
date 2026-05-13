# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Software Stack

- **Slint**: 1.15.1 (upgraded from 1.9)
- **Rust**: MSRV 1.92
- **Embedded target**: Linux 6.18 + simple-framebuffer + evdev + ALSA

## Build Commands

```bash
# Desktop (Ubuntu) — default feature, winit + femtovg
cargo build
cargo run          # optional: cargo run -- 8  (set candidate count)

# ARM embedded — linuxkms + software renderer
cargo build --no-default-features --features embedded \
    --target armv7-unknown-linux-musleabihf --release

# Tests
cargo test
cargo test test_final_z_ei
```

## Feature Flags

`Cargo.toml` defines two mutually exclusive features:

- **`desktop`** (default): `slint/backend-winit` + `renderer-femtovg` — for Ubuntu development
- **`embedded`**: `slint/backend-linuxkms-noseat` + `renderer-software` — for T113-S3 ARM board

Run on device with `SLINT_BACKEND_LINUXFB=1 ./wheel-rust` (fbdev) or `SLINT_BACKEND=linuxkms ./wheel-rust` (DRM/KMS).

## Architecture

Slint UI + Rust IME engine, connected via Slint callbacks.

### Data Flow

```
Key tap (Slint TouchArea)
  → app.rs on_key_pressed callback
    → reads active_box buffers to track per-box text state
    → engine.process_key(key_str)
      → dispatch by InputMode { ChineseFull | ChineseDouble | English | Symbols }
      → Chinese modes: append to input_buffer → update_candidates()
        → parse buffer to syllables (pinyin.rs or double_pinyin.rs)
        → dict.lookup_chars() for single-char candidates
        → dict.lookup_phrases_exact() + dict.lookup_phrases_prefix() for multi-char
        → merge order: exact phrases → chars → prefix phrases (multi-syllable)
                        chars → phrases (single syllable)
        → user_dict.freq_boost() applied
        → sorted within each group, groups kept separate
      → select_candidate(idx): insert_at_cursor selected text, record usage, trigger associations
      → insert_at_cursor / delete_before_cursor: all edits operate at engine.cursor_pos
  → layout::update_ui() pushes state to Slint properties
    → output_text split at cursor_pos into text-before-cursor / text-after-cursor
    → custom cursor rendered as a 2px Rectangle between the two text segments
```

### Write-back Policy

| Operation | Main screen (per-box buffers) |
|-----------|-------------------------------|
| Any key input / candidate selection / association | **No write-back** — only keyboard panel updates |
| Press Enter | Write-back `engine.output_text` to buffers, hide keyboard |
| Press Discard / bottom margin click | Restore `engine.output_text` from buffers (discard edits), hide keyboard |
| Switch to another box (touch a different mode box) | Save to previous box (treated as confirm) |

### Cursor System

- `engine.cursor_pos: usize` — character offset in `output_text`
- `insert_at_cursor(text)`: inserts at cursor position, advances cursor past inserted text
- `delete_before_cursor()`: deletes one char before cursor (no-op at position 0)
- `set_cursor(pos)`: clamp cursor to valid range (reserved for Slint to Engine sync)
- Cursor moved by direction keys on Row 4
- Slint 1.15.1 `TextInput` does not expose character-level cursor position; the visual cursor is rendered via custom `HorizontalLayout( Text + Rectangle(2px) + Text )` in the edit area

### Keyboard Layout (Row 4)

```
英文  数字  全拼/双拼  空格  left-arrow  right-arrow      [spacer]      放弃
```

- Row 2 ends with backspace (moved from Row 4)
- Row 3 Enter width = `key-w * 2` to right-align with Row 2 backspace
- Row 2 and 3 alignment: `start` (left-aligned)
- Discard button right-aligned via spacer

### Module Roles (`src/ime/`)

| Module | Role |
|--------|------|
| `engine.rs` | Central `ImeEngine` state machine: mode, buffers, candidates, cursor, pagination, key dispatch |
| `pinyin.rs` | Full pinyin parser — greedy longest-match against syllable table |
| `double_pinyin.rs` | Natural Code double pinyin decoder — 2 chars per syllable with disambiguation rules |
| `syllable_table.rs` | `VALID_SYLLABLES` const array, `is_valid_syllable()` / `is_valid_prefix()` |
| `dict.rs` | `Dictionary` with chars + phrases HashMaps; `lookup_chars()`, `lookup_phrases_exact()`, `lookup_phrases_prefix()` |
| `dict_core.rs` | Loads embedded dictionaries via `include_str!` |
| `association.rs` | Static char to word association map for next-word suggestions |
| `user_dict.rs` | JSON-persisted frequency tracker at `user_data/user_dict.json` |

### Slint UI Layer

- `ui/main.slint` — MainWindow: text display area + keyboard panel (800x480)
- `src/ime/ui/keyboard.slint` — Reusable components: `KeyButton`, `CandidateBar`, `KeyboardRows`, `KeyboardPanel`
- `build.rs` compiles Slint files with `fluent-light` style
- `app.rs` bridges Rust to Slint via `Rc<RefCell<ImeEngine>>` and callbacks

### Dictionary Format

Embedded text files in `src/ime/dicts/`:
- `pinyin_chars.txt`: `pinyin hanzi frequency` per line — 6763 chars (GB2312 Level 1 3755 + Level 2 3008)
- `phrases.txt`: `ni'hao 你好 1000` — multi-syllable uses `'` as delimiter, 1536 phrases

## Natural Code Double Pinyin

`double_pinyin.rs` implements the Natural Code scheme:

- Special initial mappings: `v` to zh, `i` to ch, `u` to sh
- `combine_initial_final()` handles dual-final disambiguation based on initial:
  - `d` key: uang to iang after j/q/x
  - `w` key: ia to ua after g/k/h/zh/ch/sh
  - `y` key: ing to uai after g/k/h/zh/ch/sh
  - `s` key: ong to iong after j/q/x
  - `o` key: uo to o after b/p/m/f (pinyin spelling rule)
  - j/q/x + u/un/uan/ue to u-umlaut variants; n/l + u/ue to nv/lv

## Target Hardware

Allwinner T113-S3 (Cortex-A7, ARMv7-A hard-float, musl libc), 800x480 LCD, Tina Linux.
Cross-compilation guide in `CROSS_COMPILE.md`.

## Improvements (2026-05-14)

### Slint Upgrade
- Upgraded from Slint 1.9 to 1.15.1 for newer features and bug fixes.

### Cursor Editing
- Added `cursor_pos` tracking in `ImeEngine` — all text insertion/deletion operates at cursor position instead of always appending to end.
- `insert_at_cursor()` and `delete_before_cursor()` helper methods handle byte-position insertion for CJK safety.
- Custom visual cursor rendered as a 2px `Rectangle` between `Text` segments (workaround for Slint 1.x lacking character-level `cursor-position`).
- Left/right direction keys added to Row 4 for cursor navigation.

### Keyboard Layout
- Moved backspace from Row 4 to the rightmost position of Row 2.
- Row 2 and Row 3 alignment changed to `start` (left-aligned).
- Enter width increased to `key-w * 2` to right-align with Row 2 backspace.
- Discard button right-aligned via spacer `Rectangle`.
- Chinese method button width reduced to match English/Number button width.

### Write-back Policy
- Keyboard edits no longer immediately write to the main screen text boxes.
- Enter confirms and writes back; Discard / bottom margin discards and restores original text.
- Candidate selection and association selection no longer trigger write-back (only keyboard panel updates).

### Dictionary Upgrade
- `pinyin_chars.txt` replaced with full GB2312 Level 1 (3755 chars) + Level 2 (3008 chars) = 6763 total unique characters.
- Frequency values: Level 1 chars 500 to 10, Level 2 chars 200 to 5, maintaining natural sort within groups.
- `phrases.txt` expanded with 66 common words (total 1536 phrases).
- Pinyin auto-annotated via `pypinyin` (tone numbers stripped for grouping).

### Candidate Ordering
- Multi-syllable input: exact-match phrases first, then single chars, then prefix-match phrases.
- Single-syllable input: single chars first, then phrases.
- No cross-group global sort — per-group frequency sort preserves structural ordering.

### Dict API
- Added `lookup_phrases_exact(pinyin_key)` and `lookup_phrases_prefix(pinyin_key)` to `Dictionary` for phrase-only queries (separate from mixed char+phrase lookups).

### Number Mode
- Enter in symbol/number mode no longer inserts newline — behaves same as English mode (confirm write-back).
