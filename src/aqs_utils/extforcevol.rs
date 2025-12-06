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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ForceVolumeDirection {
    Inward(f32),
    Outward(f32),
    Parallel(Vec3),
}

#[allow(dead_code)]
impl ForceVolumeDirection {
    pub fn from_parallel(direction: Vec3) -> Self {
        ForceVolumeDirection::Parallel(direction)
    }

    pub fn from_inward(speed: f32) -> Self {
        ForceVolumeDirection::Inward(speed)
    }

    pub fn from_outward(speed: f32) -> Self {
        ForceVolumeDirection::Outward(speed)
    }
}

/// An external force that's going to be applied to a fluid grid cell
///  note: this is meant for individual pieces of ext force, i.e. the sources and not the effective ext force
#[derive(Component, Resource, Serialize, Deserialize, Debug, Clone)]
pub struct ExternalForceVolume {
    pub location: Vec3,
    pub extent: Vec3,
    pub direction: ForceVolumeDirection,
    pub name: Option<String>,
}

impl Default for ExternalForceVolume {
    /// Default ForceVolume will create a parallel force field throughout the volume
    fn default() -> Self {
        ExternalForceVolume {
            location: Vec3::ZERO,
            extent: Vec3::ONE,
            direction: ForceVolumeDirection::Parallel(Vec3::ONE),
            name: None,
        }
    }
}

impl ExternalForceVolume {
    #[allow(dead_code)]
    pub fn new(
        location: Vec3,
        extent: Vec3,
        direction: ForceVolumeDirection,
        name: Option<String>,
    ) -> Self {
        ExternalForceVolume {
            location,
            extent,
            direction,
            name,
        }
    }

    // pub fn relative_distance(&self, refpoint: Vec3) -> (Vec3, f32) {
    //     let refpoint_distance = refpoint - self.location;
    //     let min_extent = self.extent.min_element().powi(2);
    //     (refpoint_distance, refpoint_distance.length_squared()/min_extent)
    // }

    pub fn get_force_for_position(&self, refpoint: Vec3) -> Vec3 {
        let floc = (refpoint - self.location).abs();
        let fextent_mask = floc.cmplt(self.extent).all();
        let outward_norm = (refpoint - self.location).normalize_or_zero();
        let force = match self.direction {
            ForceVolumeDirection::Inward(speed) => -outward_norm * speed,
            ForceVolumeDirection::Outward(speed) => outward_norm * speed,
            ForceVolumeDirection::Parallel(dir) => dir,
        };
        force * (fextent_mask as u32) as f32
    }

    pub fn scale(&mut self, scale: f32) {
        self.location *= scale;
        self.extent *= scale;
        // direction is not scaled
        self.direction = match self.direction {
            ForceVolumeDirection::Inward(speed) => ForceVolumeDirection::Inward(speed * scale),
            ForceVolumeDirection::Outward(speed) => ForceVolumeDirection::Outward(speed * scale),
            ForceVolumeDirection::Parallel(dir) => ForceVolumeDirection::Parallel(dir * scale),
        };
    }
}
