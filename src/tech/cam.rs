/*
    Copyright 2023 github.com/programaquatic

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::tech::tank::Tank;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CameraElement {
    PanningPoint = 0x0,
    OrbitHandle = 0x1,
    Camera = 0x2,
}

// Component to identify (query) the AquaSim Cameraholder
#[derive(Component)]
pub struct AquaSimCamElement(CameraElement);

// Camera Scroll Factor
const CSFACTOR: f32 = 0.5;
const CCLOSEST: f32 = 2.0;

pub struct AquaSimCamPlugin;

impl Plugin for AquaSimCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize)
            .add_systems(Update, move_cam);
    }
}

fn initialize(
    tank_cfg: Res<Tank>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let initial_cam = Vec3::new(0.0, 50.0, 200.0).normalize() * 80.0;

    // cam_center is the transparent parent of the camera to simplify the cam-panning
    // all panning happens within this parent
    let cam_center_parent = commands
        .spawn((
            Name::new("CameraCenter"),
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().ico(5).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::linear_rgba(0.7, 0.8, 0.7, 1.0),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            })),
            Transform::from_translation(tank_cfg.get_center()),
        ))
        // TransformBundle::from_transform(
        //     Transform::from_translation(tank_cfg.get_center())
        // ))
        .insert(AquaSimCamElement(CameraElement::PanningPoint))
        .id();

    // cam-holder is the transparent parent of the camera to simplify the cam-orbiting
    // all rotation and panning happens within this parent
    let cam_holder = commands
        .spawn((
            Name::new("CameraHolder"),
            Transform::from_rotation(Quat::from_rotation_x(0.0)),
        ))
        .insert(AquaSimCamElement(CameraElement::OrbitHandle))
        .id();

    // the cam is the child getting pulled/rotated along with it
    // only the distance from the center of the parent is being associated with the cam itself
    let cam = commands
        .spawn((
            Name::new("Camera"),
            Camera3d::default(),
            Transform::from_translation(initial_cam).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(AquaSimCamElement(CameraElement::Camera))
        .id();

    commands.entity(cam_center_parent).add_child(cam_holder);
    commands.entity(cam_holder).add_child(cam);
}

fn move_cam(
    windows: Query<&Window>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut camera_elements: Query<(&mut Transform, &AquaSimCamElement)>,
) {
    let window = windows.single().unwrap();
    let orbit = MouseButton::Right;
    let pan = MouseButton::Middle;

    let mut move_pan = Vec2::ZERO;
    let mut move_orbit = Vec2::ZERO;
    let mut scroll = 0.0;

    for ev in ev_scroll.read() {
        scroll += -ev.y;
    }
    // checking which mode we're in
    if input_mouse.pressed(orbit) {
        for ev in ev_motion.read() {
            move_orbit += ev.delta;
        }
    } else if input_mouse.pressed(pan) {
        for ev in ev_motion.read() {
            move_pan += ev.delta;
        }
    } else {
        for _ in ev_motion.read() {}
        if scroll == 0.0 {
            return;
        }
    }

    for (mut transform, element) in camera_elements.iter_mut() {
        match element.0 {
            // TODO: panning needs to be orthogonal to the current angle of the orbit
            CameraElement::PanningPoint => {
                if move_pan.length_squared() > 0.0 {
                    let right = Vec3::X * -move_pan.x * 0.25;
                    let up = Vec3::Y * move_pan.y * 0.25;
                    transform.translation += (right + up) * (CSFACTOR / 5.0);
                }
            }
            CameraElement::OrbitHandle => {
                if move_orbit.length_squared() > 0.0 {
                    let window = get_primary_window_size(window);
                    let delta_x = move_orbit.x / window.x * std::f32::consts::PI * 2.0;
                    let delta_y = move_orbit.y / window.y * std::f32::consts::PI;

                    // rotational axis Y (horizontal rotation)
                    let decl = Quat::from_rotation_y(-delta_x);

                    // rotational axis from orthogonal vector in XZ plane
                    let asct = Quat::from_rotation_x(-delta_y);
                    transform.rotation = decl * transform.rotation * asct;
                }
            }
            CameraElement::Camera => {
                if scroll.abs() > 0.0 {
                    scroll *= CSFACTOR;
                    transform.translation = (transform.translation
                        + (transform.translation.normalize() * scroll))
                        .clamp_length(CCLOSEST, 1000.0);
                }
            }
        };
    }
}

fn get_primary_window_size(windows: &Window) -> Vec2 {
    let window = windows;
    Vec2::new(window.width(), window.height())
}
