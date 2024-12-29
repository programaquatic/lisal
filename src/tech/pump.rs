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
    math:: {
        Vec3A,
        prelude::Sphere
    }
};


use crate::{
    aqs_utils::extforcevol::ExternalForceVolume,
    tech::tank::Tank,
};

const EFFECTIVE_RADIUS: f32 = 1.0;

#[derive(Component, Default)]
pub struct Pump {
    /// the center source position (from where the particles get pulled)
    source: Vec3A,
    /// the center of the target position
    target: Vec3A,
    /// the velocity+direction of particles at the target
    target_velocity: Vec3A,
    // /// the radius of the source and target locations
    // radius: f32,
}

impl Pump {
    #[allow(dead_code)]
    pub fn new(source: Vec3,
               target: Vec3,
               target_velocity: Vec3,
               // radius: f32,
    ) -> Self {
        Pump {
            source: Vec3A::from(source),
            target: Vec3A::from(target),
            target_velocity: Vec3A::from(target_velocity),
            // radius,
        }
    }
    pub fn from_extforcevolumes(src: &ExternalForceVolume, dst: &ExternalForceVolume) -> Self {
        Pump {
            source: Vec3A::from( src.location ),
            target: Vec3A::from( dst.location ),
            target_velocity: Vec3A::from( dst.get_force_for_position(dst.location) ),
            // radius: f32::min( src.extent.min_element(), dst.extent.min_element() ),  // using squared lengths
        }
    }

    pub fn particle_pump(&self, refpoint: Vec3A) -> Option::<(Vec3A, Vec3A)> {
        let (distance, relative) = self.relative_distance(refpoint);
        if  relative <= EFFECTIVE_RADIUS {
            Some( (self.target + distance, self.target_velocity) )
        } else {
            None
        }
    }

    fn relative_distance(&self, refpoint: Vec3A) -> (Vec3A, f32) {
        let refpoint_distance = refpoint - self.source;
        (refpoint_distance, refpoint_distance.length())
    }
}

pub fn initialize(
    tank_cfg: Res<Tank>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    dbg!("{}", tank_cfg.pump.outlet.clone());

    let pump_efv = Pump::from_extforcevolumes(
        &tank_cfg.pump.outlet,
        &tank_cfg.pump.inlet,
    );

    let water_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.5, 0.5, 0.5, 0.1),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let pump = commands
        .spawn(pump_efv)
        // .insert(PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::UVSphere {
        //         radius: 1.0,
        //         sectors: 8,
        //         stacks: 8,
        //     })),
        //     material: water_material.clone(),
        //     transform: Transform::from_translation( tank_cfg.pump.outlet.location )
        //         .with_scale( tank_cfg.pump.outlet.extent ),
        //     ..default()
        // })
        .id();

    let inlet = commands
        .spawn( tank_cfg.pump.inlet.clone() )
        .id();
    let outlet = commands
        .spawn( tank_cfg.pump.outlet.clone() )
        .insert((
            Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(8).unwrap())),
            MeshMaterial3d(water_material),
            Transform::from_translation( tank_cfg.pump.outlet.location )
                .with_scale( tank_cfg.pump.outlet.extent * EFFECTIVE_RADIUS ),
        ))
        .id();

    commands.entity(tank_cfg.get_tank_parent()).add_children(&[pump, inlet, outlet]);
    println!("pump outlet location: {}", tank_cfg.pump.outlet.location );
}
