// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slideflare_lib::cli;

fn main() {
    // Before parsing: clap prints help, version, and usage errors itself and
    // exits, and on Windows there is nowhere for any of that to go until a
    // console is attached.
    cli::attach_console();

    std::process::exit(cli::dispatch(cli::parse()));
}
