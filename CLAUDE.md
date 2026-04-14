# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

# Single test
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
    → engine.process_key(key_str)
      → dispatch by InputMode { ChineseFull | ChineseDouble | English | Symbols }
      → Chinese modes: append to input_buffer → update_candidates()
        → parse buffer to syllables (pinyin.rs or double_pinyin.rs)
        → dict.lookup_exact() + dict.lookup_prefix()
        → user_dict.freq_boost() applied
        → sorted Vec<Candidate>
      → select_candidate(idx): append to output_text, record usage, trigger associations
  → update_ui() + update_keyboard() push state back to Slint properties
```

### Module Roles (`src/ime/`)

| Module | Role |
|--------|------|
| `engine.rs` | Central `ImeEngine` state machine: mode, buffers, candidates, pagination, key dispatch |
| `pinyin.rs` | Full pinyin parser — greedy longest-match against syllable table |
| `double_pinyin.rs` | Natural Code (自然码) double pinyin decoder — 2 chars per syllable with disambiguation rules |
| `syllable_table.rs` | `VALID_SYLLABLES` const array, `is_valid_syllable()` / `is_valid_prefix()` |
| `dict.rs` | `Dictionary` with chars + phrases HashMaps, `lookup_exact()` and `lookup_prefix()` |
| `dict_core.rs` | Loads embedded dictionaries via `include_str!` |
| `association.rs` | Static char→word association map for next-word suggestions |
| `user_dict.rs` | JSON-persisted frequency tracker at `user_data/user_dict.json` |

### Slint UI Layer

- `ui/main.slint` — MainWindow: text display area + keyboard panel (800×480)
- `src/ime/ui/keyboard.slint` — Reusable components: `KeyButton`, `CandidateBar`, `KeyboardRows`
- `build.rs` compiles Slint files with `fluent-light` style
- `app.rs` bridges Rust ↔ Slint via `Rc<RefCell<ImeEngine>>` and five callbacks

### Dictionary Format

Embedded text files in `src/ime/dicts/`:
- `pinyin_chars.txt`: `pinyin hanzi frequency` per line
- `phrases.txt`: `ni'hao 你好 1000` — multi-syllable uses `'` as delimiter

## Natural Code Double Pinyin

`double_pinyin.rs` implements the 自然码 scheme:

- Special initial mappings: `v`→zh, `i`→ch, `u`→sh
- `combine_initial_final()` handles dual-final disambiguation based on initial:
  - `d` key: uang → iang after j/q/x
  - `w` key: ia → ua after g/k/h/zh/ch/sh
  - `y` key: ing → uai after g/k/h/zh/ch/sh
  - `s` key: ong → iong after j/q/x
  - `o` key: uo → o after b/p/m/f (pinyin spelling rule)
  - j/q/x + u/un/uan/ue → ü variants; n/l + u/ue → nv/lv

## Target Hardware

Allwinner T113-S3 (Cortex-A7, ARMv7-A hard-float, musl libc), 800×480 LCD, Tina Linux.
Cross-compilation guide in `CROSS_COMPILE.md`.
