#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod app;
pub mod logger;

fn main() {
    #[cfg(debug_assertions)]
    logger::init_logger();

    logger::init_sentry();

    app::init_app();
}
