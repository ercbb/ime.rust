use crate::ime::dict_core;
use crate::ime::engine::ImeEngine;
use crate::ime::layout::{self, KbLayout};
use crate::MainWindow;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, SharedString};

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
    layout::update_keyboard(&main_window, &engine, &layout);
    layout::update_ui(&main_window, &engine);

    // Wrap engine in RefCell for shared mutable access from callbacks
    let engine_rc = Rc::new(RefCell::new(engine));
    let window_weak = main_window.as_weak();

    // === Show/hide IME callbacks ===
    main_window.on_show_keyboard({
        let window_weak = window_weak.clone();
        move || {
            if let Some(win) = window_weak.upgrade() {
                win.set_show_ime(true);
            }
        }
    });

    main_window.on_hide_keyboard({
        let window_weak = window_weak.clone();
        move || {
            if let Some(win) = window_weak.upgrade() {
                win.set_show_ime(false);
            }
        }
    });

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

            if key_str == "mode_lang" {
                engine.cycle_lang_mode();
            } else if key_str == "mode_cn" {
                engine.cycle_cn_mode();
            } else if key_str == "caps_lock" {
                engine.toggle_caps_lock();
            } else {
                engine.process_key(key_str);
            }

            if let Some(win) = window_weak.upgrade() {
                layout::update_ui(&win, &engine);
                layout::update_keyboard(&win, &engine, &layout);
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
                layout::update_ui(&win, &engine);
                layout::update_keyboard(&win, &engine, &layout);
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
                layout::update_ui(&win, &engine);
                layout::update_keyboard(&win, &engine, &layout);
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
