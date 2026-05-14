use crate::ime::engine::{ImeEngine, InputMode};
use crate::{CandidateItem, MainWindow};

/// 推送引擎状态到 MainWindow Slint 属性
pub fn update_ui(window: &MainWindow, engine: &ImeEngine) {
    window.set_output_text(engine.output_text.as_str().into());

    let before: String = engine
        .output_text
        .chars()
        .take(engine.cursor_pos)
        .collect();
    let after: String = engine
        .output_text
        .chars()
        .skip(engine.cursor_pos)
        .collect();
    window.set_text_before_cursor(before.as_str().into());
    window.set_text_after_cursor(after.as_str().into());

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
