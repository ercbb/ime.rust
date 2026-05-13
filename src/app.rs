use crate::ime::dict_core;
use crate::ime::engine::{ImeEngine, InputMode};
use crate::ime::layout::{self, KbLayout};
use crate::MainWindow;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, SharedString};

fn set_box_text(win: &MainWindow, idx: i32, text: &str) {
    match idx {
        0 => win.set_text_english(text.into()),
        1 => win.set_text_symbols(text.into()),
        2 => win.set_text_full(text.into()),
        3 => win.set_text_double(text.into()),
        _ => {}
    }
}

pub fn run(candidate_count: usize, cn_double: bool) {
    let main_window = MainWindow::new().unwrap();

    // Keyboard panel size (configurable)
    let kb_width: f32 = 600.0;
    let kb_height: f32 = 320.0;
    let kb_bottom_margin: f32 = 40.0;
    let kb_opacity: f32 = 0.85;
    let layout = Rc::new(KbLayout::new(kb_width, kb_height));

    // Set layout properties on window
    main_window.set_kb_width(kb_width);
    main_window.set_kb_height(kb_height);
    main_window.set_kb_bottom_margin(kb_bottom_margin);
    main_window.set_kb_opacity(kb_opacity);
    layout::update_keyboard_config(&main_window, &layout);

    // Set initial Chinese method
    let initial_cn: &str = if cn_double { "double" } else { "full" };
    main_window.set_cn_method(initial_cn.into());

    // Chinese method preference (stable, only changed by clicking 全拼/双拼 boxes)
    let cn_pref: Rc<RefCell<String>> = Rc::new(RefCell::new(initial_cn.to_string()));

    // Initialize IME engine with configured Chinese mode
    let mut engine = ImeEngine::new();
    if cn_double {
        engine.toggle_mode(InputMode::ChineseDouble);
    }
    engine.page_size = candidate_count.max(1);
    let dict = dict_core::load_core_dict();
    engine.set_dictionary(dict);

    // Set initial state
    layout::update_ui(&main_window, &engine);

    // Wrap engine in RefCell for shared mutable access from callbacks
    let engine_rc = Rc::new(RefCell::new(engine));
    let window_weak = main_window.as_weak();

    // Per-box text buffers and active-box tracking
    let buffers: Rc<RefCell<[String; 4]>> = Rc::new(RefCell::new([
        "good boy".to_string(),
        "12345".to_string(),
        "全拼".to_string(),
        "双拼".to_string(),
    ]));
    let active_box: Rc<RefCell<i32>> = Rc::new(RefCell::new(-1));

    // === Show/hide IME callbacks ===
    main_window.on_show_keyboard_with_mode({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let buffers = buffers.clone();
        let active_box = active_box.clone();
        let cn_pref = cn_pref.clone();
        move |mode: SharedString| {
            let (new_mode, box_idx): (InputMode, i32) = match mode.as_str() {
                "english" => (InputMode::English, 0),
                "symbols" => (InputMode::Symbols, 1),
                "full" => (InputMode::ChineseFull, 2),
                "double" => (InputMode::ChineseDouble, 3),
                _ => return,
            };

            let old_idx = *active_box.borrow();
            let mut engine = engine_rc.borrow_mut();

            // Save current output to previous box's buffer
            if old_idx >= 0 && old_idx < 4 {
                buffers.borrow_mut()[old_idx as usize] = engine.output_text.clone();
                if let Some(win) = window_weak.upgrade() {
                    set_box_text(&win, old_idx, &engine.output_text);
                }
            }

            // Load new box's text into engine
            engine.output_text = buffers.borrow()[box_idx as usize].clone();
            engine.toggle_mode(new_mode);
            engine.cursor_pos = engine.output_text.chars().count();
            *active_box.borrow_mut() = box_idx;

            if let Some(win) = window_weak.upgrade() {
                win.set_output_text(engine.output_text.as_str().into());
                win.set_active_box(box_idx);
                match new_mode {
                    InputMode::ChineseFull => {
                        win.set_cn_method("full".into());
                        *cn_pref.borrow_mut() = "full".to_string();
                    }
                    InputMode::ChineseDouble => {
                        win.set_cn_method("double".into());
                        *cn_pref.borrow_mut() = "double".to_string();
                    }
                    _ => {}
                }
                layout::update_ui(&win, &engine);
                win.set_show_ime(true);
            }
        }
    });

    main_window.on_show_keyboard({
        let window_weak = window_weak.clone();
        move || {
            if let Some(win) = window_weak.upgrade() {
                win.set_show_ime(true);
            }
        }
    });

    main_window.on_hide_keyboard({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let buffers = buffers.clone();
        let active_box = active_box.clone();
        move || {
            if let Some(win) = window_weak.upgrade() {
                // 放弃：恢复 engine.output_text 为原始值
                let mut engine = engine_rc.borrow_mut();
                let idx = *active_box.borrow();
                if idx >= 0 && idx < 4 {
                    engine.output_text = buffers.borrow()[idx as usize].clone();
                }
                engine.cursor_pos = engine.output_text.chars().count();
                win.set_show_ime(false);
            }
        }
    });

    // === Virtual keyboard key-pressed callback ===
    main_window.on_key_pressed({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let buffers = buffers.clone();
        let active_box = active_box.clone();
        let cn_pref = cn_pref.clone();
        move |key_id: SharedString| {
            let mut engine = engine_rc.borrow_mut();
            let key_str = key_id.as_str();

            if key_str == "mode_en" {
                engine.toggle_mode(crate::ime::engine::InputMode::English);
            } else if key_str == "mode_num" {
                engine.toggle_mode(crate::ime::engine::InputMode::Symbols);
            } else if key_str == "mode_cn" {
                let target = if cn_pref.borrow().as_str() == "double" {
                    InputMode::ChineseDouble
                } else {
                    InputMode::ChineseFull
                };
                engine.toggle_mode(target);
            } else if key_str == "caps_lock" {
                engine.toggle_caps_lock();
            } else if key_str == "cursor_left" {
                if engine.cursor_pos > 0 {
                    engine.cursor_pos -= 1;
                }
            } else if key_str == "cursor_right" {
                let len = engine.output_text.chars().count();
                if engine.cursor_pos < len {
                    engine.cursor_pos += 1;
                }
            } else if key_str == "hide_key" {
                // 放弃：不做任何 engine 操作
            } else {
                engine.process_key(key_str);
            }

            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);

                if key_str == "enter" {
                    // 回车：确认写回并隐藏
                    let idx = *active_box.borrow();
                    if idx >= 0 && idx < 4 {
                        buffers.borrow_mut()[idx as usize] = engine.output_text.clone();
                        set_box_text(&win, idx, &engine.output_text);
                    }
                    win.set_show_ime(false);
                } else if key_str == "hide_key" {
                    // 放弃：恢复原始值，不写回，直接隐藏
                    let idx = *active_box.borrow();
                    if idx >= 0 && idx < 4 {
                        engine.output_text = buffers.borrow()[idx as usize].clone();
                    }
                    engine.cursor_pos = engine.output_text.chars().count();
                    win.set_show_ime(false);
                }
            }
        }
    });

    // === Candidate selected callback ===
    main_window.on_candidate_selected({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        move |idx: i32| {
            let mut engine = engine_rc.borrow_mut();
            let key = (idx + 1).to_string();
            engine.process_key(&key);
            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);
            }
        }
    });

    // === Association selected callback ===
    main_window.on_association_selected({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        move |text: SharedString| {
            let mut engine = engine_rc.borrow_mut();
            engine.select_association(text.as_str());
            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);
            }
        }
    });

    // === Page navigation callbacks ===
    main_window.on_prev_page({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        move || {
            let mut engine = engine_rc.borrow_mut();
            engine.prev_page();
            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);
            }
        }
    });

    main_window.on_next_page({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        move || {
            let mut engine = engine_rc.borrow_mut();
            engine.next_page();
            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);
            }
        }
    });

    main_window.run().unwrap();
}
