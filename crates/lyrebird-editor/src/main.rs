#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod editor;

fn main() {
    lyrebird_renderer::run::<crate::LyrebirdEditor>().unwrap();
}