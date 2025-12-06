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

use bevy::math::f32::Vec3;

pub struct Scaler {
    to_scale: Vec3,
    from_scale: Vec3, // is just 1./to_scale; but put it here to avoid divisions
}

impl Default for Scaler {
    fn default() -> Self {
        Scaler {
            to_scale: Vec3::ONE,
            from_scale: Vec3::ONE,
        }
    }
}

impl Scaler {
    #[allow(dead_code)]
    pub fn from_vecs(origin: Vec3, target: Vec3) -> Self {
        let scale = target / origin;
        println!("NewScaler: {}", scale);
        Self {
            to_scale: scale,
            from_scale: 1. / scale,
        }
    }
    #[allow(dead_code)]
    pub fn from_scale(scale_factor: f32) -> Self {
        Self {
            to_scale: Vec3::splat(scale_factor),
            from_scale: Vec3::splat(1. / scale_factor),
        }
    }

    #[allow(dead_code)]
    pub fn to(&self, input: Vec3) -> Vec3 {
        input * self.to_scale
    }

    #[allow(dead_code)]
    pub fn from(&self, input: Vec3) -> Vec3 {
        input * self.from_scale
    }

    #[allow(dead_code)]
    pub fn is_isometric(&self) -> bool {
        self.to_scale.x == self.to_scale.y && self.to_scale.x == self.to_scale.z
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_scale() {
        let scaler = Scaler::default();

        let input = Vec3::new(5., 4., 3.);
        assert_eq!(scaler.to(input), input);
        assert_eq!(scaler.from(input), input);
        assert!(scaler.is_isometric());
    }

    #[test]
    fn test_new_scale_from_vecs() {
        // create world and grid sizes with different scales per dimension (10x, 20x, 5x)
        let world = Vec3::new(60., 40., 35.);
        let grid = Vec3::new(6., 2., 7.);
        let scaler = Scaler::from_vecs(world, grid);

        // sample coordinates for testing
        let w_input = Vec3::new(20., 20., 15.);
        let g_input = Vec3::new(2., 1., 3.);
        assert_eq!(scaler.to(w_input), g_input);
        assert_eq!(scaler.from(g_input), w_input);
        assert!(!scaler.is_isometric());
    }

    #[test]
    fn test_new_scale_from_scale() {
        // create world and grid sizes with different scales per dimension (10x, 20x, 5x)
        let scaler = Scaler::from_scale(2.0);

        // sample coordinates for testing
        let w_input = Vec3::new(20., 20., 15.);
        let g_input = Vec3::new(40., 40., 30.);
        assert_eq!(scaler.to(w_input), g_input);
        assert_eq!(scaler.from(g_input), w_input);
        assert!(scaler.is_isometric());
    }
}
