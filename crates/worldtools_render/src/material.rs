use bevy::{
    app::App,
    asset::{AssetPath, embedded_asset, embedded_path},
    prelude::{Asset, Handle, Image, TypePath, Vec4},
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::Material2d,
};

pub(crate) fn register_shader(app: &mut App) {
    embedded_asset!(app, "worldtools_terrain.wgsl");
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum DisplayLayerKind {
    #[default]
    Elevation = 0,
    Categorical = 1,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct TerrainMaterialParams {
    pub view: Vec4,
    pub display: Vec4,
    pub style: Vec4,
}

impl Default for TerrainMaterialParams {
    fn default() -> Self {
        Self {
            view: Vec4::new(0.5, 0.5, 1.0, 1.0),
            display: Vec4::new(1280.0, 720.0, 0.0, 0.7),
            style: Vec4::new(-8_000.0, 8_000.0, 0.0, 0.0),
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub params: TerrainMaterialParams,
    #[texture(1, filterable = false)]
    pub elevation: Handle<Image>,
    #[texture(2, filterable = false)]
    pub blue_noise: Handle<Image>,
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("worldtools_terrain.wgsl"))
                .with_source("embedded"),
        )
    }
}
