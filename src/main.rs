//! Command-line entry point for the Nostalgia application.

// use inline_colorization::*
use std::process::exit;

fn main() {
    use tracing_subscriber::Layer;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::filter::filter_fn;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let cli_arguments = nostalgia::cli::get_cli_arguments(std::env::args_os());
    let max_level = match cli_arguments.get_count("verbose") {
        0 if cli_arguments.get_flag("validate-database") => LevelFilter::INFO,
        0 => LevelFilter::ERROR,
        1 => LevelFilter::INFO,
        _ => LevelFilter::DEBUG,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false)
                .with_writer(std::io::stdout)
                .with_filter(filter_fn(|metadata| {
                    *metadata.level() != tracing::Level::ERROR
                })),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(true)
                .with_target(false)
                .with_writer(std::io::stderr)
                .with_filter(filter_fn(|metadata| {
                    *metadata.level() == tracing::Level::ERROR
                })),
        )
        .with(max_level)
        .init();

    if let Err(error) = nostalgia::run_with_cli_arguments(cli_arguments) {
        tracing::error!("Error: {error:#}");
        exit(1)
    }
}
