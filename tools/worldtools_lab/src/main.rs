mod args;
mod continuity;
mod cube_edges;
mod preview;
mod report;
mod sweep;
mod tile_set;
mod verify;
mod world;

use anyhow::Result;
use args::{Cli, Command};
use clap::Parser;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    match Cli::parse().command {
        Command::Overview(arguments) => preview::overview(&arguments),
        Command::Tile(arguments) => preview::tile(&arguments),
        Command::Verify(arguments) => verify::verify(&arguments),
        Command::Sweep(arguments) => sweep::sweep(&arguments),
        Command::World(arguments) => world::world(&arguments),
    }
}
