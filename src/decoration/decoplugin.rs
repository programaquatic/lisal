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
    decoration::{
        types::DecorationTag,
        ground,
    },
};


pub struct DecorationPlugin;

impl Plugin for DecorationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, ground::ground)
            .add_systems(PreStartup, initialize)
            .add_systems(PreStartup, remove_colliders);
    }
}


// Temporary function to place some 'obstacles' into the tank for debugging flow
fn initialize(
    tank_cfg: Res<Tank>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let decoration_material_hdl = materials.add(StandardMaterial {
        base_color: Color::rgba(0.7, 0.8, 0.7, 1.0),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    // experimental 'rock'-sphere to see how the flow goes around the obstacle
    let rock_mesh = Mesh::from(shape::UVSphere {
        radius: 15. * tank_cfg.scale,
        sectors: 32,
        stacks: 16,
    });
    let collider = Collider::from_bevy_mesh( &rock_mesh, &ComputedColliderShape::TriMesh ).unwrap();
    let rock = commands.spawn(PbrBundle {
        mesh: meshes.add(rock_mesh),
        material: decoration_material_hdl,
        transform: Transform::from_translation( Vec3::new( 80., 0.0, 35. ) * tank_cfg.scale ),
        ..default()
    })
        .insert( collider )
        .insert( RigidBody::Fixed )
        .insert( DecorationTag )
        .id();
    commands.entity(tank_cfg.get_tank_parent()).push_children( &[ rock ]);
}





// get rid of decoration colliders because they're only needed during initialization for fluid grid cells to become solid
fn remove_colliders(
    mut commands: Commands,
    colliders: Query<(Entity, &Collider), With<DecorationTag>>,
) {
    colliders.for_each( | (item, _) | {
        commands.entity( item ).remove::<Collider>();
    })
}


