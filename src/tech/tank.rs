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
use serde::{Serialize, Deserialize};
use std::fmt;
use bevy_rapier3d::prelude::*;

use crate::{
    aqs_utils::{
        constants::Constants,
        config,
        extforcevol::ExternalForceVolume,
    },
    tech::pump,
    decoration::types::DecorationTag,
};
// use crate::water::surface as sf;

#[derive(Serialize, Deserialize, Debug)]
enum RelPosition {
    Right,
    Left,
    Bottom,
    Back,
    Front,
}

#[derive(Serialize, Deserialize, Debug)]
struct HoleAndLocation {
    position: RelPosition,
    x: u32,
    y: u32,
    diameter: u32,
}

impl fmt::Display for HoleAndLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}( {}, {} )/{}", self.position, self.x, self.y, self.diameter)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
struct TankDimensions {
    width: f32,
    depth: f32,
    height: f32,
    glass: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct OverFlowData {
    drill: Vec<HoleAndLocation>,
    shaft: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PumpDefinition {
    pub inlet: ExternalForceVolume,
    pub outlet: ExternalForceVolume,
}

#[derive(Resource, Serialize, Deserialize, Debug)]
pub struct Tank {
    tank: TankDimensions,
    overflow: OverFlowData,
    #[serde(default)]
    pub scale: f32,
    #[serde(default)]
    tank_id: Option<Entity>,
    #[serde(default)]
    pub pump: PumpDefinition,
}


impl fmt::Display for Tank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size    : {} x {} x {}; {}", self.tank.width, self.tank.depth, self.tank.height, self.tank.glass)?;
        write!(f, "Overflow: shaft: {:?}", self.overflow.shaft)?;
        write!(f, "Overflow: drill: {:?}", self.overflow.drill)
    }
}

impl FromWorld for Tank {
    fn from_world( _world: &mut World ) -> Self {
        let mut tank_cfg: Tank = config::read_json::<Tank>(String::from("assets/tank.json")).unwrap();
        println!("{:?}", tank_cfg);
        let aqs_constants: Constants = config::read_json::<Constants>(String::from("assets/constants.json")).unwrap();

        tank_cfg.pump.outlet.name = Some("OUT".to_string());
        // adjust tank config for config parameters
        tank_cfg.update( aqs_constants.MAX_GRID_CELLS );

        // this is the meshless parent entity for the tank to allow for a global offset,
        // it's a SpatialBundle to assure Transform- and Visibility Propagation
        // which require Visibility, ComputedVisibility, Transform and GlobalTransform to be set up
        let ptank = _world.spawn(SpatialBundle {
            // transform: Transform::from_translation(-tank_cfg.get_center()),
            transform: Transform::from_translation(Vec3::ZERO),
            visibility: Visibility::default(),
            ..Default::default()
        })
            .insert(Name::new("Core-Tank-box"))
            .insert(ParentTankTag)
            .id();

        tank_cfg.tank_id = Some(ptank);
        tank_cfg
    }
}

impl Tank {
    pub fn get_size(&self) -> Vec3 {
        Vec3::new(self.tank.width,
                  self.tank.height,
                  self.tank.depth)
    }
    pub fn get_center(&self) -> Vec3 {
        self.get_size() / 2.0
    }

    // CALL ONLY WHEN SURE TANK HAS BEEN INITIALIZED
    pub fn get_tank_parent(&self) -> Entity {
        self.tank_id.unwrap()
    }

    pub fn get_pump_definition(&self) -> &PumpDefinition {
        &self.pump
    }

    pub fn update(&mut self, grid_cells: usize) -> f32 {
        let cell_count = self.tank.width * self.tank.depth * self.tank.height;
        let cell_scale_factor = f32::powf( grid_cells as f32 / cell_count, 1./3. );
        self.scale = cell_scale_factor;
        println!("Tank-to-Grid Scale: {}", cell_scale_factor );

        self.tank.width *= cell_scale_factor;
        self.tank.depth *= cell_scale_factor;
        self.tank.height *= cell_scale_factor;
        self.tank.glass *= cell_scale_factor;

        self.pump.inlet.scale( cell_scale_factor );
        self.pump.outlet.scale( cell_scale_factor );

        for s in self.overflow.shaft.iter_mut() {
            s.x *= cell_scale_factor;
            s.y *= cell_scale_factor;
        }
        println!("TANK_AFTER CONVERSION: {:?}", self);
        cell_scale_factor
    }

    #[allow(dead_code)]
    #[inline]
    pub fn to_world(&self, point: Vec3) -> Vec3 {
        point * self.scale
    }
}


pub struct TankPlugin;

impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Constants>()
            .init_resource::<Tank>()
            .add_systems( PreStartup, initialize)
            .add_systems( PreStartup, pump::initialize );
    }
}

// #[derive(Default)]
struct GlassPaneDefinition {
    name: Name,
    mesh_hdl: Handle<Mesh>,
    mat_hdl: Handle<StandardMaterial>,
    position: Vec3,
    scale: Vec3,
    rotation: Quat,
    is_decoration: bool,
}

impl Default for GlassPaneDefinition {
    fn default() -> Self {
        // let def_mesh = Mesh::from(shape::Box::new(1.,1.,1.));
        GlassPaneDefinition {
            name: Name::new(""),
            // mesh: def_mesh,
            mesh_hdl: Handle::<Mesh>::default(),
            mat_hdl: Handle::<StandardMaterial>::default(),
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::from_axis_angle( Vec3::Y, 0.0 ),
            is_decoration: false,
        }
    }
}

#[derive(Component)]
pub struct ParentTankTag;

fn initialize(
    mut commands: Commands,
    tank_cfg: ResMut<Tank>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /* create a surface from the tank-cfg and make the surface the defining resource
     */

    // let mut tank_srf = sf::Surface::default();

    let glass_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.9, 1.0, 0.9, 0.2),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let black_glass_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(1., 1., 1., 1.0),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    // create dimensions and center from tank configuration
    let dim: Vec3 = tank_cfg.get_size();
    let dim_center: Vec3 = tank_cfg.get_center();

    let glass_thick = tank_cfg.tank.glass / 10.0;

    // temp extra plane as artificial bottom (for now)
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d::default().mesh().size( 200.0, 200.0 ))),
        material: materials.add(Color::linear_rgb(0.3, 0.3, 0.3)),
        transform: Transform::from_xyz( 0.0, -dim_center[1]-glass_thick, 0.0),
        ..default()
    });

    // pre-define the glas panes as mesh and handles for re-use in Rapier colliders
    // pre-define side panes
    let side_pane_mesh = Mesh::from(Cuboid::from_corners(Vec3{x:glass_thick, y:dim[1] + glass_thick, z:dim[2]},
                                                         Vec3{x:0.0, y:0.0,  z:0.0}));
    let side_pane = meshes.add(side_pane_mesh);

    // pre-define front/back panes
    let front_pane_mesh = Mesh::from(Cuboid::from_corners(Vec3{ x: dim[0]+2.0*glass_thick,y: dim[1]+glass_thick, z: glass_thick},
                                                          Vec3{ x: 0.0, y: 0.0, z: 0.0 }));
    let front_pane = meshes.add(front_pane_mesh);

    // pre-define bottom pane
    let bottom_pane_mesh = Mesh::from(Cuboid::from_corners(Vec3{ x: dim[0], y: glass_thick, z: dim[2]},
                                                           Vec3{ x: 0.0, y: 0.0, z: 0.0 }));
    let bottom_pane = meshes.add(bottom_pane_mesh);

    let ptank = tank_cfg.get_tank_parent();

    // generate array of glass pane definitions/locations
    //  note: no .clone() if this is the 'last use' of the variable
    let mut glass_panes = vec![
        GlassPaneDefinition {
            name: Name::new("Right Side Glass"),
            position: Vec3::new( dim[0], -glass_thick, 0.0 ),
            mesh_hdl: side_pane.clone(),
            mat_hdl: glass_material_hdl.clone(),
            ..default()
        },
        GlassPaneDefinition {
            name: Name::new("Left Side Glass"),
            position: Vec3::new( -glass_thick, -glass_thick, 0.0 ),
            mesh_hdl: side_pane,
            mat_hdl: glass_material_hdl.clone(),
            ..default()
        },
        GlassPaneDefinition {
            name: Name::new("Back Side Glass"),
            position: Vec3::new( -glass_thick, -glass_thick, -glass_thick ),
            mesh_hdl: front_pane.clone(),
            mat_hdl: glass_material_hdl.clone(),
            ..default()
        },
        GlassPaneDefinition {
            name: Name::new("Front Side Glass"),
            position: Vec3::new( -glass_thick, -glass_thick, dim[2] ),
            mesh_hdl: front_pane,
            mat_hdl: glass_material_hdl.clone(),
            ..default()
        },
        GlassPaneDefinition {
            name: Name::new("Bottom Glass"),
            position: Vec3::new( 0.0, -glass_thick, 0.0 ),
            mesh_hdl: bottom_pane,
            mat_hdl: glass_material_hdl,
            ..default()
        },
    ];

    // build the list of shaft panes
    let path_len = tank_cfg.overflow.shaft.len();
    if path_len > 0 {
        // define a base-pane mesh where x is the length of 1.0 and y and z are defaults (height and glass thickness)
        // then when inserting, stretch the pane in x-direction to match the config
        let spane_base_mesh = Mesh::from(Cuboid::from_corners(
            Vec3 {
                x: 1.0,
                y: dim[1]-(9.0*tank_cfg.scale),
                z: glass_thick},
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0}
        ));
        let spane_base = meshes.add( spane_base_mesh );

        // zip the shaft-path definitions such that we get a tuple of (i+1, i), i.e. the endpoint and the current point
        for (i, (b, a)) in tank_cfg.overflow.shaft.iter().skip(1).zip( tank_cfg.overflow.shaft.iter().take( path_len ) ).enumerate() {
            let (xd, zd) = ( b.x - a.x, b.y - a.y );

            let plen = f32::sqrt( xd*xd + zd*zd );
            let rpos = Vec3::ZERO;

            let xangle = get_angle( xd, zd);
            println!("{},{}", plen, xangle);
            glass_panes.push( GlassPaneDefinition {
                name: Name::new(i.to_string() + "Shaft-Pane"),
                mesh_hdl: spane_base.clone(),
                mat_hdl: black_glass_material_hdl.clone(),
                position: Vec3::new( rpos.x+a.x, rpos.y, rpos.z+a.y ),
                scale: Vec3::from( (plen, 1.0, 1.0) ),
                rotation: Quat::from_axis_angle( Vec3::from ( (0.0, -1.0, 0.0) ), xangle),
                is_decoration: true,
            } );
        }
    }

    //////////////////////////////////////////////
    // TODO: drilled holes-feature at some point in the future
    //////////////////////////////////////////////

    // Insert the accumulated list of glass panes
    let mut panes_list = Vec::<Entity>::with_capacity(glass_panes.len());
    for glass in glass_panes.into_iter() {
        let glass_mesh = meshes.get( &glass.mesh_hdl ).unwrap();
        let collider = {
            let mut tc = Collider::from_bevy_mesh( glass_mesh, &ComputedColliderShape::TriMesh ).unwrap();
            // This scale+promote_shape is necessary because bevy_rapier appears to not correctly scale the
            // parts of the collider that are used when calculating intersections
            tc.set_scale( glass.scale, 2);
            tc.promote_scaled_shape();
            tc
        };

        let pane = commands
            .spawn(PbrBundle {
                mesh: glass.mesh_hdl.clone(),
                material: glass.mat_hdl.clone(),
                transform: Transform::from_translation( glass.position )
                    .with_scale( glass.scale )
                    .with_rotation( glass.rotation ),
                ..default()
            }).insert(glass.name.clone())
            .id();
        if glass.is_decoration {
            commands
                .entity(pane)
                .insert( RigidBody::Fixed )
                .insert( collider )
                .insert( ColliderScale::Absolute( Vec3::ONE ) )
                .insert( DecorationTag );
        }
        panes_list.push( pane );
        println!("Glass: {} -> id{}", glass.name, pane.index() );
    }
    commands.entity(ptank).push_children(&panes_list);
}

fn get_angle( xd: f32, yd: f32 ) -> f32 {
    yd.atan2(xd)
}



#[cfg(test)]
mod test
{
    use super::*;
    use crate::aqs_utils::extforcevol::ForceVolumeDirection;

    #[test]
    fn test_serial_out() {
        let tank = Tank {
            tank: TankDimensions {
                width: 160.,
                depth: 60.,
                height: 70.,
                glass: 15.,
            },
            overflow: OverFlowData {
                drill: vec![],
                shaft: vec![ Vec2::new( 40., 0.), Vec2::new( 40., 15.), Vec2::new( 0., 15.) ],
            },
            scale: 1.0,
            tank_id: None,
            pump: PumpDefinition {
                inlet: ExternalForceVolume::new( Vec3::new(10.,60.,25.),
                                                 Vec3::new(2.,2.,10.),
                                                 ForceVolumeDirection::from_parallel(
                                                     Vec3::new(20.,1.0,0.0)),
                                                 Some("IN".to_string())),
                outlet: ExternalForceVolume::new( Vec3::new(10.,10.,25.),
                                                  Vec3::new(2.,2.,10.),
                                                  ForceVolumeDirection::from_parallel(
                                                    Vec3::new(20.,1.0,0.0)),
                                                  Some("OUT".to_string())),
            },
        };
        let ostr = serde_json::to_string_pretty(&tank).unwrap();
        println!("{}",ostr);
    }
}
