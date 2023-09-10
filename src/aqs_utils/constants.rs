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
    math::Mat3A,
};
use serde::{Serialize, Deserialize};

use crate::aqs_utils::{
    config as cfg,
};


// fluid constitutive model properties
#[derive(Resource, Serialize, Deserialize, Debug, Default)]
pub struct FluidModel {
    pub rest_density: f32,
    pub dynamic_viscosity: f32,
    pub eos_stiffness: f32,
    pub eos_power: f32,
}

#[derive(Resource, Serialize, Deserialize, Debug, Default)]
pub struct NeoHookeanHyperElasticModel {
    pub deformation_gradient: Mat3A,
    pub elastic_lambda: f32,
    pub elastic_mu: f32,
}



#[derive(Resource, Serialize, Deserialize, Debug)]
pub struct ParticleVisibilityConf {
    pub base: bool,
    pub fill: bool,
    pub spec: bool,
}

impl Default for ParticleVisibilityConf {
    fn default() -> Self {
        ParticleVisibilityConf {
            base: false,
            fill: false,
            spec: true,
        }
    }
}


#[allow(non_snake_case)] // allow those constants to be uppercase var names
#[derive(Resource, Serialize, Deserialize, Debug)]
pub struct Constants {
    pub MAX_GRID_CELLS: usize,
    pub WORLD_DT: f32,
    pub DEFAULT_GRAVITY: f32,

    pub DEFAULT_DENSITY: Vec2,
    pub DEFAULT_PARTICLE_MASS: f32,
    pub DEFAULT_FILL_HEIGHT: f32,
    pub DEFAULT_DAMPENING: f32,

    pub MAX_PARTICLES: usize,
    pub VISIBLE_PARTICLES: usize,

    #[serde(default)]
    pub FLUID_MODEL: FluidModel,
    #[serde(default)]
    pub ELASTIC_MODEL: NeoHookeanHyperElasticModel,

    #[serde(default)]
    pub DEBUG_FLUID_PARTICLES: ParticleVisibilityConf,

    #[serde(default)]
    pub DEFAULT_PPC: u32,
}

impl FromWorld for Constants {
    fn from_world( _world: &mut World ) -> Self {
        let mut aqs_constants: Constants = cfg::read_json::<Constants>(String::from("assets/constants.json")).unwrap();

        let fluid_model = FluidModel {
            rest_density: aqs_constants.DEFAULT_DENSITY.y,
            dynamic_viscosity: 0.001,
            eos_stiffness: 10.,
            eos_power: 4.,
        };
        let elastic_model = NeoHookeanHyperElasticModel {
            deformation_gradient: Default::default(),
            elastic_lambda: 180. * 1000.,
            elastic_mu: 78. * 1000.,
        };
        aqs_constants.FLUID_MODEL = fluid_model;
        aqs_constants.ELASTIC_MODEL = elastic_model;
        aqs_constants.DEFAULT_PPC = aqs_constants.DEFAULT_DENSITY.x as u32;

        aqs_constants
    }
}
