mod artifact;
mod case;
mod check;
mod cli;
mod debug_script;
mod doctor;
mod process;
mod reproduce;
mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let workspace = workspace::root();

    match Cli::parse().command {
        Command::Doctor(arguments) => doctor::run(&workspace, &arguments),
        Command::Repro(arguments) => reproduce::run(&workspace, &arguments, false),
        Command::Capture(arguments) => reproduce::run(&workspace, &arguments, true),
        Command::Check(arguments) => check::run(&workspace, &arguments),
        Command::DebugScript(arguments) => debug_script::run(&workspace, &arguments),
    }
}
