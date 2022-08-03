use std::env;
use std::io::Write;

use dotenv::dotenv;
use env_logger;
use sentry;

pub fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "[{} {} {}] {} {}:{}",
                ts,
                record.level(),
                record.target(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            )
        })
        .init();
}

pub fn init_sentry() {
    dotenv().ok();

    let sentry_dsn = if cfg!(debug_assertions) {
        dotenv::var("SENTRY_DSN").unwrap()
    } else {
        env!("SENTRY_DSN").to_string()
    };

    let _guard = sentry::init((
        sentry_dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            sample_rate: 0.2,
            traces_sample_rate: 0.2,
            ..Default::default()
        },
    ));
}
