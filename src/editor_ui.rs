use crate::interaction::{self, BrushMode, TerrainBrush};
use crate::terrain::map_io;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle, egui};

/// Configuration for the terrain texture atlas used by the editor palette.
///
/// The resource is inserted during asset initialization once the atlas handle
/// is available.
#[derive(Resource)]
pub struct TerrainAtlasConfig {
    pub handle: Handle<Image>,
    pub cols: usize,
    pub rows: usize,
}

impl TerrainAtlasConfig {
    pub const DEFAULT_COLS: usize = 4;
    pub const DEFAULT_ROWS: usize = 37;
}

impl Default for TerrainAtlasConfig {
    fn default() -> Self {
        Self {
            handle: Handle::default(),
            cols: Self::DEFAULT_COLS,
            rows: Self::DEFAULT_ROWS,
        }
    }
}

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainAtlasConfig>();
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.add_systems(Startup, load_ui_atlas_handle)
            .add_systems(EguiPrimaryContextPass, draw_ui);
    }
}

fn load_ui_atlas_handle(
    mut atlas_config: ResMut<TerrainAtlasConfig>,
    asset_server: Res<AssetServer>,
) {
    atlas_config.handle = asset_server.load("gsc://TILES3.BMP");
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut brush: ResMut<TerrainBrush>,
    atlas_config: Res<TerrainAtlasConfig>,
    mut io_state: ResMut<map_io::MapIoState>,
    mut save_events: MessageWriter<map_io::SaveMapRequest>,
    mut load_events: MessageWriter<map_io::LoadMapRequest>,
) {
    let texture_id = contexts.add_image(EguiTextureHandle::Strong(atlas_config.handle.clone()));
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    let window_response = egui::Window::new("Terrain Editor").show(ctx, |ui| {
        ui.heading("Brush Settings");

        ui.horizontal(|ui| {
            ui.radio_value(&mut brush.mode, interaction::BrushMode::None, "Select");
            ui.radio_value(&mut brush.mode, BrushMode::Paint, "Paint");
            ui.radio_value(&mut brush.mode, BrushMode::Raise, "Raise");
            ui.radio_value(&mut brush.mode, BrushMode::Lower, "Lower");
            ui.radio_value(&mut brush.mode, BrushMode::Smooth, "Smooth");
            ui.radio_value(&mut brush.mode, BrushMode::Flatten, "Flatten");
        });

        ui.add(egui::Slider::new(&mut brush.radius, 5.0..=300.0).text("Radius"));

        if brush.mode == BrushMode::Paint {
            ui.separator();
            ui.label("Terrain Texture:");

            let tile_size = egui::vec2(32.0, 32.0);
            let cols = atlas_config.cols.max(1);
            let rows = atlas_config.rows.max(1);
            let tile_count = cols.saturating_mul(rows);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    egui::Grid::new("terrain_texture_palette")
                        .num_columns(4)
                        .spacing(egui::vec2(4.0, 4.0))
                        .show(ui, |ui| {
                            for tile_id in 0..tile_count {
                                let column = tile_id % cols;
                                let row = tile_id / cols;
                                let min_uv = egui::pos2(
                                    column as f32 / cols as f32,
                                    row as f32 / rows as f32,
                                );
                                let max_uv = egui::pos2(
                                    (column + 1) as f32 / cols as f32,
                                    (row + 1) as f32 / rows as f32,
                                );
                                let image = egui::Image::new((texture_id, tile_size))
                                    .uv(egui::Rect::from_min_max(min_uv, max_uv));
                                let selected = brush.tile_id == tile_id;

                                if ui
                                    .add(egui::Button::image(image).selected(selected))
                                    .clicked()
                                {
                                    brush.tile_id = tile_id;
                                }

                                if (tile_id + 1) % 4 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Map File:");
            ui.text_edit_singleline(&mut io_state.filename);
        });
        ui.horizontal(|ui| {
            if ui.button("Save Map").clicked() {
                save_events.write(map_io::SaveMapRequest(io_state.filename.clone()));
                io_state.status = "Saving...".to_string();
            }
            if ui.button("Load Map").clicked() {
                load_events.write(map_io::LoadMapRequest(io_state.filename.clone()));
                io_state.status = "Loading...".to_string();
            }
        });
        ui.label(egui::RichText::new(&io_state.status).color(egui::Color32::YELLOW));
    });

    if let Some(res) = window_response {
        brush.over_ui = res.response.hovered() || res.response.dragged();
    } else {
        brush.over_ui = false;
    }
}
