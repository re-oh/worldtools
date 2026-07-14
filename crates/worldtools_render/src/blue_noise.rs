use bevy::{
    asset::RenderAssetUsages,
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(crate) const BLUE_NOISE_SIZE: u32 = 64;

pub(crate) fn image() -> Image {
    let pattern = &shade::dither::BNVC64x64;
    debug_assert_eq!(u32::from(pattern.width), BLUE_NOISE_SIZE);
    debug_assert_eq!(u32::from(pattern.height), BLUE_NOISE_SIZE);

    Image::new(
        Extent3d {
            width: BLUE_NOISE_SIZE,
            height: BLUE_NOISE_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pattern.data.to_vec(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    )
}
