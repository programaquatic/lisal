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
    prelude::*,
    pbr::CascadeShadowConfigBuilder,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};

use bevy_rapier3d::{
    plugin::*,
    render::RapierDebugRenderPlugin,
};
// use bevy_inspector_egui::WorldInspectorPlugin;

use std::f32::consts::PI;

mod aqs_utils;
mod tech;
mod decoration;
mod water;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // create a textured background so that any potential reflections and/or water surface vectors become more visible
    let text_hdl = Some(asset_server.load("textures/flower_background.png"));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500., 500.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgba(0.8, 0.7, 0.1, 1.0),
            base_color_texture: text_hdl.clone(),
            // emissive: (),
            emissive_texture: text_hdl,
            // perceptual_roughness: (),
            // metallic: (),
            // metallic_roughness_texture: (),
            // reflectance: (),
            // normal_map_texture: (),
            // flip_normal_map_y: (),
            // occlusion_texture: (),
            // double_sided: (),
            // cull_mode: (),
            // unlit: (),
            // alpha_mode: (),
            // depth_bias: ()
            ..default()
        })),
        Transform::from_xyz(0.0, 50., -50.)
            .with_rotation( Quat::from_rotation_x( std::f32::consts::PI / 2.) ),
    ));


    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(60., 150.0, 20.0),
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
        ));
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins) //.set(CorePlugin { task_pool_options: TaskPoolOptions::with_num_threads(8), }))
        .add_systems(Startup, setup)

        // Diagnostics and Inspectors
        // .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::new(5))
        // .add_plugins(WorldInspectorPlugin::new())

        // old Rapier/Physics experiments
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        //.insert_resource(RapierConfiguration { gravity: Vec3::ZERO, ..default() })
        // .add_plugins(RapierDebugRenderPlugin::default())

        .add_plugins(tech::tank::TankPlugin)
        .add_plugins(tech::cam::AquaSimCamPlugin)
        .add_plugins(decoration::decoplugin::DecorationPlugin)
        .add_plugins(water::fluid::FluidPlugin)

        .run();
}
