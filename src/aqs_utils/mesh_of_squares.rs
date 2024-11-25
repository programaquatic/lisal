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
    render::{
        mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology
    },
};
use rand::Rng;
use itertools::Itertools;


pub struct MeshOfSquares {
    indices: Vec<u16>,
    normals: Vec<Vec3>,
    locations: Vec<Vec3>,
    uvs: Vec<Vec2>,
    colors: Vec<Vec4>,
}

/** MeshOfSquares uses builder pattern to allow mesh creation and subsequent ops like randomization
    use .into_mesh() to complete the Mesh creation.
*/
impl MeshOfSquares {
/** generates a triangle strip mesh for a 2D area with locations and UV coords scaled according to input
    Inspired by: https://github.com/Neopallium/bevy_water/blob/main/src/generate.rs

    - the grid size equals the area size, world and UV coordinates
      will be scaled

    - first it generates the relative world locations and UVs scaled

    - then generates the triangle strip by traversing the x-dimension
      in alternating directions so that only one degenerated triangle
      appears at the end of each 'line'.

    - the alternating direction causes the last index of row N-1 to
      reappear in row N and therefore creating a degenerated triangle
      that connects 2 rows.

    - triange ordering looks like (not showing degenerated triangles
      at the ends of each row):
          0   1   2   3   4   5
         11  10   9   8   7   6
         12  13  14  15  16  17
 */
    pub fn new( area_size: UVec2, pos_scale: Vec2, uv_scale: Vec2 ) -> MeshOfSquares {
        // indices use u16, i.e. area cannot have more than 16k vertices
        assert!( area_size.x * area_size.y < 16384 );
        let space = (area_size.x * (area_size.y + 1)) as usize;

        let mut locations = Vec::with_capacity(space);
        let mut normals = Vec::with_capacity(space);
        let mut uvs = Vec::with_capacity(space);
        let mut colors = Vec::with_capacity(space);

        (0..area_size.y).cartesian_product(0..area_size.x)
            .for_each(|(y,x)| {
                // texture coordinates
                let uv = Vec2{x: x as f32, y: y as f32 } * uv_scale;
                let pos = Vec3{x: x as f32 * pos_scale.x, y: 0.0, z: y as f32 * pos_scale.y };
                locations.push(pos);
                normals.push(Vec3{x: 0.0, y: 1.0, z: 0.0});
                uvs.push(uv);
                colors.push(Vec4::new(uv.x, uv.y, 0.8, 0.3) );
            });

        // generate index list

        let triangle_count = (area_size.x-1)*(area_size.y-1)*2 + 2;
        let mut indices = Vec::with_capacity( triangle_count as usize);
        println!("Even number of rows: {:?}, space:{}|{}", area_size, space, locations.len());
        (0..area_size.y-1).cartesian_product(0..area_size.x)
            .for_each(|(y,x)| {
                let top_offset = y * area_size.x;
                let bot_offset = top_offset + area_size.x;

                let direction = y % 2;  // 0 forward; 1 backward

                // x_idx moves forward or backward depending on direction
                let x_idx = (1-direction) * x + direction * (area_size.x - 1 - x);

                // triangle definition ordering matters for which face is
                if direction == 0 { // even numbered rows
                    indices.push( (top_offset + x_idx) as u16 );
                    indices.push( (bot_offset + x_idx) as u16 );
                } else { // odd numbered rows
                    indices.push( (bot_offset + x_idx) as u16 );
                    indices.push( (top_offset + x_idx) as u16 );
                }
            });
        MeshOfSquares {
            indices,
            normals,
            locations,
            uvs,
            colors,
        }
    }

    /// randomize the y-coordinate of the mesh surface
    #[allow(dead_code)]
    pub fn randomize_position(mut self, range: (f32, f32)) -> MeshOfSquares {
        let granularity = 200.;
        let mut rng = rand::thread_rng();
        for vertex in self.locations.iter_mut() {
            let new_y = rng.gen_range(range.0 * granularity..range.1 * granularity) / (2.*granularity);
            vertex.y = new_y;
        }
        self
    }

    /// randomize the y-coordinate of the mesh surface
    #[allow(dead_code)]
    pub fn randomize_normals(mut self, noise_level: f32) -> MeshOfSquares {
        let granularity = 200.;
        let mut rng = rand::thread_rng();
        for normal in self.normals.iter_mut() {
            normal.x += rng.gen_range( -granularity/2.0 .. granularity/2.0 ) * (noise_level / 2.0);
            normal.y += rng.gen_range( -granularity/2.0 .. granularity/2.0 ) * (noise_level / 2.0);
            normal.z += rng.gen_range( -granularity/2.0 .. granularity/2.0 ) * (noise_level / 2.0);
        }
        self
    }

    #[allow(dead_code)]
    pub fn into_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::default());
        mesh.insert_indices(Indices::U16(self.indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.locations);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors );
        mesh
    }
}
