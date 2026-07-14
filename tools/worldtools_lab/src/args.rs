use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use worldtools_world::CubeFace;

#[derive(Debug, Parser)]
#[command(
    name = "worldtools-lab",
    about = "Headless WorldTools generator diagnostics"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Render an equirectangular overview from the continuous world source.
    Overview(OverviewArgs),
    /// Render and audit one cube-sphere terrain tile.
    Tile(TileArgs),
    /// Check seams, LOD consistency, and terrain statistics at one level.
    Verify(VerifyArgs),
    /// Compare exhaustive six-face terrain aggregates across a seed range.
    Sweep(SweepArgs),
}

#[derive(Debug, Args)]
pub struct OverviewArgs {
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    #[arg(long, default_value_t = 1024)]
    pub width: u32,
    #[arg(long, default_value_t = 512)]
    pub height: u32,
    #[arg(long, default_value = "worldtools-overview.png")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct TileArgs {
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    #[arg(long, value_enum, default_value_t = FaceArg::PositiveX)]
    pub face: FaceArg,
    #[arg(long, default_value_t = 4)]
    pub level: u8,
    #[arg(long, default_value_t = 0)]
    pub x: u32,
    #[arg(long, default_value_t = 0)]
    pub y: u32,
    #[arg(long, default_value = "worldtools-tile.png")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    #[arg(long, default_value_t = 1)]
    pub seed: u64,
    #[arg(long, default_value_t = 2)]
    pub level: u8,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SweepArgs {
    #[arg(long, default_value_t = 1)]
    pub first_seed: u64,
    #[arg(long, default_value_t = 8)]
    pub count: usize,
    /// Exhaustive cube-sphere level sampled for every seed (0..=2).
    #[arg(long, default_value_t = 0)]
    pub level: u8,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum FaceArg {
    #[default]
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl From<FaceArg> for CubeFace {
    fn from(value: FaceArg) -> Self {
        match value {
            FaceArg::PositiveX => Self::PositiveX,
            FaceArg::NegativeX => Self::NegativeX,
            FaceArg::PositiveY => Self::PositiveY,
            FaceArg::NegativeY => Self::NegativeY,
            FaceArg::PositiveZ => Self::PositiveZ,
            FaceArg::NegativeZ => Self::NegativeZ,
        }
    }
}
