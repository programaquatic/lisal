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
    mesh::VertexAttributeValues, prelude::*, reflect::TypePath,
    render::render_resource::AsBindGroup,
};

use crate::{aqs_utils::mesh_of_squares::MeshOfSquares, tech::tank, water::grid::*};

use super::resources::{FluidParticleVelocity, FluidQuantityMass};

#[derive(Component)]
pub struct WaveGridCellTag(pub Handle<Mesh>);

#[derive(Component)]
pub struct WaveGridFrameTag;

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    // fn fragment_shader() -> ShaderRef {
    //     // "shaders/surface_vertex_shader.wgsl".into()
    //     "shaders/custom_material.wgsl".into()
    // }

    // fn vertex_shader() -> ShaderRef {
    //     "shaders/surface_vertex_shader.wgsl".into()
    // }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(2)]
    #[sampler(3)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
}

pub fn init_water_surface_system(
    grid: Res<Grid>,
    tank_cfg: Res<tank::Tank>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let offset = Vec3::Y * 2.0 * 2.0;

    // create a non-visible parent frame for offsetting and proper scaling of the surface
    let wavegrid_frame = commands
        .spawn((
            Name::new("Particle_Frame"),
            WaveGridFrameTag,
            Transform::from_translation(-offset)
                .with_scale(tank_cfg.get_size() / (grid.grid_size().as_vec3() * 2. - 3.)),
            Visibility::default(),
        ))
        .id();

    let sgrid_size = UVec2 {
        x: grid.grid_size().x,
        y: grid.grid_size().z,
    } * 2
        - 2;
    let sgrid_scale = Vec2::splat(1.0);
    let sgrid_uv_scale = Vec2::new(1. / sgrid_size.x as f32, 1. / sgrid_size.y as f32);
    // let sgrid_uv_scale = Vec2::splat(1.0);
    let surface_mesh = MeshOfSquares::new(sgrid_size, sgrid_scale, sgrid_uv_scale).into_mesh();
    let smesh_hdl = meshes.add(surface_mesh);

    let mt_hdl = materials.add(StandardMaterial {
        // color: Color::rgba(0.0, 0.0, 0.2, 0.5),
        // color_texture: Some(asset_server.load("textures/wgenerated.png")),
        alpha_mode: AlphaMode::Blend,
        reflectance: 1.0,
        metallic: 0.4,
        double_sided: true,
        // height: 0.0,
        ..default()
    });

    let surface_plane = commands
        .spawn((
            Mesh3d(smesh_hdl.clone()),
            MeshMaterial3d(mt_hdl),
            Transform::from_translation(Vec3::ZERO),
            WaveGridCellTag(smesh_hdl),
        ))
        .id();
    commands.entity(wavegrid_frame).add_child(surface_plane);
}

pub fn update_surface(
    grid: Res<Grid>,
    cells: Query<
        (
            Entity,
            &FluidQuantityMass,
            &FluidParticleVelocity,
            &GridCellIndex,
        ),
        With<GridCellType>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Query<&WaveGridCellTag>,
    mut surface_frames: Query<&mut Transform, With<WaveGridFrameTag>>,
) {
    fn calculate_surface_updates(
        x: f32,
        y: f32,
        z: f32,
        grid: &Res<Grid>,
        velo: &[Vec3],
        mass: &[f32],
    ) -> [f32; 5] {
        let cell_ivec = IVec3::new(x as i32, y as i32, z as i32);
        let cell_idx = cell_ivec / 2 + 1;
        let cell_neighbor = (cell_ivec % 2) - 1;
        let mut avg_velocity = Vec3::ZERO; // (local_cell.velocity.y + local_cell.mass) * 0.2; //0.075;
        for iz in 0..2 {
            for ix in 0..2 {
                let weight = f32::powi(
                    2.0,
                    2 - (i32::abs(ix + cell_neighbor.x) + i32::abs(iz + cell_neighbor.z)),
                );
                // println!("   --> {}, {}, {}", (x + cell_neighbor.x), (z + cell_neighbor.z), weight);
                let neighbor_cell_idx = grid.index_of(
                    (cell_idx.x + (ix + cell_neighbor.x)) as usize,
                    (grid.get_surface_level()) as usize,
                    // 1,
                    (cell_idx.z + (iz + cell_neighbor.z)) as usize,
                );
                avg_velocity +=
                    (velo[neighbor_cell_idx] + mass[neighbor_cell_idx]) * 0.075 * weight;
            }
        }
        avg_velocity /= 9.0;
        [
            x,
            0.75 * avg_velocity.y,
            z,
            avg_velocity.x * 0.2,
            avg_velocity.z * 0.2,
        ]
    }

    let mut cell_velo = vec![Vec3::ZERO; cells.iter().len()];
    let mut cell_mass = vec![0.0; cells.iter().len()];
    cells.iter().for_each(|(_, mass, vel, idx)| {
        cell_velo[idx.0] = vel.0.into();
        cell_mass[idx.0] = mass.0;
    });

    surface_frames.par_iter_mut().for_each(|mut transform| {
        transform.translation.y = grid.to_world_coord(Vec3::splat(grid.get_surface_level())).y;
    });
    // technically, we should only have one mesh that matches the query
    let mesh_hdl = mesh_handles.single().unwrap();

    if let Some(mesh) = meshes.get_mut(&mesh_hdl.0) {
        if let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            let updates: Vec<[f32; 5]> = positions
                .iter()
                .map(|[x, y, z]| {
                    calculate_surface_updates(*x, *y, *z, &grid, &cell_velo, &cell_mass)
                })
                .collect();

            let heights: Vec<[f32; 3]> =
                updates.iter().map(|[x, y, z, _, _]| [*x, *y, *z]).collect();
            let colors: Vec<[f32; 4]> = updates
                .iter()
                .map(|[_, g, _, _, _]| {
                    [
                        0.0, //*g,
                        *g, 0.8, 0.7,
                    ]
                })
                .collect();
            let normals: Vec<[f32; 3]> = updates
                .iter()
                .map(|[_, _, _, x, z]| [*x, 1.0, *z])
                .collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, heights);
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_surface_indices() {
        let size = UVec2::new(5, 4);
        let loc_scale = Vec2::splat(20.);
        let uv_scale = Vec2::splat(3.);

        let meshy = MeshOfSquares::new(size, loc_scale, uv_scale).into_mesh();
        dbg!(meshy.clone());
        assert!(meshy.primitive_topology().is_strip());
    }
}
