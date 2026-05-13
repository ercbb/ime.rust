# Wheel IME

A Chinese input method engine (IME) with a custom virtual keyboard, built with Rust and Slint.

## Features

- **Full & Double Pinyin** — full pinyin and Natural Code (自然码) double pinyin input
- **English & Symbol modes** — direct text entry and symbol/number input
- **Cursor editing** — insert/delete at any position via direction keys and custom visual cursor
- **GB2312 dictionary** — complete Level 1 (3755 chars) + Level 2 (3008 chars) character set
- **Phrase input** — multi-character phrase lookup with prefix matching
- **User dictionary** — frequency-based learning persisted to `user_data/user_dict.json`
- **Per-box buffers** — four independent text boxes (English, Symbols, Full Pinyin, Double Pinyin)

## Stack

- **Slint** 1.15.1
- **Rust** MSRV 1.92
- **Target**: Allwinner T113-S3, Linux 6.18, simple-framebuffer + evdev + ALSA

## Build

```bash
# Desktop
cargo build && cargo run

# Embedded ARM
cargo build --no-default-features --features embedded \
    --target armv7-unknown-linux-musleabihf --release
```

## License

MIT
