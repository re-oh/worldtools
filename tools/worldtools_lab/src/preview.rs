use anyhow::{Context, Result, ensure};
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use worldtools_analysis::audit_terrain;
use worldtools_world::{
    GeoPoint, TILE_CELLS, TerrainGenerator, TerrainSettings, TileId, WorldSeed,
};

use crate::{
    args::{OverviewArgs, TileArgs},
    report::write_json,
};

pub fn overview(arguments: &OverviewArgs) -> Result<()> {
    ensure!(
        arguments.width >= 2 && arguments.height >= 2,
        "overview must be at least 2 by 2"
    );
    let generator = TerrainGenerator::new(WorldSeed(arguments.seed), TerrainSettings::default());
    let width = usize::try_from(arguments.width).context("overview width does not fit usize")?;
    let height = usize::try_from(arguments.height).context("overview height does not fit usize")?;
    let row_bytes = width
        .checked_mul(4)
        .context("overview row size overflowed")?;
    let pixel_bytes = row_bytes
        .checked_mul(height)
        .context("overview pixel buffer size overflowed")?;
    let latitudes = (0..arguments.height)
        .map(|y| 90.0 - 180.0 * f64::from(y) / f64::from(arguments.height - 1))
        .collect::<Vec<_>>();
    let longitudes = (0..arguments.width)
        .map(|x| -180.0 + 360.0 * f64::from(x) / f64::from(arguments.width))
        .collect::<Vec<_>>();
    let mut pixels = vec![0_u8; pixel_bytes];

    pixels
        .par_chunks_exact_mut(row_bytes)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, &longitude) in longitudes.iter().enumerate() {
                let elevation =
                    generator.sample_geo(GeoPoint::from_degrees(latitudes[y], longitude));
                row[x * 4..x * 4 + 4].copy_from_slice(&terrain_color(elevation));
            }
        });

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(arguments.width, arguments.height, pixels)
        .context("overview pixel buffer had an invalid size")?;
    image
        .save(&arguments.output)
        .with_context(|| format!("failed to write {}", arguments.output.display()))?;
    println!("wrote {}", arguments.output.display());
    Ok(())
}

pub fn tile(arguments: &TileArgs) -> Result<()> {
    let settings = TerrainSettings::default();
    let id = TileId::new(
        arguments.face.into(),
        arguments.level,
        arguments.x,
        arguments.y,
    )?;
    let tile = TerrainGenerator::new(WorldSeed(arguments.seed), settings).generate(id);
    let image_extent =
        u32::try_from(TILE_CELLS + 1).context("tile image extent does not fit u32")?;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(image_extent, image_extent);
    for y in 0..=TILE_CELLS {
        for x in 0..=TILE_CELLS {
            image.put_pixel(
                u32::try_from(x).context("tile pixel x does not fit u32")?,
                u32::try_from(y).context("tile pixel y does not fit u32")?,
                Rgba(terrain_color(tile.interior_sample(x, y))),
            );
        }
    }
    image
        .save(&arguments.output)
        .with_context(|| format!("failed to write {}", arguments.output.display()))?;
    write_json(&audit_terrain(&tile, settings.planet_radius_m), None)?;
    println!("wrote {}", arguments.output.display());
    Ok(())
}

fn terrain_color(height_m: f32) -> [u8; 4] {
    let rgb = if height_m < 0.0 {
        mix(
            [20.0, 93.0, 110.0],
            [6.0, 23.0, 46.0],
            (-height_m / 6500.0).clamp(0.0, 1.0),
        )
    } else if height_m < 450.0 {
        mix([69.0, 122.0, 71.0], [110.0, 132.0, 69.0], height_m / 450.0)
    } else if height_m < 2200.0 {
        mix(
            [110.0, 132.0, 69.0],
            [120.0, 89.0, 64.0],
            (height_m - 450.0) / 1750.0,
        )
    } else {
        mix(
            [120.0, 89.0, 64.0],
            [225.0, 230.0, 224.0],
            ((height_m - 2200.0) / 3000.0).clamp(0.0, 1.0),
        )
    };
    [channel(rgb[0]), channel(rgb[1]), channel(rgb[2]), 255]
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn channel(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn mix(first: [f32; 3], second: [f32; 3], amount: f32) -> [f32; 3] {
    let amount = amount.clamp(0.0, 1.0);
    std::array::from_fn(|index| first[index] + (second[index] - first[index]) * amount)
}
