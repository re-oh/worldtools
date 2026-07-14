use bevy::{
    app::App,
    asset::{AssetPath, embedded_asset, embedded_path},
    prelude::{Asset, Handle, Image, TypePath, Vec4},
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::Material2d,
};

pub(crate) fn register_shader(app: &mut App) {
    embedded_asset!(app, "worldtools_tile.wgsl");
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TerrainTileMaterialParams {
    /// Source sample origin and extent, measured in texels.
    pub sample_rect: Vec4,
    /// Metres per source sample in X/Y, dither amplitude, reserved.
    pub metrics: Vec4,
    /// Bit flags, border width, desired LOD, source LOD.
    pub debug: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainTileMaterial {
    #[uniform(0)]
    pub params: TerrainTileMaterialParams,
    #[texture(1, filterable = false)]
    pub elevation: Handle<Image>,
    #[texture(2, filterable = false)]
    pub blue_noise: Handle<Image>,
}

impl Material2d for TerrainTileMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("worldtools_tile.wgsl"))
                .with_source("embedded"),
        )
    }
}
