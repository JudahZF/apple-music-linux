//! Runs the sign-in window the way `adam signin` will: logs on stderr, one JSON line on stdout.
//!
//!   cargo run -p adam-signin --example signin -- PROFILE_DIR [STOREFRONT]
//!   cargo run -p adam-signin --example signin -- --sign-out PROFILE_DIR
//!
//! `RUST_LOG=adam_signin=debug` also logs every navigation and the cookie / localStorage key
//! names (never values).
use std::process::ExitCode;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "adam_signin=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [flag, dir] if flag == "--sign-out" => match adam_signin::sign_out(dir.as_ref()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        [dir, rest @ ..] if rest.len() <= 1 => adam_signin::run(&adam_signin::Options {
            profile_dir: dir.into(),
            storefront: rest.first().cloned(),
        }),
        _ => {
            eprintln!("usage: signin PROFILE_DIR [STOREFRONT] | --sign-out PROFILE_DIR");
            ExitCode::from(2)
        }
    }
}
