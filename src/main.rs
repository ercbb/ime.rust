#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod app;
mod ime;

fn main() {
    let candidate_count = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(5);
    let cn_double = std::env::args()
        .nth(2)
        .map(|s| s == "double")
        .unwrap_or(false);
    app::run(candidate_count, cn_double);
}
