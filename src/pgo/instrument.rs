use crate::cli::cli_format_path;
use crate::pgo::build::build_with_flags;
use crate::workspace::{get_cargo_workspace, get_pgo_directory};
use cargo_metadata::Message;
use colored::Colorize;
use std::fmt::Write;

#[derive(clap::Parser, Debug)]
#[clap(trailing_var_arg(true))]
pub struct PgoInstrumentArgs {
    /// Additional arguments that will be passed to `cargo build`.
    cargo_args: Vec<String>,
}

pub fn pgo_instrument(args: PgoInstrumentArgs) -> anyhow::Result<()> {
    let config = cargo::Config::default()?;
    let workspace = get_cargo_workspace(&config)?;
    let pgo_dir = get_pgo_directory(&workspace)?;

    let mut flags = std::env::var("RUSTFLAGS").unwrap_or_default();
    write!(&mut flags, " -Cprofile-generate={}", pgo_dir.display()).unwrap();

    log::info!("PGO profiles will be stored into {}", pgo_dir.display());

    let flags = format!("-Cprofile-generate={}", pgo_dir.display());
    let output = build_with_flags(flags, args.cargo_args)?;

    for message in Message::parse_stream(output.stdout.as_slice()) {
        let message = message?;
        match message {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    log::info!(
                        "PGO-instrumented binary {} built successfully. Now run {} on your workload.",
                        artifact.target.name.blue(),
                        cli_format_path(&executable)
                    );
                }
            }
            Message::BuildFinished(res) => {
                if res.success {
                    println!(
                        "{}",
                        "PGO instrumentation build successfully finished".green()
                    );
                } else {
                    println!("{}", "PGO instrumentation build has failed".red());
                }
            }
            Message::TextLine(line) => println!("{}", line),
            Message::CompilerMessage(message) => {
                print!(
                    "{}",
                    message.message.rendered.unwrap_or(message.message.message)
                );
            }
            _ => {}
        }
    }

    Ok(())
}
