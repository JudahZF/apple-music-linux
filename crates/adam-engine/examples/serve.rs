//! Runs the engine as `adam engine` will, for local testing:
//! `cargo run -p adam-engine --example serve -- --api 20125 [--tls-identity-stdin]`.
//!
//! With `--tls-identity-stdin` the TLS identity is read as PEM from standard input (see
//! [`adam_engine::TlsSource::Stdin`]).

fn main() -> anyhow::Result<()> {
    let mut port = None;
    let mut tls = adam_engine::TlsSource::Generate;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--api" => port = Some(args.next().unwrap_or_default().parse()?),
            "--tls-identity-stdin" => tls = adam_engine::TlsSource::Stdin,
            _ => anyhow::bail!("usage: serve --api <port> [--tls-identity-stdin]"),
        }
    }
    let Some(port) = port else { anyhow::bail!("usage: serve --api <port> [--tls-identity-stdin]") };
    adam_engine::prepare_process();
    tokio::runtime::Runtime::new()?.block_on(adam_engine::run(adam_engine::Options { port, tls }))
}
