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

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    tech::tank::Tank,
    aqs_utils::mesh_of_squares::MeshOfSquares,
    decoration::types::DecorationTag,
};


pub fn ground(
    tank_cfg: Res<Tank>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let sgrid_size = UVec2::new( tank_cfg.get_size().x as u32 - 1, tank_cfg.get_size().z as u32 - 1);
    let sscale = Vec3::new( tank_cfg.get_size().x / sgrid_size.x as f32,
                            1.0,
                            tank_cfg.get_size().z / sgrid_size.y as f32 );
    dbg!(tank_cfg.get_size());
    let sgrid_scale = Vec2::splat( 1.0 );
    let sgrid_uv_scale = Vec2::new(1. / sgrid_size.x as f32, 1. / sgrid_size.y as f32);
    // // let sgrid_uv_scale = Vec2::splat(1.0);
    let ground_mesh = MeshOfSquares::new(sgrid_size + 1, sgrid_scale, sgrid_uv_scale)
        .randomize_position((-0.2, 0.5))  // roughness of surface
        .randomize_normals(0.002)         // bumpiness via normals
        .into_mesh();
    let gmesh_hdl = meshes.add(ground_mesh.clone());


    let mt_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.3, 0.2, 0.0, 1.0),
        // color_texture: Some(asset_server.load("textures/wgenerated.png")),
        alpha_mode: AlphaMode::Opaque,
        reflectance: 1.0,
        metallic: 0.4,
        double_sided: true,
        // height: 0.0,
        ..default()
    });

    let collider = Collider::from_bevy_mesh( &ground_mesh, &ComputedColliderShape::TriMesh(TriMeshFlags::all()) ).unwrap();
    let _ground_surface = commands
        .spawn((
            Mesh3d(gmesh_hdl),
            MeshMaterial3d(mt_hdl),
            Transform::from_translation(Vec3::Y * 2.0)
                .with_scale(sscale),
            Visibility::default(),
        ))
        .insert( collider )
        .insert( RigidBody::Fixed )
        .insert( DecorationTag )
        .id();
}
