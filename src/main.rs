#![forbid(unsafe_code)]

use clap::Parser;

fn main() -> std::process::ExitCode {
    let cli = trk::cli::Cli::parse();
    let code = trk::run(cli);
    std::process::ExitCode::from(code.clamp(0, 255) as u8)
}
