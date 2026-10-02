use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass}; // 1. Імпортуємо EguiPrimaryContextPass
use crate::interaction::{BrushMode, TerrainBrush};

pub struct EditorUiPlugin;

impl Plugin for EditorUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        
        // 2. ГОЛОВНА ЗМІНА: додаємо систему у спеціальний розклад EguiPrimaryContextPass
        app.add_systems(EguiPrimaryContextPass, draw_ui);
    }
}

fn draw_ui(
    mut contexts: EguiContexts,
    mut brush: ResMut<TerrainBrush>,
) {
    // 3. Безпечно отримуємо контекст для bevy_egui 0.42
    let ctx = match contexts.ctx_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    let window_response = egui::Window::new("Terrain Editor").show(ctx, |ui| {
        ui.heading("Brush Settings");

        ui.horizontal(|ui| {
            ui.radio_value(&mut brush.mode, BrushMode::Paint, "Paint");
            ui.radio_value(&mut brush.mode, BrushMode::Raise, "Raise");
            ui.radio_value(&mut brush.mode, BrushMode::Lower, "Lower");
        });

        ui.add(egui::Slider::new(&mut brush.radius, 5.0..=300.0).text("Radius"));

        if brush.mode == BrushMode::Paint {
            ui.separator();
            ui.label("Material Layer:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut brush.tile_id, 0, "Grass");
                ui.radio_value(&mut brush.tile_id, 1, "Dirt");
                ui.radio_value(&mut brush.tile_id, 2, "Sand");
                ui.radio_value(&mut brush.tile_id, 3, "Rock");
            });
        }
    });

    if let Some(res) = window_response {
        brush.over_ui = res.response.hovered() || res.response.dragged();
    } else {
        brush.over_ui = false;
    }
}