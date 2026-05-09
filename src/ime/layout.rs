use crate::ime::engine::{ImeEngine, InputMode};
use crate::{CandidateItem, KeyValue, MainWindow};

/// 键盘面板布局配置
pub struct KbLayout {
    pub width: f32,
    pub height: f32,
    pub gap: f32,
    pub key_w: f32,
    pub key_h: f32,
    pub mode_w: f32,
    pub space_w: f32,
    pub ctrl_w: f32,
}

impl KbLayout {
    pub fn new(width: f32, height: f32) -> Self {
        let gap = 4.0;
        let pad = 4.0; // matches VerticalLayout padding in KeyboardRows
        let candidate_h = 36.0;
        let kb_area = height - candidate_h;
        // 4 rows: subtract 2×padding (top+bottom) + 3×gap
        let key_h = ((kb_area - 2.0 * pad - 3.0 * gap) / 4.0).floor();
        // Letter row: 10 keys, 9 gaps
        let key_w = ((width - 10.0 * gap - 2.0 * gap) / 10.0).floor();
        Self {
            width,
            height,
            gap,
            key_w,
            key_h,
            mode_w: (key_w * 2.0).floor(),
            space_w: (key_w * 3.0).floor(),
            ctrl_w: (key_w * 1.15).floor(),
        }
    }
}

/// 推送引擎状态到 MainWindow Slint 属性
pub fn update_ui(window: &MainWindow, engine: &ImeEngine) {
    window.set_output_text(engine.output_text.as_str().into());
    window.set_input_buffer(engine.input_buffer.as_str().into());

    let (indicator, color) = match engine.mode {
        InputMode::ChineseFull => ("中", slint::Color::from_rgb_u8(0, 100, 180)),
        InputMode::ChineseDouble => ("双", slint::Color::from_rgb_u8(0, 100, 180)),
        InputMode::English => ("英", slint::Color::from_rgb_u8(40, 140, 50)),
        InputMode::Symbols => ("数", slint::Color::from_rgb_u8(180, 90, 0)),
    };
    window.set_mode_indicator(indicator.into());
    window.set_mode_color(color);

    if !engine.candidates.is_empty() {
        let total = engine.candidates.len();
        let page = engine.candidate_page + 1;
        let total_pages = (total + engine.page_size - 1) / engine.page_size;
        window.set_has_prev_page(page > 1);
        window.set_has_next_page(page < total_pages);
    } else {
        window.set_has_prev_page(false);
        window.set_has_next_page(false);
    }

    let page_candidates = engine.get_current_page_candidates();
    let candidate_items: Vec<CandidateItem> = page_candidates
        .iter()
        .enumerate()
        .map(|(i, c)| CandidateItem {
            text: c.text.as_str().into(),
            index: i as i32,
        })
        .collect();
    window.set_candidates(slint::ModelRc::from(candidate_items.as_slice()));

    let assoc_items: Vec<CandidateItem> = engine
        .get_association_candidates()
        .iter()
        .take(engine.page_size)
        .enumerate()
        .map(|(i, s)| CandidateItem {
            text: s.as_str().into(),
            index: i as i32,
        })
        .collect();
    window.set_associations(slint::ModelRc::from(assoc_items.as_slice()));
    window.set_show_associations(engine.association_mode);
}

fn make_key_row(keys: &[&str], key_w: f32) -> Vec<KeyValue> {
    keys.iter()
        .map(|&k| KeyValue {
            label: k.into(),
            id: k.into(),
            key_width: key_w,
        })
        .collect()
}

pub fn update_keyboard(window: &MainWindow, engine: &ImeEngine, layout: &KbLayout) {
    let w = layout.key_w;

    if engine.mode == InputMode::Symbols {
        // Number/symbol keyboard:
        // Row 1: 0-9 number keys
        let row1: Vec<KeyValue> = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|&k| KeyValue { label: k.into(), id: k.into(), key_width: w })
            .collect();

        // Row 2: common punctuation (9 keys to match letter row 2)
        let row2: Vec<KeyValue> = ["，", "。", "？", "！", "、", "：", "；", "\"", "@"]
            .iter()
            .map(|&k| KeyValue { label: k.into(), id: k.into(), key_width: w })
            .collect();

        // Row 3: [~] more symbols [enter]
        let mut row3: Vec<KeyValue> = vec![
            KeyValue { label: "~".into(), id: "~".into(), key_width: layout.ctrl_w },
        ];
        row3.extend(
            ["￥", "（", "）", "【", "】", "《", "》"]
                .iter()
                .map(|&k| KeyValue { label: k.into(), id: k.into(), key_width: w }),
        );
        row3.push(KeyValue { label: "⏎".into(), id: "enter".into(), key_width: layout.ctrl_w });

        window.set_row1_keys(slint::ModelRc::from(row1.as_slice()));
        window.set_row2_keys(slint::ModelRc::from(row2.as_slice()));
        window.set_row3_keys(slint::ModelRc::from(row3.as_slice()));
    } else {
        let row1 = make_key_row(&["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"], w);
        let row2 = make_key_row(&["a", "s", "d", "f", "g", "h", "j", "k", "l"], w);

        // Row 3: [caps] z x c v b n m [enter]
        let caps_label = if engine.caps_lock { "CAP" } else { "cap" };
        let mut row3: Vec<KeyValue> = vec![
            KeyValue {
                label: caps_label.into(),
                id: "caps_lock".into(),
                key_width: layout.ctrl_w,
            },
        ];
        row3.extend(make_key_row(
            &["z", "x", "c", "v", "b", "n", "m"],
            w,
        ));
        row3.push(KeyValue {
            label: "\u{23ce}".into(),
            id: "enter".into(),
            key_width: layout.ctrl_w,
        });

        window.set_row1_keys(slint::ModelRc::from(row1.as_slice()));
        window.set_row2_keys(slint::ModelRc::from(row2.as_slice()));
        window.set_row3_keys(slint::ModelRc::from(row3.as_slice()));
    }

    // Row 4: mode buttons + space + backspace
    let lang_label = match engine.mode {
        InputMode::English => "英文/数字",
        InputMode::Symbols => "数字/英文",
        _ => "英文/数字",
    };
    let cn_label = match engine.mode {
        InputMode::ChineseFull => "全拼/双拼",
        InputMode::ChineseDouble => "双拼/全拼",
        _ => "全拼/双拼",
    };
    let row4: Vec<KeyValue> = vec![
        KeyValue {
            label: lang_label.into(),
            id: "mode_lang".into(),
            key_width: layout.mode_w,
        },
        KeyValue {
            label: cn_label.into(),
            id: "mode_cn".into(),
            key_width: layout.mode_w,
        },
        KeyValue {
            label: "空格".into(),
            id: "space".into(),
            key_width: layout.space_w,
        },
        KeyValue {
            label: "\u{232b}".into(),
            id: "backspace".into(),
            key_width: layout.ctrl_w,
        },
    ];
    window.set_row4_keys(slint::ModelRc::from(row4.as_slice()));

    window.set_mode_lang_active(matches!(engine.mode, InputMode::English | InputMode::Symbols));
    window.set_mode_cn_active(matches!(engine.mode, InputMode::ChineseFull | InputMode::ChineseDouble));
}
