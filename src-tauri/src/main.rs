// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    // If CLI args are provided (beyond the program name), run in CLI mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // CLI mode
        tracing_subscriber::fmt::init();
        let cli = matrix_fileshare_lib::cli::Cli::parse();
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        if let Err(e) = rt.block_on(matrix_fileshare_lib::cli::run_cli(cli)) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // GUI mode (Tauri)
        matrix_fileshare_lib::run();
    }
}
