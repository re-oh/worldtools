use bevy::prelude::Resource;

use super::WorldLayer;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerVisualStyle {
    pub opacity: f32,
    pub relief: f32,
    pub shadows: f32,
    pub detail: f32,
    pub boundaries: f32,
    pub show_labels: bool,
}

impl LayerVisualStyle {
    #[must_use]
    pub const fn for_layer(layer: WorldLayer) -> Self {
        match layer {
            WorldLayer::Elevation => Self {
                opacity: 1.0,
                relief: 0.92,
                shadows: 0.64,
                detail: 0.78,
                boundaries: 0.0,
                show_labels: true,
            },
            WorldLayer::Tectonics => Self {
                opacity: 0.82,
                relief: 0.62,
                shadows: 0.42,
                detail: 0.38,
                boundaries: 0.88,
                show_labels: true,
            },
            WorldLayer::Hydrology => Self {
                opacity: 0.94,
                relief: 0.84,
                shadows: 0.58,
                detail: 0.74,
                boundaries: 0.0,
                show_labels: true,
            },
            WorldLayer::Climate => Self {
                opacity: 0.84,
                relief: 0.34,
                shadows: 0.22,
                detail: 0.24,
                boundaries: 0.0,
                show_labels: true,
            },
            WorldLayer::Soil | WorldLayer::Vegetation | WorldLayer::Geology => Self {
                opacity: 0.90,
                relief: 0.72,
                shadows: 0.46,
                detail: 0.62,
                boundaries: 0.76,
                show_labels: true,
            },
            WorldLayer::Resources => Self {
                opacity: 0.96,
                relief: 0.74,
                shadows: 0.50,
                detail: 0.55,
                boundaries: 0.0,
                show_labels: true,
            },
        }
    }
}

impl Default for LayerVisualStyle {
    fn default() -> Self {
        Self::for_layer(WorldLayer::Elevation)
    }
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct MapPresentationSettings {
    styles: [LayerVisualStyle; WorldLayer::COUNT],
    pub legend_visible: bool,
    pub sun_azimuth_degrees: f32,
    pub sun_elevation_degrees: f32,
    pub ambient_occlusion: f32,
}

impl MapPresentationSettings {
    #[must_use]
    pub const fn style(&self, layer: WorldLayer) -> &LayerVisualStyle {
        &self.styles[layer.index()]
    }

    pub fn style_mut(&mut self, layer: WorldLayer) -> &mut LayerVisualStyle {
        &mut self.styles[layer.index()]
    }

    pub fn reset_layer(&mut self, layer: WorldLayer) {
        self.styles[layer.index()] = LayerVisualStyle::for_layer(layer);
    }
}

impl Default for MapPresentationSettings {
    fn default() -> Self {
        let mut styles = [LayerVisualStyle::default(); WorldLayer::COUNT];
        for layer in WorldLayer::ALL {
            styles[layer.index()] = LayerVisualStyle::for_layer(layer);
        }
        Self {
            styles,
            legend_visible: true,
            sun_azimuth_degrees: 315.0,
            sun_elevation_degrees: 38.0,
            ambient_occlusion: 0.48,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorical_layers_start_with_boundaries() {
        let presentation = MapPresentationSettings::default();
        assert!(presentation.style(WorldLayer::Soil).boundaries > 0.0);
        assert!(presentation.style(WorldLayer::Vegetation).boundaries > 0.0);
        assert!(presentation.style(WorldLayer::Hydrology).boundaries.abs() < f32::EPSILON);
    }

    #[test]
    fn reset_only_changes_the_selected_layer() {
        let mut presentation = MapPresentationSettings::default();
        presentation.style_mut(WorldLayer::Soil).opacity = 0.1;
        presentation.style_mut(WorldLayer::Climate).opacity = 0.2;
        presentation.reset_layer(WorldLayer::Soil);
        assert!((presentation.style(WorldLayer::Soil).opacity - 0.9).abs() < f32::EPSILON);
        assert!((presentation.style(WorldLayer::Climate).opacity - 0.2).abs() < f32::EPSILON);
    }
}
