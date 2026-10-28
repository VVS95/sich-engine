use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use serde::{Deserialize, Serialize};

use super::grid::TerrainGrid;
use super::splatmap::TerrainSplatmap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapSpawnPoint {
    pub spawn_type: u32,
    pub x: f32,
    pub z: f32,
    pub player: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapDataV2 {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub heights: Vec<i32>,
    pub splat_resolution: [u32; 2],
    pub splat_indices: Vec<u8>,
    pub splat_weights: Vec<u8>,
    pub passability_overrides: Vec<u8>,
    pub spawns: Vec<MapSpawnPoint>,
}

pub const MAP_MAGIC: [u8; 4] = *b"SICH";
pub const CURRENT_MAP_VERSION: u32 = 2;

fn calculate_checksum(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0u64, |acc, &x| acc.wrapping_add(x as u64))
}

pub fn save_map_to_file(path: &Path, map_data: &MapDataV2) -> std::io::Result<()> {
    let payload = bincode::serialize(map_data)
        .map_err(|error| std::io::Error::other(format!("failed to serialize map: {error}")))?;
    let checksum = calculate_checksum(&payload);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(path)?;
    file.write_all(&MAP_MAGIC)?;
    file.write_all(&CURRENT_MAP_VERSION.to_le_bytes())?;
    file.write_all(&checksum.to_le_bytes())?;
    file.write_all(&payload)?;

    Ok(())
}

pub fn load_map_from_file(path: &Path) -> Result<MapDataV2, String> {
    let mut file = File::open(path).map_err(|error| format!("failed to open map file: {error}"))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read map file: {error}"))?;

    if bytes.len() < 16 {
        return Err("map file is too small to contain a valid header".to_string());
    }

    if bytes[0..4] != MAP_MAGIC {
        return Err("invalid map magic".to_string());
    }

    let version = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .expect("map header version must be exactly 4 bytes"),
    );
    if version != CURRENT_MAP_VERSION {
        return Err(format!("unsupported map version: {version}"));
    }

    let expected_checksum = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .expect("map header checksum must be exactly 8 bytes"),
    );
    let payload = &bytes[16..];
    if calculate_checksum(payload) != expected_checksum {
        return Err("map file checksum mismatch".to_string());
    }

    bincode::deserialize(payload).map_err(|error| format!("failed to deserialize map: {error}"))
}

#[derive(Event, Message)]
pub struct SaveMapRequest(pub String);

#[derive(Event, Message)]
pub struct LoadMapRequest(pub String);

#[derive(Resource)]
pub struct MapIoState {
    pub filename: String,
    pub status: String,
}

impl Default for MapIoState {
    fn default() -> Self {
        Self {
            filename: "test_map.sich".to_string(),
            status: String::new(),
        }
    }
}

pub fn save_map_system(
    mut requests: MessageReader<SaveMapRequest>,
    grid_query: Query<&TerrainGrid>,
    splatmap: Res<TerrainSplatmap>,
    images: Res<Assets<Image>>,
    mut io_state: ResMut<MapIoState>,
) {
    for event in requests.read() {
        let path = Path::new("assets/maps").join(&event.0);
        let Some(grid) = grid_query.iter().next() else {
            let error = "no terrain grid found";
            error!("Failed to save map {:?}: {error}", path);
            io_state.status = format!("Save Error: {error}");
            continue;
        };
        let Some(indices_image) = images.get(&splatmap.indices) else {
            let error = "splat indices image not found";
            error!("Failed to save map {:?}: {error}", path);
            io_state.status = format!("Save Error: {error}");
            continue;
        };
        let Some(weights_image) = images.get(&splatmap.weights) else {
            let error = "splat weights image not found";
            error!("Failed to save map {:?}: {error}", path);
            io_state.status = format!("Save Error: {error}");
            continue;
        };
        let Some(splat_indices) = indices_image.data.as_ref() else {
            let error = "splat indices image has no data";
            error!("Failed to save map {:?}: {error}", path);
            io_state.status = format!("Save Error: {error}");
            continue;
        };
        let Some(splat_weights) = weights_image.data.as_ref() else {
            let error = "splat weights image has no data";
            error!("Failed to save map {:?}: {error}", path);
            io_state.status = format!("Save Error: {error}");
            continue;
        };

        let map_data = MapDataV2 {
            width: grid.width as u32,
            height: grid.height as u32,
            tile_size: grid.tile_size,
            heights: grid.heights.clone(),
            splat_resolution: [grid.width as u32, grid.height as u32],
            splat_indices: splat_indices.clone(),
            splat_weights: splat_weights.clone(),
            passability_overrides: Vec::new(),
            spawns: Vec::new(),
        };

        match save_map_to_file(&path, &map_data) {
            Ok(()) => {
                info!("Saved map to {:?}", path);
                io_state.status = "Map saved successfully!".to_string();
            }
            Err(error) => {
                error!("Failed to save map {:?}: {error}", path);
                io_state.status = format!("Save Error: {error}");
            }
        }
    }
}

pub fn load_map_system(
    mut requests: MessageReader<LoadMapRequest>,
    mut terrain_query: Query<(&mut TerrainGrid, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    splatmap: Res<TerrainSplatmap>,
    mut io_state: ResMut<MapIoState>,
) {
    for event in requests.read() {
        let path = Path::new("assets/maps").join(&event.0);
        let map_data = match load_map_from_file(&path) {
            Ok(map_data) => map_data,
            Err(error) => {
                error!("Failed to load map {:?}: {error}", path);
                io_state.status = format!("Load Error: {error}");
                continue;
            }
        };

        let Some((mut grid, mesh3d)) = terrain_query.iter_mut().next() else {
            let error = "no terrain grid found";
            error!("Failed to load map {:?}: {error}", path);
            io_state.status = format!("Load Error: {error}");
            continue;
        };

        grid.width = map_data.width as usize;
        grid.height = map_data.height as usize;
        grid.tile_size = map_data.tile_size;
        grid.heights = map_data.heights;

        let extent = Extent3d {
            width: map_data.splat_resolution[0],
            height: map_data.splat_resolution[1],
            depth_or_array_layers: 1,
        };

        if let Some(mut image) = images.get_mut(&splatmap.indices) {
            image.data = Some(map_data.splat_indices.clone());
            image.texture_descriptor.size = extent;
        } else {
            let error = "splat indices image not found";
            error!("Failed to load map {:?}: {error}", path);
            io_state.status = format!("Load Error: {error}");
            continue;
        }

        if let Some(mut image) = images.get_mut(&splatmap.weights) {
            image.data = Some(map_data.splat_weights);
            image.texture_descriptor.size = extent;
        } else {
            let error = "splat weights image not found";
            error!("Failed to load map {:?}: {error}", path);
            io_state.status = format!("Load Error: {error}");
            continue;
        }

        let new_mesh = grid.generate_mesh();
        if let Some(mut existing_mesh) = meshes.get_mut(&mesh3d.0) {
            *existing_mesh = new_mesh;
        } else {
            let error = "terrain mesh asset not found";
            error!("Failed to load map {:?}: {error}", path);
            io_state.status = format!("Load Error: {error}");
            continue;
        }

        info!("Loaded map from {:?}", path);
        io_state.status = "Map loaded successfully!".to_string();
    }
}
