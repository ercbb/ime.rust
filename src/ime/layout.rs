use crate::ime::engine::{ImeEngine, InputMode};
use crate::{CandidateItem, MainWindow};

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
        let edit_h = 62.0; // TextEdit area (40px min + 8px pad + 2px border + 12px layout pad)
        let candidate_h = 36.0;
        let kb_area = height - edit_h - candidate_h;
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

    let (indicator, color, tag) = match engine.mode {
        InputMode::ChineseFull => ("中", slint::Color::from_rgb_u8(0, 100, 180), "full"),
        InputMode::ChineseDouble => ("双", slint::Color::from_rgb_u8(0, 100, 180), "double"),
        InputMode::English => ("英", slint::Color::from_rgb_u8(40, 140, 50), "english"),
        InputMode::Symbols => ("数", slint::Color::from_rgb_u8(180, 90, 0), "symbols"),
    };
    window.set_mode_indicator(indicator.into());
    window.set_mode_color(color);
    window.set_symbol_mode(engine.mode == InputMode::Symbols);
    window.set_caps_lock(engine.caps_lock);
    window.set_mode_tag(tag.into());

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

/// 推送键盘布局尺寸到 MainWindow（仅在初始化时调用）
pub fn update_keyboard_config(window: &MainWindow, layout: &KbLayout) {
    window.set_key_height(layout.key_h);
    window.set_key_gap(layout.gap);
    window.set_key_w(layout.key_w);
    window.set_mode_w(layout.mode_w);
    window.set_space_w(layout.space_w);
    window.set_ctrl_w(layout.ctrl_w);
}
