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
    prelude::*, math::{Vec3A, Mat3A},
};

use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{
    aqs_utils::constants::Constants,
    tech::{
        tank::Tank,
        pump::Pump,
    },
    water::{
        grid::{GridCellType, GridCellIndex, Grid},
        grid,
        resources,
        mlsmpm,
        surface,
        spraybar::SprayBar,
    },
};

pub const BOUNDARY_WALL_MARGIN: f32 = 1.5;
pub const WPARTICLE_RADIUS: f32 = 0.1;

pub struct FluidPlugin;

// #[derive(Component)]
// pub struct SurfaceProducer;


#[derive(Component)]
pub struct ColliderExperiment;


fn fill_tank(
    constants: Res<Constants>,
    tank_cfg: Res<Tank>,
    grid: Res<Grid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut particle_frame: Query<(Entity, &mut resources::ParticleCount), With<resources::ParticleFrameTag>>,
) {
    let visible_particles = usize::min( constants.VISIBLE_PARTICLES, constants.MAX_PARTICLES );
    let inlet = &tank_cfg.get_pump_definition().inlet;
    let mut spraybar = SprayBar::new( inlet.location, inlet.extent );

    let (id, mut count) = particle_frame.get_single_mut().unwrap();
    if count.0 > constants.MAX_PARTICLES {
        return;
    }

    // fake inlet pump (location based)
    let pump_v = inlet.get_force_for_position(inlet.location) * 0.25; // * constants.WORLD_DT;

    let particle_radius = WPARTICLE_RADIUS / grid.get_scale();

    // spawn N particles
    for _ in 0..10 {
        if count.0 % 1000 == 0 {
            println!("Particles in play: {}", count.0);
        }
        let red = if count.0 % (constants.MAX_PARTICLES / constants.VISIBLE_PARTICLES) == 0 { 1.0 } else { 0.0 };
        let water_material = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(red, 0.03, 1.0, 0.8),
            // alpha_mode: AlphaMode::Blend,
            ..default()
        });

        let wiggle = spraybar.new_position();

        let visible;
        let particle = if count.0 % (constants.MAX_PARTICLES / visible_particles) == 0 {
            visible = constants.DEBUG_FLUID_PARTICLES.spec;
            commands// only make one visible particle
                .spawn((
                    Mesh3d(meshes.add(Sphere::new(particle_radius + (particle_radius * red)).mesh().ico(4).unwrap())),
                    MeshMaterial3d(water_material.clone()),
                    Transform::from_translation( wiggle ),
                ))
                    // .insert(ColliderExperiment)
                    // .insert(RigidBody::KinematicPositionBased)
                    // .insert(Collider::ball( particle_radius / grid.get_scale() ))
                    // .insert(Group::GROUP_1)
                    // .insert(LockedAxes::ROTATION_LOCKED)
                    // .insert(Velocity {
                    //     linvel: pump_v,
                    //     ..default()
                    // })


                .insert(resources::FluidParticlePosition(Vec3A::from(wiggle)))
                .insert(resources::FluidParticleVelocity(Vec3A::from(pump_v)))
                .insert(resources::FluidQuantityMass( constants.DEFAULT_PARTICLE_MASS ))
                .insert(resources::AffineMomentum(Mat3A::ZERO))
                .insert(resources::CellMMAccumulation(
                    [resources::CellMMAChange {
                        cell_idx: 0,
                        mass: 0.0,
                        momentum: Vec3A::ZERO,
                    }; 27],
                ))
                .insert(resources::ParticleTag( count.0 + 100000 ))
                .id()
        } else { //  otherwise spawn a particle without visibility
            visible = constants.DEBUG_FLUID_PARTICLES.fill;
            commands
                .spawn((
                    Transform::from_translation( wiggle ),
                    Visibility::default(),
                ))
                .insert(resources::FluidParticlePosition(Vec3A::from(wiggle)))
                .insert(resources::FluidParticleVelocity(Vec3A::from(pump_v)))
                .insert(resources::FluidQuantityMass( constants.DEFAULT_PARTICLE_MASS ))
                .insert(resources::AffineMomentum(Mat3A::ZERO))
                .insert(resources::CellMMAccumulation(
                    [resources::CellMMAChange {
                        cell_idx: 0,
                        mass: 0.0,
                            momentum: Vec3A::ZERO,
                    }; 27],
                ))
                .insert(resources::ParticleTag( count.0 ))
                .id()
        };
        // insert particle as children
        commands.entity(id).add_child(particle);

        // visualize every added particle (if configured)
        if visible {
            commands.entity( particle )
                .insert(Mesh3d(meshes.add(Sphere::new(particle_radius).mesh().ico(8).unwrap())))
                .insert(MeshMaterial3d(water_material.clone()));
        }
        count.0 += 1;
    }
}


// derive/create temporary (per iteration) Lagrangian particles with velocities
fn init_fluid_particle_system(
    grid: Res<Grid>,
    constants: Res<Constants>,
    cells: Query< (&Transform, &grid::GridCellType, &GridCellIndex)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();

    let mut water_material : [Handle<StandardMaterial>; 4] = Default::default();

    for (c, m) in  water_material.iter_mut().enumerate() {
        *m = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(0.0, 0.0, 1.0/(c as f32), 0.5),
            // alpha_mode: AlphaMode::Blend,
            ..default()
        });
    }

    let water_material_hdl = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
        reflectance: 0.0,
        ..default()
    });

    let mut particle_id = 0;

    let particle_frame = commands
        .spawn((
            Name::new("Particle_Frame"),
            resources::ParticleFrameTag,
            resources::ParticleCount(0),
            Transform::from_translation( grid.to_world_coord( -Vec3::ONE )),
                // .with_scale( Vec3::splat( grid.get_scale()) ),
            Visibility::default(),
        ))
        .id();


    let _particle_radius = WPARTICLE_RADIUS / grid.get_scale();
    let fill_height = constants.DEFAULT_FILL_HEIGHT * grid.grid_size().y as f32;

    cells.iter().for_each(
        | ( position, gct, cidx ) | if *gct == grid::GridCellType::Fluid {
            // grid::GridCellType::Fluid => {
            // println!("Cell_idx: {}", idx);
            let extra_particle = i32::from( cidx.0 % 20 == 0 );
            for _ in 0..1 + extra_particle {
                let wiggle = position.translation
                    + Vec3::new(
                        rng.gen_range(0.0..399.0) / 400.,
                        rng.gen_range(0.0..399.0) / 400.,
                        rng.gen_range(0.0..399.0) / 400.,
                    );
                if wiggle.y > fill_height {
                    continue;
                }
                let particle = commands
                            .spawn((
                                Transform::from_translation( wiggle ),
                                Visibility::default(),
                            ))
                    .insert(resources::FluidParticlePosition(Vec3A::from(wiggle)))
                    .insert(resources::FluidParticleVelocity(Vec3A::ZERO))
                    .insert(resources::FluidQuantityMass( constants.DEFAULT_PARTICLE_MASS ))
                    .insert(resources::AffineMomentum(Mat3A::ZERO))
                    .insert(resources::CellMMAccumulation(
                        [resources::CellMMAChange {
                            cell_idx: 0,
                            mass: 0.0,
                                    momentum: Vec3A::ZERO,
                        }; 27],
                    ))
                    .insert(resources::ParticleTag( particle_id ))
                    .id();

                particle_id += 1;

                // insert particle as children
                commands.entity(particle_frame).add_child(particle);
                if constants.DEBUG_FLUID_PARTICLES.base {
                    commands.entity(particle)
                    //// Uncomment if you want to see all particles
                        .insert(Mesh3d(meshes.add(Sphere::new(_particle_radius).mesh().ico(4).unwrap())))
                        .insert(MeshMaterial3d(water_material_hdl.clone()));
                }
            }
        }
    );
    println!("Cells: {}; Particles: {}", grid.cell_count(), particle_id );
}

pub fn grid_to_particle(
    constants: Res<Constants>,
    mut grid: ResMut<Grid>,
    mut particles: Query<
            (
                &mut resources::FluidParticlePosition,
                &mut resources::FluidParticleVelocity,
                &mut resources::AffineMomentum,
                &resources::ParticleTag,
            ), Without<GridCellType>
            >,
    cells: Query<(&GridCellIndex,  &resources::FluidParticleVelocity), With<GridCellType>>,
) {
    // let mut max_vel: f32 = 0.0;
    cells.iter().for_each( | (idx, vel) | {
        grid.get_tmp_velo_mut()[ idx.0 ] = vel.0;
    });

    particles.par_iter_mut().for_each(
        |(mut location, mut velocity, mut affine_momentum, _ptag)| {
            //// reset particle velocity. we calculate it from scratch each step using the grid
            velocity.0 = Vec3A::ZERO;

            let cell_pos = location.0.as_uvec3();
            let cell_diff = location.0 - cell_pos.as_vec3a() - Vec3A::splat(0.5);

            let weights = grid::quadratic_interpolation_weights(cell_diff);

            // affine per-particle momentum matrix from APIC / MLS-MPM.
            // see APIC paper (https://web.archive.org/web/20190427165435/https://www.math.ucla.edu/~jteran/papers/JSSTS15.pdf), page 6
            // below equation 11 for clarification. this is calculating C = B * (D^-1) for APIC equation 8,
            // where B is calculated in the inner loop at (D^-1) = 4 is a constant when using quadratic interpolation functions
            let mut b = Mat3A::ZERO;
            // for all surrounding 9 cells
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let neighbor = UVec3::new(
                            (cell_pos.x as i32 + gx as i32 - 1) as u32,
                            (cell_pos.y as i32 + gy as i32 - 1) as u32,
                            (cell_pos.z as i32 + gz as i32 - 1) as u32,
                        );
                        let cell_dist = (neighbor.as_vec3a() - location.0) + Vec3A::splat(0.5);
                        let cell_at_index = grid.index_of_vec( &neighbor );
                        let weighted_velocity = grid.get_tmp_velo()[ cell_at_index ] * weight;

                        b += grid::weighted_velocity_and_cell_dist_to_term(weighted_velocity, cell_dist);
                        velocity.0 += weighted_velocity;
                    }
                }
            }
            affine_momentum.0 = b * 4.0;
            location.0 += velocity.0 * constants.WORLD_DT;
        },
    );
}

pub fn particle_boundary_enforcement(
    constants: Res<Constants>,
    grid: Res<Grid>,
    mut particles: Query<
            (
                &mut resources::FluidParticlePosition,
                &mut resources::FluidParticleVelocity,
                &mut resources::AffineMomentum,
            ), Without<GridCellType>
            >,
    pumping: Query<&Pump>,
) {
    // predictive boundary velocity cap
    let wall_min: f32 = BOUNDARY_WALL_MARGIN;
    let wall_max: Vec3A = *grid.wall_vector()
        - Vec3A::splat(wall_min);

    particles.par_iter_mut().for_each(
        | (mut location, mut velocity, mut afmom) | {
            pumping.iter().for_each(| r | {
                if let Some( ( new_loc, vel_diff) ) = r.particle_pump(location.0) {
                    location.0 = new_loc;
                    velocity.0 = vel_diff;
                    afmom.0 = Mat3A::ZERO;
                }
            });

            location.0.x = location.0.x.clamp(1.001, grid.grid_size().x as f32 - 1.001);
            location.0.y = location.0.y.clamp(1.001, grid.grid_size().y as f32 - 1.001);
            location.0.z = location.0.z.clamp(1.001, grid.grid_size().z as f32 - 1.001);

            // apply boundary conditions about 0.1 seconds before reaching edge
            let dt_multiplier = 0.1 * constants.WORLD_DT;
            let position_next = location.0 + velocity.0 * dt_multiplier;

            if position_next.x < wall_min {
                velocity.0.x += wall_min - position_next.x;
            }
            if position_next.x > wall_max.x {
                velocity.0.x += wall_max.x - position_next.x;
            }
            if position_next.y < wall_min {
                velocity.0.y += wall_min - position_next.y;
            }
            if position_next.y > wall_max.y {
                velocity.0.y += wall_max.y - position_next.y;
            }
            if position_next.z < wall_min {
                velocity.0.z += wall_min - position_next.z;
            }
            if position_next.z > wall_max.z {
                velocity.0.z += wall_max.z - position_next.z;
            }
        }
    );
}

pub fn _collider_update(
    _constants: Res<Constants>,
    r3d_context: ReadDefaultRapierContext,
    mut particles: Query<( &GlobalTransform, &mut resources::FluidParticlePosition,
                            &mut resources::FluidParticleVelocity ), With<ColliderExperiment>>
) {
    // println!("Observed Particles: {}", particles.iter().len());
    particles.par_iter_mut().for_each(
        |(position, mut _part_location, mut velocity)| {
            if let Some( (_collision_with, ray_x) ) =
                r3d_context.cast_ray_and_get_normal(position.translation(),
                                                    Vec3::from( velocity.0 ),
                                                    0.1,
                                                    true,
                                                    QueryFilter::only_fixed() )
            {
                let projected_v = Vec3A::from( ray_x.normal ) * velocity.0.dot( ray_x.normal.into() );
                // part_location.0 += ray_x.normal * _constants.WORLD_DT;
                velocity.0 -= projected_v;
            }
        }
    );
}

pub fn particle_world_update(
    mut particles: Query<(&resources::FluidParticlePosition, &mut Transform)>,
) {
    particles.par_iter_mut().for_each( |(location, mut transform)| {
        transform.translation = location.0.into();
    });
}

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterialPlugin::<surface::CustomMaterial>::default())
            .add_systems(PreStartup, grid::setup_fluid_grid)
            .add_systems(Startup, surface::init_water_surface_system)
            .add_systems(Startup, grid::grid_initialize_external_forces)
            .add_systems(Startup, init_fluid_particle_system)
            .add_systems(Startup,
                grid::grid_collider_setup
                    .before(grid::show_grid_cells)
            )
            .add_systems(Startup,
                grid::show_grid_cells
            )

            .add_systems(Update,
                grid::reset_fluid_grid_cells
                    .before(mlsmpm::p2g_stage1))
            .add_systems(Update,
                mlsmpm::p2g_stage1
                    .before(mlsmpm::p2g_apply_stage1))
            .add_systems(Update,
                mlsmpm::p2g_apply_stage1
                    .before(mlsmpm::p2g_stage2))
            .add_systems(Update,
                mlsmpm::p2g_stage2
                    .before(mlsmpm::grid_update))
            .add_systems(Update,
                mlsmpm::p2g_stage2_solids
                         .before(grid::wall_to_active_momentum))
            .add_systems(Update,
                         grid::wall_to_active_momentum
                            .before(mlsmpm::grid_update))
            .add_systems(Update,
                mlsmpm::grid_update
                    .before(grid::update_grid_cells))
            .add_systems(Update,
                grid::update_grid_cells
                    .before(grid_to_particle))
            .add_systems(Update,
                surface::update_surface
                    .after(grid::update_grid_cells))
            // .add_systems(Update,
            //     grid::external_forces_grid_cells
            //         .label("grid_ext_forces")
            //         .before("g2p"))
            .add_systems(Update,
                grid_to_particle
                    .before(particle_boundary_enforcement))
            .add_systems(Update,
                particle_boundary_enforcement
                .before(particle_world_update))
            // .add_systems(Update,
            //     _collider_update
            //         .label("collider_update")
            //         .after("g2p")
            //         .before("particle_world_update"))
            .add_systems(Update,
                grid::debug_grid_cells
                    .after(grid::update_grid_cells))
            .add_systems(Update,
                particle_world_update
            )
            .add_systems(Update,fill_tank)
            ;
    }
}
