use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy_egui::EguiContexts;
use super::components::OrbitCamera;

pub fn orbit_camera(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let mut rotation_delta = Vec2::ZERO;
    let mut zoom_delta: f32 = 0.0;

    if mouse_button.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            rotation_delta += ev.delta;
        }
    } else {
        mouse_motion.clear();
    }

    for ev in scroll_events.read() {
        match ev.unit {
            MouseScrollUnit::Line => zoom_delta -= ev.y * 1.0,
            MouseScrollUnit::Pixel => zoom_delta -= ev.y * 0.05,
        }
    }

    for (mut orbit, mut transform) in &mut query {
        orbit.yaw -= rotation_delta.x * 0.005;
        orbit.pitch = (orbit.pitch + rotation_delta.y * 0.005).clamp(0.15, 1.4);
        orbit.radius = (orbit.radius + zoom_delta).clamp(5.0, 30.0);

        let cam_pos = orbit.focus + Vec3::new(
            orbit.radius * orbit.pitch.cos() * orbit.yaw.sin(),
            orbit.radius * orbit.pitch.sin(),
            orbit.radius * orbit.pitch.cos() * orbit.yaw.cos(),
        );
        *transform = Transform::from_translation(cam_pos).looking_at(orbit.focus, Vec3::Y);
    }
}
