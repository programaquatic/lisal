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


use std::ops::Sub;

use bevy::{
    prelude::*,
    math::{
        Vec3A,
        Mat3A,
    },
};

use crate::aqs_utils::constants;

use crate::water::{
    resources,
    grid,
    grid::{GridCellType, GridCellIndex},
};

/// STEP: 0 resetting the grid
// pub fn reset_grid(mut grid: ResMut<grid::Grid>) {
//     grid.reset();
// }

// STEP: 1
// Collecting the grid quantities onto each cell cmma
pub fn p2g_stage1(
    grid: Res<grid::Grid>,
    mut particles: Query<
        (
            &resources::FluidParticlePosition,
            &resources::FluidParticleVelocity,
            &resources::FluidQuantityMass,
            &resources::AffineMomentum,
            &mut resources::CellMMAccumulation,
        ),
        With<resources::ParticleTag>,
        >,
) {
    particles.par_iter_mut().for_each_mut(
        |(location, velocity, mass, affine_momentum, mut cmma)| {
            // assert_eq!(location.0.is_nan(), false);
            let cell_idx = location.0.as_uvec3();
            let cell_diff = (location.0 - cell_idx.as_vec3a()) - 0.5;

            let weights = grid::quadratic_interpolation_weights(cell_diff);
            // if !grid.cell_at_vec_is_fluid( &cell_idx ) {
            //     println!("{}, {}", cell_idx, location.0 );
            //     assert!( false );
            // }

            //collect momentum changes for surrounding 27 cells.
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let cell_pos = UVec3::new(
                            (cell_idx.x as i32 + gx as i32 - 1) as u32,
                            (cell_idx.y as i32 + gy as i32 - 1) as u32,
                            (cell_idx.z as i32 + gz as i32 - 1) as u32,
                        );
                        let cell_dist = (cell_pos.as_vec3a() - location.0) + Vec3A::splat(0.5);
                        let cell_at_index = grid.index_of_vec( &cell_pos );

                        let q = affine_momentum.0 * cell_dist;
                        let mass_contrib = weight * mass.0;

                        // mass and momentum update
                        cmma.0[gx + 3 * gy + 9 * gz] = resources::CellMMAChange {
                            cell_idx: cell_at_index,
                            mass: mass_contrib,
                            momentum: (velocity.0 + q) * mass_contrib,
                        };
                    }
                }
            }
        },
    );
}

// Helper system to go over each particle and accumulate the grid-cell computation results
pub fn p2g_apply_stage1(
    mut grid: ResMut<grid::Grid>,
    particles: Query<(&resources::CellMMAccumulation,), With<resources::ParticleTag>>,
) {
    particles.for_each(|cmma| {
        for change in cmma.0 .0.iter() {
            grid.get_tmp_mass_mut()[ change.cell_idx ] += change.mass;
            grid.get_tmp_velo_mut()[ change.cell_idx ] += change.momentum;
        }
    });
}

// STEP: 2
pub fn p2g_stage2(
    constants: Res<constants::Constants>,
    grid: Res<grid::Grid>,
    mut flparticles: Query<
        (
            &resources::FluidParticlePosition,
            &resources::FluidQuantityMass,
            &resources::AffineMomentum,
            &mut resources::CellMMAccumulation,
        ),
        With<resources::ParticleTag>,
        >,
) {
    flparticles.par_iter_mut().for_each_mut(
        |(location, quantity, affmom, mut cmma)| {
            let mut density: f32 = 0.0;

            let cell_idx = location.0.as_uvec3();
            let cell_diff = (location.0 - cell_idx.as_vec3a()) - 0.5;

            let weights = grid::quadratic_interpolation_weights(cell_diff);

            // println!("----- next particle {} -> {}-------", location.0, cell_pos);
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let cell_pos = UVec3::new(
                            (cell_idx.x as i32 + gx as i32 - 1) as u32,
                            (cell_idx.y as i32 + gy as i32 - 1) as u32,
                            (cell_idx.z as i32 + gz as i32 - 1) as u32,
                        );
                        let cell_at_index = grid.index_of_vec( &cell_pos );

                        density += grid.get_tmp_mass()[ cell_at_index ] * weight;
                    }
                }
            }
            // virtual volume of the particle
            let volume = quantity.0 / density;

            // fluid constitutive model
            let pressure = f32::max(
                -0.1,
                constants.FLUID_MODEL.eos_stiffness
                    * (f32::powf(density / constants.FLUID_MODEL.rest_density,
                                 constants.FLUID_MODEL.eos_power) - 1.0),
            );
            let mut stress = Mat3A::from_cols(
                Vec3A::new(-pressure, 0.0, 0.0),
                Vec3A::new(0.0, -pressure, 0.0),
                Vec3A::new(0.0, 0.0, -pressure),
            );
            let mut strain = affmom.0;
            let trace = strain.determinant();
            strain.z_axis.x = trace;
            strain.y_axis.y = trace;
            strain.y_axis.z = trace;
            let viscosity_term: Mat3A = strain * constants.FLUID_MODEL.dynamic_viscosity;
            stress += viscosity_term;

            let eq_16_term_0 = -volume * 4.0 * stress * constants.WORLD_DT;

            // for all surrounding 27 cells
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let cell_pos = UVec3::new(
                            (cell_idx.x as i32 + gx as i32 - 1) as u32,
                            (cell_idx.y as i32 + gy as i32 - 1) as u32,
                            (cell_idx.z as i32 + gz as i32 - 1) as u32,
                        );

                        let cell_dist = (cell_pos.as_vec3a() - location.0) + Vec3A::splat(0.5);
                        let cell_at_index = grid.index_of_vec( &cell_pos );
                        let new_momentum = (eq_16_term_0 * weight) * cell_dist;
                        cmma.0[gx + 3 * gy + 9 * gz] = resources::CellMMAChange {
                            cell_idx: cell_at_index,
                            mass: 0.,
                            momentum: new_momentum,
                        };
                    }
                }
            }
        },
    );
}


pub fn p2g_stage2_solids(
    constants: Res<constants::Constants>,
    grid: Res<grid::Grid>,
    mut sdparticles: Query<
            (
                &resources::FluidParticlePosition,
                &resources::FluidQuantityMass,
                &resources::AffineMomentum,
                &mut resources::CellMMAccumulation,
            ),
        With<resources::SolidParticleTag>,
        >,
) {
    let num_particles = sdparticles.iter().count();
    if num_particles < 1 {
        return;
    }
    sdparticles.par_iter_mut().for_each_mut(
        |(location, mass, _, mut mmc)| {
            let mut density: f32 = 0.0;

            let cell_idx = location.0.as_uvec3();
            let cell_diff = (location.0 - cell_idx.as_vec3a()) - 0.5;

            let weights = grid::quadratic_interpolation_weights(cell_diff);

            // check surrounding 27 cells to get volume from density
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let cell_pos = UVec3::new(
                            (cell_idx.x as i32 + gx as i32 - 1) as u32,
                            (cell_idx.y as i32 + gy as i32 - 1) as u32,
                            (cell_idx.z as i32 + gz as i32 - 1) as u32,
                        );
                        let cell_at_index = grid.index_of_vec( &cell_pos );

                        density += grid.get_tmp_mass()[ cell_at_index ] * weight;
                    }
                }
            }

            let volume = mass.0 / density;

            let pp = &constants.ELASTIC_MODEL;
            let j: f32 = pp.deformation_gradient.determinant();
            let volume_scaled = volume * j;

            let f_t: Mat3A = pp.deformation_gradient.transpose();
            let f_inv_t = f_t.inverse();
            let f_minus_f_inv_t = pp.deformation_gradient.sub(f_inv_t);

            let p_term_0: Mat3A = f_minus_f_inv_t.mul_scalar(pp.elastic_mu);
            let p_term_1: Mat3A = f_inv_t.mul_scalar(j.ln() * pp.elastic_lambda);
            let p_combined: Mat3A = p_term_0.add_mat3(&p_term_1);

            let stress: Mat3A = p_combined.mul_mat3(&f_t).mul_scalar(1.0 / j);
            let eq_16_term_0 = stress * (-volume_scaled * 4.0 * constants.WORLD_DT);

            // for all surrounding 27 cells
            for gz in 0..3 {
                for gy in 0..3 {
                    for gx in 0..3 {
                        let weight = weights[gx].x * weights[gy].y * weights[gz].z;
                        let cell_pos = UVec3::new(
                            (cell_idx.x as i32 + gx as i32 - 1) as u32,
                            (cell_idx.y as i32 + gy as i32 - 1) as u32,
                            (cell_idx.z as i32 + gz as i32 - 1) as u32,
                        );

                        let cell_dist = (cell_pos.as_vec3a() - location.0) + Vec3A::splat(0.5);
                        let cell_at_index = grid.index_of_vec( &cell_pos );

                        // store the fused force/momentum update from MLS-MPM to apply onto grid later.
                        // todo combine into grid(x,y) = total changes as they come in here...?
                        mmc.0[gx + 3 * gy + 9 * gz] = resources::CellMMAChange {
                            cell_idx: cell_at_index,
                            mass: 0.,
                            momentum: eq_16_term_0.mul_scalar(weight).mul_vec3a(cell_dist),
                        };
                    }
                }
            }
        },
    );
}


pub fn grid_update(
    mut grid: ResMut<grid::Grid>,
    particles: Query<(&resources::CellMMAccumulation,), With<resources::ParticleTag>>,
    mut cells: Query<(
        &mut resources::FluidParticleVelocity,
        &mut resources::FluidQuantityMass,
        &GridCellIndex
    ), With<GridCellType>>,
) {
    particles.for_each(|cmma| {
        for change in cmma.0 .0.iter() {
            grid.get_tmp_velo_mut()[ change.cell_idx ] += change.momentum;
        }
    });

    cells.par_iter_mut().for_each_mut(
        | (mut vel, mut mass, idx) | {
            vel.0 = grid.get_tmp_velo()[ idx.0 ];
            mass.0 = grid.get_tmp_mass()[ idx.0 ];
        }
    );
}
