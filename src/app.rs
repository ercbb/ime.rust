use crate::ime::dict_core;
use crate::ime::engine::{ImeEngine, InputMode};

use std::cell::RefCell;
use std::rc::Rc;

use slint::SharedString;

slint::include_modules!();

/// 键盘面板布局配置
struct KbLayout {
    width: f32,
    height: f32,
    gap: f32,
    key_w: f32,
    key_h: f32,
    mode_w: f32,
    space_w: f32,
    ctrl_w: f32,
}

impl KbLayout {
    fn new(width: f32, height: f32) -> Self {
        let gap = 4.0;
        let candidate_h = 36.0;
        let kb_area = height - candidate_h;
        // 5 rows, 4 gaps
        let key_h = ((kb_area - 5.0 * gap) / 5.0).floor();
        // Number row: 10 keys, 9 gaps
        let key_w = ((width - 10.0 * gap - 2.0 * gap) / 10.0).floor();
        Self {
            width,
            height,
            gap,
            key_w,
            key_h,
            mode_w: (key_w * 1.3).floor(),
            space_w: (key_w * 3.5).floor(),
            ctrl_w: (key_w * 1.15).floor(),
        }
    }
}

pub fn run(candidate_count: usize) {
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
    main_window.set_key_height(layout.key_h);
    main_window.set_key_gap(layout.gap);
    main_window.set_kb_opacity(kb_opacity);

    // Initialize IME engine
    let mut engine = ImeEngine::new();
    engine.page_size = candidate_count.max(1);
    let dict = dict_core::load_core_dict();
    engine.set_dictionary(dict);

    // Set initial state
    update_keyboard(&main_window, &engine, &layout);
    update_ui(&main_window, &engine);

    // Wrap engine in RefCell for shared mutable access from callbacks
    let engine_rc = Rc::new(RefCell::new(engine));
    let window_weak = main_window.as_weak();

    // === Virtual keyboard key-pressed callback ===
    let flash_timer = Rc::new(slint::Timer::default());
    main_window.on_key_pressed({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let layout = layout.clone();
        let flash_timer = flash_timer.clone();
        move |key_id: SharedString| {
            let mut engine = engine_rc.borrow_mut();
            let key_str = key_id.as_str();

            if let Some(win) = window_weak.upgrade() {
                win.set_active_key(key_id.clone());
            }

            if key_str == "mode_cycle" {
                engine.cycle_mode();
            } else if key_str == "caps_lock" {
                engine.toggle_caps_lock();
            } else if key_str == "symbol_mode" {
                engine.toggle_mode(InputMode::Symbols);
            } else {
                engine.process_key(key_str);
            }

            if let Some(win) = window_weak.upgrade() {
                update_ui(&win, &engine);
                update_keyboard(&win, &engine, &layout);
                win.set_active_key(key_id.clone());
            }

            let w = window_weak.clone();
            flash_timer.start(
                slint::TimerMode::SingleShot,
                std::time::Duration::from_millis(120),
                move || {
                    if let Some(win) = w.upgrade() {
                        win.set_active_key(SharedString::from(""));
                    }
                },
            );
        }
    });

    // === Candidate selected callback ===
    main_window.on_candidate_selected({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let layout = layout.clone();
        move |idx: i32| {
            let mut engine = engine_rc.borrow_mut();
            let key = (idx + 1).to_string();
            engine.process_key(&key);
            if let Some(win) = window_weak.upgrade() {
                update_ui(&win, &engine);
                update_keyboard(&win, &engine, &layout);
            }
        }
    });

    // === Association selected callback ===
    main_window.on_association_selected({
        let engine_rc = engine_rc.clone();
        let window_weak = window_weak.clone();
        let layout = layout.clone();
        move |text: SharedString| {
            let mut engine = engine_rc.borrow_mut();
            engine.select_association(text.as_str());
            if let Some(win) = window_weak.upgrade() {
                update_ui(&win, &engine);
                update_keyboard(&win, &engine, &layout);
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
                update_ui(&win, &engine);
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
                update_ui(&win, &engine);
            }
        }
    });

    main_window.run().unwrap();
}

fn update_ui(window: &MainWindow, engine: &ImeEngine) {
    window.set_output_text(engine.output_text.as_str().into());
    window.set_input_buffer(engine.input_buffer.as_str().into());

    let (indicator, color) = match engine.mode {
        InputMode::ChineseFull => ("中", slint::Color::from_rgb_u8(0, 100, 180)),
        InputMode::ChineseDouble => ("双", slint::Color::from_rgb_u8(0, 100, 180)),
        InputMode::English => ("英", slint::Color::from_rgb_u8(40, 140, 50)),
        InputMode::Symbols => ("符", slint::Color::from_rgb_u8(180, 90, 0)),
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

fn make_key_row(
    keys: &[&str],
    is_symbol: bool,
    key_w: f32,
) -> Vec<KeyValue> {
    keys.iter()
        .map(|&k| KeyValue {
            label: if is_symbol {
                ImeEngine::symbol_for_key(k).unwrap_or(k).into()
            } else {
                k.into()
            },
            id: k.into(),
            key_width: key_w,
        })
        .collect()
}

fn update_keyboard(window: &MainWindow, engine: &ImeEngine, layout: &KbLayout) {
    let is_symbol = engine.mode == InputMode::Symbols;
    let w = layout.key_w;

    let row1 = make_key_row(&["1","2","3","4","5","6","7","8","9","0"], is_symbol, w);
    let row2 = make_key_row(&["q","w","e","r","t","y","u","i","o","p"], is_symbol, w);
    let row3 = make_key_row(&["a","s","d","f","g","h","j","k","l"], is_symbol, w);

    // Row 4: [caps] z x c v b n m [enter]
    let caps_label = if engine.caps_lock { "CAP" } else { "cap" };
    let mut row4: Vec<KeyValue> = vec![
        KeyValue { label: caps_label.into(), id: "caps_lock".into(), key_width: layout.ctrl_w },
    ];
    row4.extend(make_key_row(&["z","x","c","v","b","n","m"], is_symbol, w));
    row4.push(KeyValue { label: "\u{23ce}".into(), id: "enter".into(), key_width: layout.ctrl_w });

    window.set_row1_keys(slint::ModelRc::from(row1.as_slice()));
    window.set_row2_keys(slint::ModelRc::from(row2.as_slice()));
    window.set_row3_keys(slint::ModelRc::from(row3.as_slice()));
    window.set_row4_keys(slint::ModelRc::from(row4.as_slice()));

    let mode_label = match engine.mode {
        InputMode::ChineseFull => "全拼",
        InputMode::ChineseDouble => "双拼",
        InputMode::English => "英文",
        InputMode::Symbols => "符号",
    };
    let func: Vec<KeyValue> = vec![
        KeyValue { label: mode_label.into(), id: "mode_cycle".into(), key_width: layout.mode_w },
        KeyValue { label: "空格".into(), id: "space".into(), key_width: layout.space_w },
        KeyValue { label: "\u{232b}".into(), id: "backspace".into(), key_width: layout.ctrl_w },
    ];
    window.set_func_keys(slint::ModelRc::from(func.as_slice()));
}
