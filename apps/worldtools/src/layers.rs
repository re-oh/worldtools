use worldtools_simulation::WorldDataLayer;
use worldtools_ui::WorldLayer;

pub const fn simulation_layer(layer: WorldLayer) -> WorldDataLayer {
    match layer {
        WorldLayer::Elevation => WorldDataLayer::Elevation,
        WorldLayer::Tectonics => WorldDataLayer::Tectonics,
        WorldLayer::Hydrology => WorldDataLayer::Hydrology,
        WorldLayer::Climate => WorldDataLayer::Climate,
        WorldLayer::Soil => WorldDataLayer::Soil,
        WorldLayer::Vegetation => WorldDataLayer::Vegetation,
        WorldLayer::Geology => WorldDataLayer::Geology,
        WorldLayer::Resources => WorldDataLayer::Resources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_and_simulation_layers_keep_the_same_stable_order() {
        for (ui, simulation) in WorldLayer::ALL.into_iter().zip(WorldDataLayer::ALL) {
            assert_eq!(simulation_layer(ui), simulation);
        }
    }
}
