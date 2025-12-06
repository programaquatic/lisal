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
    math::{Mat3A, Vec3A},
    prelude::*,
};

#[derive(Component)]
pub struct ParticleFrameTag;

#[derive(Component)]
pub struct ParticleTag(pub usize);

#[derive(Component)]
pub struct SolidParticleTag(pub usize);

#[derive(Component, Debug)]
pub struct FluidParticlePosition(pub Vec3A);

#[derive(Component, Debug)]
pub struct FluidParticleVelocity(pub Vec3A);

#[derive(Component, Debug)]
pub struct FluidQuantityMass(pub f32);

// computed changes to-be-applied to grid on next steps
#[derive(Clone, Copy)]
pub struct CellMMAChange {
    pub cell_idx: usize,
    pub mass: f32,
    pub momentum: Vec3A,
}

#[derive(Component)]
pub struct CellMMAccumulation(pub(super) [CellMMAChange; 27]);

// 2x2 affine momentum matrix
#[derive(Component)]
pub struct AffineMomentum(pub Mat3A);

#[derive(Component)]
pub struct ParticleCount(pub usize);
