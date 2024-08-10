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
    math::prelude::Sphere,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
};


use bevy_rapier3d::{
    plugin::*,
    prelude::RapierConfiguration,
    // render::RapierDebugRenderPlugin,
};
// use bevy_inspector_egui::WorldInspectorPlugin;

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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::default().mesh().size(500., 500.))),
        material: materials.add(StandardMaterial {
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
        }),
        transform: Transform::from_xyz(0.0, 50., -50.)
            .with_rotation( Quat::from_rotation_x( std::f32::consts::PI / 2.) ),
        ..default()});

    // create a small sphere with a light source
    commands.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(1.0).mesh().ico(5).unwrap()),
        transform: Transform::from_xyz(30., 150., 0.0),
        material: materials.add(StandardMaterial {
            unlit: true,
            ..default()
        }),
        ..default()
    })
        .with_children(| children | {
            children.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity: 600000.,
                    radius: 20.,
                    range: 1000.,
                    ..default()
                },
                ..default()
            });
        });

    // // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 600000.,
    //         range: 1000.,
    //         ..default()
    //     },
    //     ..default()
    // });
}


fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins) //.set(CorePlugin { task_pool_options: TaskPoolOptions::with_num_threads(8), }))
        .add_systems(Startup, setup)

        // Diagnostics and Inspectors
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        // .add_plugins(WorldInspectorPlugin::new())

        // old Rapier/Physics experiments
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(RapierConfiguration { gravity: Vec3::ZERO, ..default() })
        // .add_plugin(RapierDebugRenderPlugin::default())

        .add_plugins(tech::tank::TankPlugin)
        .add_plugins(tech::cam::AquaSimCamPlugin)
        .add_plugins(decoration::decoplugin::DecorationPlugin)
        .add_plugins(water::fluid::FluidPlugin)

        .run();
}
