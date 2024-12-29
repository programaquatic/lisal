// MIT License

// Copyright (c) 2022 github.com/robkau
// Copyright (c) 2023 github.com/programaquatic

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.


// Parts of this code are based on github.com/robkau who did this for 2D
//    see: https://github.com/robkau/mlsmpm-particles-rs

use bevy::{
    prelude::*,
    math::{UVec3, Mat3A, Vec3A},
};
use bevy_rapier3d::prelude::*;

use crate::{
    tech::tank::Tank,
    decoration::types::DecorationTag,
    aqs_utils::{
        constants::Constants,
        extforcevol::ExternalForceVolume,
        coneshape::ZCone,
    },
    water::resources::{
        FluidParticleVelocity,
        FluidQuantityMass,
    },
};

pub const DEBUG_GRID: bool = false;


#[derive( Component, Clone, PartialEq, Eq, Debug )]
pub enum GridCellType {
    Solid,
    Fluid,
    Air,
}

#[derive( Component, Clone, PartialEq, Eq, Debug )]
pub struct GridCellIndex(pub usize);


/// Stores the static accumulated external forces for a grid cell
#[derive( Component, Debug)]
pub struct GridCellAccumulatedForce(Vec3A);

/// Stores the normals from wher the cell touches a collider
#[derive( Component, Debug)]
pub struct ColliderNormals( Vec<Vec3A> );

/// Stores a vector or fluid neighbors in case of solid/wall cells
#[derive( Component, Debug)]
pub struct GridFluidNeighbors( Vec<usize> );

/** The definition of a grid with the total size (including boundaries)
    the cell scaling and the array of cell definitions
**/
#[derive(Resource)]
pub struct Grid {
    /// Number of grid cells per dimension
    grid_dim: UVec3,
    /// scaling factor to translate from unit-grid to graphics
    /// store as f32 because it's often used to translate Vec3 coordinates
    scale: f32,
    // /// the real-world coordinates of the center of the tank's grid
    // /// todo: this is technically the wrong place for this item
    // grid_center: Vec3,
    /// grid cell array
    cells: Vec< Entity >,

    /// scratchpads for velocity and mass to more efficiently iterate over grid cells
    tmp_velo: Vec< Vec3A >,
    tmp_mass: Vec< f32 >,

    /// current level of water surface
    _surface_level: f32,

    /// upper world boundary to simplify clamp-down
    wall_limit: Vec3A
}

impl Grid {
    pub fn new(space: UVec3, cell_scale: f32) -> Self {
        let tank_space = space;
        let cell_count_v = (tank_space.as_vec3() / cell_scale).as_uvec3();
        if (cell_count_v * cell_scale as u32) != tank_space {
            println!("WARNING: Grid and Tank Spec are not compatible!");
        }
        let grid_size = cell_count_v + UVec3{ x: 2, y: 4, z: 2 };
        let cell_count = grid_size.x * grid_size.y * grid_size.z;
        println!("INFO: GridSize ({},{},{}); cells={}, scale={}", grid_size.x, grid_size.y, grid_size.z, cell_count, cell_scale);
        Grid {
            grid_dim: grid_size,
            cells: Vec::with_capacity( cell_count as usize ),
            tmp_velo: vec![ Vec3A::ZERO; cell_count as usize ],
            tmp_mass: vec![ 0.0; cell_count as usize ],
            scale: cell_scale,
            // grid_center: (cell_count_v + UVec3::splat(2)).as_vec3() * cell_scale / 2.,
            _surface_level: 0.0,
            wall_limit: grid_size.as_vec3a(),
        }
    }

    pub fn index_of(&self, x: usize, y: usize, z: usize) -> usize {
        let index= (self.grid_dim.x as usize * self.grid_dim.y as usize* z)
            + (self.grid_dim.x as usize * y) + x;
        if index >= self.cell_count() {
            self.cell_count() - 1
        } else {
            index
        }
    }
    pub fn index_of_vec(&self, xyz: &UVec3) -> usize {
        let idx = self.index_of( xyz.x as usize, xyz.y as usize, xyz.z as usize );
        // since type is usize, we skip: idx >= 0 test
        if idx >= self.cell_count() {
            println!(" Cell out of range: {} -> {}/{}, {}", xyz, idx, self.cell_count()
                     , self.grid_dim);
            assert!( idx < self.cell_count() );
        }
        idx
    }

    // turn index into coordinates assuming 3D self represents the dimensions
    pub fn to_3d(&self, index: usize) -> UVec3 {
        let xi = index as u32 % self.grid_dim.x;
        let yi = (index as u32 / self.grid_dim.x) % self.grid_dim.y;
        let zi = (index as u32/ (self.grid_dim.x * self.grid_dim.y)) % self.grid_dim.z;
        UVec3::new( xi, yi, zi )
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
        // 1.0
    }

    pub fn to_world_coord(&self, input: Vec3) -> Vec3 {
        input * self.get_scale() // - self.grid_center
    }

    pub fn cell_count(&self) -> usize {
        (self.grid_dim.x * self.grid_dim.y * self.grid_dim.z) as usize
    }

    pub fn grid_size(&self) -> &UVec3 {
        &self.grid_dim
    }

    pub fn wall_vector(&self) -> &Vec3A {
        &self.wall_limit
    }

    pub fn get_tmp_velo(&self) -> &Vec<Vec3A> {
        &self.tmp_velo
    }
    pub fn get_tmp_mass(&self) -> &Vec<f32> {
        &self.tmp_mass
    }

    pub fn get_tmp_velo_mut(&mut self) -> &mut Vec<Vec3A> {
        &mut self.tmp_velo
    }
    pub fn get_tmp_mass_mut(&mut self) -> &mut Vec<f32> {
        &mut self.tmp_mass
    }


    pub fn reset_tmp_mass(&mut self) {
        self.tmp_mass.iter_mut().for_each(| item | *item = 0.0);
    }

    pub fn reset_tmp_velo(&mut self) {
        self.tmp_velo.iter_mut().for_each(| item | *item = Vec3A::ZERO);
    }

    pub fn get_surface_level(&self) -> f32 {
        // self.surface_level - 0.5
        self.grid_size().y as f32 - 5.
    }

    // actually prepare the grid cells with data
    pub fn initialize(&mut self, cells: Vec::<Entity>) {
        self.cells = cells;
    }
}




pub fn quadratic_interpolation_weights(cell_diff: Vec3A) -> [Vec3A; 3] {
    if cell_diff.x.abs() > 1.5 || cell_diff.y.abs() > 1.5 || cell_diff.z.abs() > 1.5 {
        println!("WARNING: cell_diff distance is more than expected");
    }
    [
        // cell to the 'left'/'previous'/idx-1: cell_diff would be
        Vec3A::new(
            0.5 * f32::powi(0.5 - cell_diff.x, 2),
            0.5 * f32::powi(0.5 - cell_diff.y, 2),
            0.5 * f32::powi(0.5 - cell_diff.z, 2),
        ),
        Vec3A::new(
            0.75 - f32::powi(cell_diff.x, 2),  // quadratic kernel weight for < 1/2 of cell: 3/4-|x|^2   MPM-course EQ 123
            0.75 - f32::powi(cell_diff.y, 2),
            0.75 - f32::powi(cell_diff.z, 2),
        ),
        Vec3A::new(
            0.5 * f32::powi(0.5 + cell_diff.x, 2),
            0.5 * f32::powi(0.5 + cell_diff.y, 2),
            0.5 * f32::powi(0.5 + cell_diff.z, 2),
        ),
    ]
}


pub fn weighted_velocity_and_cell_dist_to_term(
    weighted_velocity: Vec3A,
    cell_dist: Vec3A,
) -> Mat3A {
    Mat3A::from_cols(
        weighted_velocity * cell_dist.x,
        weighted_velocity * cell_dist.y,
        weighted_velocity * cell_dist.z,
    )
}


pub fn setup_fluid_grid(
    tank_cfg: Res<Tank>,
    mut commands: Commands,
) {
    let tank_size = tank_cfg.get_size();

    let mut grid = Grid::new(
        tank_size.as_uvec3(),
        1.0,
    );
    let ptank = tank_cfg.get_tank_parent();

    let mut cells = Vec::<Entity>::with_capacity( grid.cell_count() );
    let mut temp_type_info = Vec::<(Entity, GridCellType)>::with_capacity( grid.cell_count() );

    for idx in 0..grid.cell_count() {
        let xyz = grid.to_3d(idx);

        // determine grid cell type
        let mut gct = GridCellType::Fluid;
        if xyz.x * xyz.y * xyz.z == 0 ||
            xyz.x >= grid.grid_dim.x - 1 || xyz.z >= grid.grid_dim.z - 1
        {
            gct = GridCellType::Solid;
        }
        if xyz.y >= grid.grid_dim.y - 1 {
            gct = GridCellType::Air;
        }

        let cell_id = commands
            .spawn((
                Transform::from_translation( xyz.as_vec3() ),
                Visibility::default(),
            ))
            .insert(gct.clone())
            .insert(FluidParticleVelocity(Vec3A::ZERO))
            .insert(FluidQuantityMass( 0.0 ))
            .insert(GridCellIndex( idx ))
            .insert(ColliderNormals( vec![] ))
            .id();
        cells.push( cell_id );
        temp_type_info.push( (cell_id, gct) );
    }

    for idx in 0..grid.cell_count() {
        let xyz = grid.to_3d(idx);
        if temp_type_info[ idx ].1 == GridCellType::Solid {
            let mut neighbors = GridFluidNeighbors( vec![] );
            for z in 0..3 {
                for y in 0..3 {
                    for x in 0..3 {
                        let tcell_xyz = xyz + UVec3{x, y, z};
                        if tcell_xyz.x * tcell_xyz.y * tcell_xyz.z != 0 &&
                            tcell_xyz.x <= grid.grid_dim.x && tcell_xyz.z <= grid.grid_dim.z
                        {
                            let ocell_xyz = tcell_xyz - UVec3::splat(1);
                            let ocidx = grid.index_of_vec(&ocell_xyz);
                            if temp_type_info[ocidx].1 == GridCellType::Fluid {
                                neighbors.0.push( ocidx );
                            }
                        }
                    }
                }
            }
            commands.entity( temp_type_info[ idx ].0 )
                .insert(neighbors);
        }
    }

    commands.entity( ptank ).add_children( &cells );
    grid.initialize( cells );
    commands.insert_resource(grid);
}


pub fn grid_initialize_external_forces(
    constants: Res<Constants>,
    mut commands: Commands,
    mut cells: Query<(Entity, &Transform, &GridCellType)>,
    ext_forces: Query< &ExternalForceVolume >,
) {
    let gravity = Vec3::Y * constants.DEFAULT_GRAVITY;
    // walk through all cells
    cells.iter_mut().for_each( | ( cid, pos, gct ) | {

        // determine position-dependent external forces
        let ext_f = if *gct == GridCellType::Fluid {
            let mut acc_force = gravity;
            ext_forces.iter().for_each( | force_location | {
                acc_force += force_location.get_force_for_position( pos.translation )
            });
            acc_force
        } else {
            Vec3::ZERO
        };
        // if ext_f != Vec3::ZERO && ext_f != gravity {
        //     println!("grid_initialize_external_forces::Ext-Force > Grav {} at {}", ext_f, pos.translation);
        // }
        commands.entity( cid )
            .insert(GridCellAccumulatedForce( Vec3A::from(ext_f) ));
    });
}

pub fn grid_collider_setup(
    mut cells: Query<(&mut GridCellType, &Transform, &mut ColliderNormals)>,
    colliders: Query<(&Transform, &Collider), With<DecorationTag>>,
) {
    let dist_thresh = 0.5;

    // walk through all cells
    cells.iter_mut().for_each( | (mut gct, pos, mut cnorm) | {

        // and check for all colliders whether the cell touches that collider in any way
        colliders.iter().for_each(| (cloc, c) | {
            let (_sc, ro, _tr) = (cloc.scale, cloc.rotation, cloc.translation);
            let ccenter = pos.translation;
            if let Some( _pp ) = c.project_point_with_max_dist( cloc.translation, ro,
                                                                ccenter, false,
                                                                dist_thresh) {
                // println!("GRID: {} close to collider at: {}: {}", pos.translation, pp.is_inside, pp.point );
                *gct = GridCellType::Solid;
            } else if let Some( pp ) = c.project_point_with_max_dist( cloc.translation, ro,
                                                                      ccenter, false,
                                                                      dist_thresh*2.0 /*f32::sqrt(2.0)*0.75*/) {
                cnorm.0.push( Vec3A::from( (pp.point - ccenter).normalize_or_zero() ) );
            }
        });
    });
}

pub fn reset_fluid_grid_cells(
    mut grid: ResMut<Grid>,
    mut cells: Query<(&mut FluidQuantityMass, &mut FluidParticleVelocity), With<GridCellType>>
) {
    cells.par_iter_mut().for_each(
        | (mut mass, mut velo) | {
            mass.0 = 0.0;
            velo.0 = Vec3A::ZERO;
        }
    );
    grid.reset_tmp_mass();
    grid.reset_tmp_velo();

}

pub fn wall_to_active_momentum(
    cells: Query<(&FluidQuantityMass,
                  &FluidParticleVelocity,
                  &GridFluidNeighbors,
    ), With<GridFluidNeighbors>>,
    mut grid: ResMut<Grid>,
) {
    cells.iter().for_each(
        | (mass, vel, fluid_neighbors) | {
            let dmass = mass.0/fluid_neighbors.0.len() as f32 * 2.0;
            let dvel = vel.0/fluid_neighbors.0.len() as f32 * 2.0;
            for &fcell in &fluid_neighbors.0 {
                grid.tmp_mass[ fcell ] += dmass;
                grid.tmp_velo[ fcell ] += dvel;
            }
        }
    )
}

pub fn update_grid_cells(
    constants: Res<Constants>,
    mut cells: Query<(&FluidQuantityMass,
                      &mut FluidParticleVelocity,
                      &GridCellAccumulatedForce,
                      &GridCellType,
                      &ColliderNormals,
    )>,
) {
    let _lookahead = 1.0;

    cells.par_iter_mut().for_each(
        | ( mass, mut vel, ext_f, gct, cnorm ) | {

            if *gct == GridCellType::Solid {
                vel.0 = Vec3A::ZERO;
            } else {
                // convert momentum to velocity and apply external force and dampening
                if mass.0 > 0.0 {
                    vel.0 *= 1.0/mass.0;
                    vel.0 += ext_f.0 * constants.WORLD_DT;

                    if ! cnorm.0.is_empty() {
                        // collect all projected velocities
                        // let mut proj_vect = vec![];
                        let mut vel_new = vel.0;
                        cnorm.0.iter().for_each(| &norm | {
                            if norm != Vec3A::ZERO {
                                // dot product: projecting velocity onto the normal
                                // (this is going to be the part of the velocity that we're going to loose because of the collision)
                                // And make sure we're only considering situations where the velocity goes against the direction of the normal
                                let dot_prod = vel_new.dot( norm );
                                if dot_prod < 0.0 {
                                    let delta = norm * dot_prod;  // delta is the result of projecting vel.0 onto the neg. norm unit vector
                                    vel_new -= delta;             // subtract the projected amount of velocity from the remaining velocity
                                    // proj_vect.push(norm * dot_prod);
                                }
                            }
                        });
                        let delta_v_factor = f32::sqrt( vel.0.length_squared() / vel_new.length_squared() ) * 0.5;
                        // let delta_v_factor = if !cnorm.0.is_empty() && vel_new != Vec3::ZERO {
                        //     if vel.0.length() < vel_new.length() {
                        //         println!("{} < {}", vel.0.length(), vel_new.length() );
                        //     }
                        //     assert!( vel.0.length() >= vel_new.length() * 0.999 );
                        //     (vel.0.length() - vel_new.length() * 0.999) / vel_new.length()
                        // } else {
                        //     1.0
                        // };
                        vel.0 = vel_new * delta_v_factor; // * constants.DEFAULT_DAMPENING;
                    }
                }
            }
        }
    );
}


pub fn show_grid_cells(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    cells: Query<(Entity,
        &Transform,
        &ColliderNormals,
        &GridCellType),
        With<GridCellIndex>>,
) {
    if !DEBUG_GRID {
        return
    }

    let grid_center_mesh = meshes.add(Mesh::from(ZCone {
        radius: 0.05,
        height: 0.5,
        subdivisions: 5,
    }));
    let grid_center_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.5, 0.1, 0.1, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let grid_fluid_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.0, 1.0, 0.0, 1.0),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    let grid_air_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.8, 0.8, 1.0, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    cells.iter().for_each(
        | (item, position, cn, gct) | {
            if ! cn.0.is_empty() {
                // println!("showgrid::loc: {}; cellvec: {}", position.translation, cn.0[0] );
                let lookat = position.translation + Vec3::from( cn.0[0] );
                commands.entity(item).insert((
                    Mesh3d( grid_center_mesh.clone() ),
                    MeshMaterial3d( match gct {
                        GridCellType::Fluid => grid_fluid_material_hdl.clone(),
                        GridCellType::Air => grid_air_material_hdl.clone(),
                        GridCellType::Solid => grid_center_material_hdl.clone(),
                    }),
                    Transform::from_translation( position.translation ).looking_at(lookat, Vec3::Y), //position.translation ),
                ));
            }
        }
    );
}




pub fn debug_grid_cells(

    mut cells: Query<(&FluidParticleVelocity, &ColliderNormals, &mut Transform), With<GridCellType>>,
) {
    if !DEBUG_GRID {
        return
    }
    cells.par_iter_mut().for_each(
        | (vel, cn, mut tf) | {
            if !cn.0.is_empty() {
                let srcloc = tf.translation - Vec3::from(vel.0);  // USE '-' vel.0 because look_at point rotates towards neg Z!!!!
                tf.look_at( srcloc, Vec3::Y );
            }
        }
    );
}





#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn test_grid_scale() {
        let grid = Grid::new( UVec3::new( 160, 80, 75 ),
                              5.);
        let min_cell_dim = 5.0;
        assert_eq!(grid.get_scale(), min_cell_dim);

        let expected_cells = ((160. / min_cell_dim)+2.) * ((80./min_cell_dim)+4.) * ((75./min_cell_dim)+2.);
        assert_eq!(grid.cell_count(), expected_cells as usize);
    }

    #[test]
    fn test_normals() {
        let cn = Vec3::new( 0.5, 0.0, 0.0).normalize();
        let vel = Vec3::new(-2.0, -1.0, 0.0);

        println!("dot: {}, {}", vel.dot( cn ), cn );
        println!("projected: {}", vel - vel.dot( cn ) * cn);
    }
}
