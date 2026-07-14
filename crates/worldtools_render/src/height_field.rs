use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    prelude::{Image, Message},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use thiserror::Error;

/// A global endpoint-sampled raster.
///
/// Rows run north to south. Columns include both longitudinal seam endpoints,
/// so the final column represents the same meridian as the first. Rendering
/// uses `width - 1` as the horizontal period and keeps both latitude endpoints.
#[derive(Clone, Debug)]
pub struct HeightField {
    width: u32,
    height: u32,
    samples: Vec<f32>,
    range_m: [f32; 2],
}

impl HeightField {
    /// Builds an endpoint-sampled height field.
    ///
    /// # Errors
    /// Returns [`HeightFieldError`] when the dimensions are too small, the
    /// sample count does not match, or a sample is not finite.
    pub fn new(width: u32, height: u32, samples: Vec<f32>) -> Result<Self, HeightFieldError> {
        let expected = width as usize * height as usize;
        if width < 2 || height < 2 {
            return Err(HeightFieldError::TooSmall { width, height });
        }
        if samples.len() != expected {
            return Err(HeightFieldError::Length {
                expected,
                actual: samples.len(),
            });
        }
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(HeightFieldError::NonFinite);
        }

        let range_m = samples.iter().fold(
            [f32::INFINITY, f32::NEG_INFINITY],
            |[minimum, maximum], &sample| [minimum.min(sample), maximum.max(sample)],
        );
        Ok(Self {
            width,
            height,
            samples,
            range_m,
        })
    }

    /// # Panics
    /// Panics when either dimension is smaller than two.
    #[must_use]
    pub fn flat(width: u32, height: u32, elevation_m: f32) -> Self {
        Self::new(
            width,
            height,
            vec![elevation_m; width as usize * height as usize],
        )
        .expect("flat height fields use valid dimensions")
    }

    #[must_use]
    pub const fn size(&self) -> [u32; 2] {
        [self.width, self.height]
    }

    #[must_use]
    pub const fn range_m(&self) -> [f32; 2] {
        self.range_m
    }

    pub(crate) fn to_image(&self) -> Image {
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bytemuck::cast_slice(&self.samples).to_vec(),
            TextureFormat::R32Float,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    }
}

#[derive(Clone, Debug, Message)]
pub struct HeightFieldUpload(pub Arc<HeightField>);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HeightFieldError {
    #[error("height field must be at least 2 by 2, got {width} by {height}")]
    TooSmall { width: u32, height: u32 },
    #[error("height field expected {expected} samples, got {actual}")]
    Length { expected: usize, actual: usize },
    #[error("height field contains a non-finite sample")]
    NonFinite,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_dimensions_and_sample_count() {
        assert!(matches!(
            HeightField::new(1, 2, vec![0.0, 0.0]),
            Err(HeightFieldError::TooSmall { .. })
        ));
        assert!(matches!(
            HeightField::new(2, 2, vec![0.0; 3]),
            Err(HeightFieldError::Length { .. })
        ));
    }

    #[test]
    fn records_elevation_range() {
        let field = HeightField::new(2, 2, vec![-200.0, 50.0, 800.0, 10.0]).unwrap();
        let [minimum, maximum] = field.range_m();
        assert!((minimum + 200.0).abs() < f32::EPSILON);
        assert!((maximum - 800.0).abs() < f32::EPSILON);
    }

    #[test]
    fn retains_endpoint_sample_dimensions() {
        let field = HeightField::new(5, 3, vec![0.0; 15]).unwrap();
        assert_eq!(field.size(), [5, 3]);
    }
}
