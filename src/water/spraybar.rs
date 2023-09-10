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

use bevy::math::Vec3;
use rand::{
    Rng,
    rngs::StdRng,
};


pub struct SprayBar {
    center: Vec3,
    extent: Vec3,
    precalc: Vec<Vec3>,
    precalc_count: usize,
    rng: StdRng,
}

impl SprayBar {
    pub fn new( center: Vec3, extent: Vec3 ) -> Self {
        Self {
            center,
            extent,
            precalc: vec![ Vec3::ZERO; 1 ],
            precalc_count: 1,
            rng: rand::SeedableRng::from_entropy(),
        }
    }

    #[allow(dead_code)]
    pub fn precomp(&mut self, count: usize) {
        self.precalc_count = count;
        self.precalc.resize_with(
            count,
            || {
                self.center
                    + Vec3::new(
                        self.rng.gen_range(-100.0..100.0) / 201. * self.extent.x,
                        self.rng.gen_range(-100.0..100.0) / 201. * self.extent.y,
                        self.rng.gen_range(-100.0..100.0) / 201. * self.extent.z,
                    )
            });
    }

    pub fn new_position(&mut self) -> Vec3 {
        self.center
            + Vec3::new(
                self.rng.gen_range(-100.0..100.0) / 201. * self.extent.x,
                self.rng.gen_range(-100.0..100.0) / 201. * self.extent.y,
                self.rng.gen_range(-100.0..100.0) / 201. * self.extent.z,
            )
    }

    #[allow(dead_code)]
    pub fn precomp_position(&self, idx: usize) -> Vec3 {
        self.precalc[ idx % self.precalc_count ]
    }

}
