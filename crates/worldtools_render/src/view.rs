use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    math::DVec2,
    prelude::{ButtonInput, MouseButton, Query, Res, ResMut, Resource, Vec2, Window, With},
    window::PrimaryWindow,
};

const MIN_VERTICAL_SPAN: f32 = 1.0 / 32_768.0;

#[derive(Clone, Copy, Debug, Resource)]
pub struct MapView {
    pub center: DVec2,
    pub vertical_span: f32,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            center: DVec2::splat(0.5),
            vertical_span: 1.0,
        }
    }
}

impl MapView {
    #[must_use]
    pub fn horizontal_span(self, aspect: f32) -> f32 {
        self.vertical_span * aspect * 0.5
    }

    pub fn pan_pixels(&mut self, delta: Vec2, viewport_size: Vec2) {
        if viewport_size.min_element() <= 0.0 {
            return;
        }
        let aspect = viewport_size.x / viewport_size.y;
        self.center.x -= f64::from(delta.x / viewport_size.x * self.horizontal_span(aspect));
        self.center.y = (self.center.y - f64::from(delta.y / viewport_size.y * self.vertical_span))
            .clamp(
                f64::from(self.vertical_span * 0.5),
                f64::from(1.0 - self.vertical_span * 0.5),
            );
    }

    pub fn zoom_at(&mut self, wheel_steps: f32, pointer: Vec2, viewport_size: Vec2) {
        if wheel_steps == 0.0 || viewport_size.min_element() <= 0.0 {
            return;
        }
        let old_vertical = self.vertical_span;
        let new_vertical =
            (old_vertical * (-wheel_steps * 0.14).exp()).clamp(MIN_VERTICAL_SPAN, 1.0);
        let aspect = viewport_size.x / viewport_size.y;
        let old_horizontal = old_vertical * aspect * 0.5;
        let new_horizontal = new_vertical * aspect * 0.5;
        let local = pointer / viewport_size - Vec2::splat(0.5);

        self.center.x += f64::from(local.x * (old_horizontal - new_horizontal));
        self.center.y += f64::from(local.y * (old_vertical - new_vertical));
        self.vertical_span = new_vertical;
        self.center.y = self.center.y.clamp(
            f64::from(new_vertical * 0.5),
            f64::from(1.0 - new_vertical * 0.5),
        );
    }
}

/// Configures pointer gestures without coupling the renderer to editor tools.
///
/// Middle-button panning is always available. The application should disable
/// primary-button panning while a tool owns primary-button drags.
#[derive(Clone, Copy, Debug, Resource)]
pub struct MapNavigationSettings {
    pub primary_pan: bool,
}

impl Default for MapNavigationSettings {
    fn default() -> Self {
        Self { primary_pan: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanButton {
    Primary,
    Middle,
}

impl PanButton {
    const fn mouse_button(self) -> MouseButton {
        match self {
            Self::Primary => MouseButton::Left,
            Self::Middle => MouseButton::Middle,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub(crate) struct MapNavigationState {
    active_pan: Option<PanButton>,
}

impl MapNavigationState {
    fn update(
        &mut self,
        buttons: &ButtonInput<MouseButton>,
        primary_pan: bool,
        can_start: bool,
    ) -> bool {
        if self
            .active_pan
            .is_some_and(|button| !buttons.pressed(button.mouse_button()))
        {
            self.active_pan = None;
        }

        if self.active_pan.is_none() && can_start {
            self.active_pan = if buttons.just_pressed(MouseButton::Middle) {
                Some(PanButton::Middle)
            } else if primary_pan && buttons.just_pressed(MouseButton::Left) {
                Some(PanButton::Primary)
            } else {
                None
            };
        }

        self.active_pan.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct MapViewport {
    pub min: Vec2,
    pub max: Vec2,
    pub input_blocked: bool,
    pub pixels_per_point: f32,
}

impl MapViewport {
    #[must_use]
    pub fn size(self, fallback: Vec2) -> Vec2 {
        let measured = self.max - self.min;
        if measured.min_element() > 1.0 {
            measured
        } else {
            fallback
        }
    }

    #[must_use]
    pub fn physical_size(self, fallback: Vec2) -> Vec2 {
        let scale = if self.pixels_per_point.is_finite() && self.pixels_per_point > 0.0 {
            self.pixels_per_point
        } else {
            1.0
        };
        self.size(fallback) * scale
    }

    #[must_use]
    pub fn local_pointer(self, pointer: Vec2, fallback: Vec2) -> Option<(Vec2, Vec2)> {
        let size = self.size(fallback);
        let min = if self.max.x > self.min.x {
            self.min
        } else {
            Vec2::ZERO
        };
        let local = pointer - min;
        (local.cmpge(Vec2::ZERO).all() && local.cmple(size).all()).then_some((local, size))
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
#[allow(clippy::too_many_arguments)] // Input resources remain independently observable by Bevy.
pub(crate) fn navigate(
    mut view: ResMut<MapView>,
    viewport: Res<MapViewport>,
    settings: Res<MapNavigationSettings>,
    mut state: ResMut<MapNavigationState>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let fallback = Vec2::new(window.width(), window.height());
    let pointer_in_viewport = window
        .cursor_position()
        .and_then(|pointer| viewport.local_pointer(pointer, fallback));
    let can_start = !viewport.input_blocked && pointer_in_viewport.is_some();

    if state.update(&buttons, settings.primary_pan, can_start) {
        view.pan_pixels(motion.delta, viewport.size(fallback));
    }
    let Some((local_pointer, viewport_size)) = pointer_in_viewport else {
        return;
    };
    if viewport.input_blocked {
        return;
    }
    let scroll_scale = match scroll.unit {
        MouseScrollUnit::Line => 1.0,
        MouseScrollUnit::Pixel => 0.02,
    };
    view.zoom_at(scroll.delta.y * scroll_scale, local_pointer, viewport_size);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longitude_stays_unwrapped_during_pan() {
        let mut view = MapView::default();
        view.pan_pixels(Vec2::new(2_000.0, 0.0), Vec2::new(1_000.0, 500.0));
        assert!(view.center.x < 0.0);
    }

    #[test]
    fn vertical_pan_uses_grab_direction() {
        let mut view = MapView {
            vertical_span: 0.5,
            ..MapView::default()
        };
        view.pan_pixels(Vec2::new(0.0, 50.0), Vec2::new(1_000.0, 500.0));
        assert!((view.center.y - 0.45).abs() < 1.0e-6);
    }

    #[test]
    fn viewport_allows_input_by_default() {
        assert!(!MapViewport::default().input_blocked);
    }

    #[test]
    fn zoom_keeps_pointer_anchor() {
        let mut view = MapView::default();
        let size = Vec2::new(1_000.0, 500.0);
        let pointer = Vec2::new(750.0, 250.0);
        let before = view.center.x + f64::from(0.25 * view.horizontal_span(2.0));
        view.zoom_at(2.0, pointer, size);
        let after = view.center.x + f64::from(0.25 * view.horizontal_span(2.0));
        assert!((before - after).abs() < 1.0e-5);
    }

    #[test]
    fn metre_scale_pan_survives_many_world_wraps() {
        let mut view = MapView {
            center: DVec2::new(128.5, 0.5),
            vertical_span: MIN_VERTICAL_SPAN,
        };
        let before = view.center.x;

        view.pan_pixels(Vec2::X, Vec2::new(1_000.0, 500.0));

        assert!((view.center.x - before).abs() > f64::EPSILON);
        assert!((view.center.x - before).abs() < 1.0e-6);
    }

    #[test]
    fn physical_viewport_size_uses_display_scale() {
        let viewport = MapViewport {
            min: Vec2::new(10.0, 20.0),
            max: Vec2::new(610.0, 420.0),
            pixels_per_point: 2.0,
            ..MapViewport::default()
        };

        assert_eq!(
            viewport.physical_size(Vec2::new(1_280.0, 720.0)),
            Vec2::new(1_200.0, 800.0)
        );
    }

    #[test]
    fn zoom_stops_at_source_resolution_limit() {
        let mut view = MapView::default();

        view.zoom_at(10_000.0, Vec2::splat(500.0), Vec2::splat(1_000.0));

        assert!((view.vertical_span - MIN_VERTICAL_SPAN).abs() < f32::EPSILON);
    }

    #[test]
    fn primary_pan_is_controlled_by_settings() {
        let mut state = MapNavigationState::default();
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Left);

        assert!(!state.update(&buttons, false, true));
        assert!(state.update(&buttons, true, true));
    }

    #[test]
    fn middle_pan_is_always_available() {
        let mut state = MapNavigationState::default();
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Middle);

        assert!(state.update(&buttons, false, true));
    }

    #[test]
    fn active_pan_remains_captured_outside_viewport() {
        let mut state = MapNavigationState::default();
        let mut buttons = ButtonInput::default();
        buttons.press(MouseButton::Middle);
        assert!(state.update(&buttons, false, true));

        assert!(state.update(&buttons, false, false));
        buttons.release(MouseButton::Middle);
        assert!(!state.update(&buttons, false, false));
    }
}
