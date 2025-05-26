use egui::{Id, Response, ThemePreference, Vec2};

use crate::app::LivechartApp;
use crate::structs::{PixelCoordinate, ZoomState};

// Paint red line:
pub fn paint_crosshair(ui: &mut egui::Ui, imagething: &Response /*ctx: &egui::Context*/) {
    if let Some(pos) = imagething.hover_pos() {
        let painter = ui.painter_at(imagething.rect);
        // let crosshair_color = if ctx.theme() == egui::Theme::Dark {
        //     egui::Color32::BLACK
        // } else {
        //     egui::Color32::WHITE
        // };

        let crosshair_color = egui::Color32::BLACK;
        let crosshair_width = 1.5;

        painter.line_segment(
            [
                egui::pos2(pos.x, imagething.rect.top()),
                egui::pos2(pos.x, imagething.rect.bottom()),
            ],
            egui::Stroke::new(crosshair_width, crosshair_color),
        );

        painter.line_segment(
            [
                egui::pos2(imagething.rect.left(), pos.y),
                egui::pos2(imagething.rect.right(), pos.y),
            ],
            egui::Stroke::new(crosshair_width, crosshair_color),
        );
    }
}

pub fn custom_theme_switch(ui: &mut egui::Ui) {
    if std::convert::Into::<ThemePreference>::into(ui.ctx().theme()) == ThemePreference::Dark {
        if ui
            .add(egui::Button::new("☀").frame(false))
            .on_hover_text("Switch to light mode")
            .clicked()
        {
            ui.ctx().set_theme(ThemePreference::Light);
        }
    } else if ui
        .add(egui::Button::new("🌙").frame(false))
        .on_hover_text("Switch to dark mode")
        .clicked()
    {
        ui.ctx().set_theme(ThemePreference::Dark);
    }
}

pub fn handle_zoom_input(ui: &mut egui::Ui, zoom_state: &mut ZoomState) {
    let zoom_delta = ui.input(|i| {
        let mut delta = i.zoom_delta() - 1.0;
        if i.pointer.primary_down() {
            delta = 0.0; // Disable zoom while panning
        }
        delta
    });

    if zoom_delta != 0.0 {
        let zoom_factor = 1.0 + zoom_delta * 0.1;
        zoom_state.scale *= zoom_factor;
        zoom_state.scale = zoom_state.scale.clamp(0.1, 10.0);
    }
}

pub fn handle_pan_input(ui: &egui::Ui, zoom_state: &mut ZoomState) {
    let pan_delta = ui.input(|i| {
        if i.pointer.primary_down() {
            i.pointer.delta()
        } else {
            Vec2::ZERO
        }
    });

    if pan_delta != Vec2::ZERO {
        zoom_state.offset += pan_delta;
    }
}

pub fn calculate_display_parameters(
    ui: &egui::Ui,
    image_size: Vec2,
    zoom_state: &ZoomState,
) -> egui::Rect {
    let available_rect = ui.available_rect_before_wrap();
    let padding = 20.0;
    let max_size = available_rect.size() - Vec2::splat(padding * 2.0);

    let base_scale = (max_size.x / image_size.x).min(max_size.y / image_size.y);
    let total_scale = base_scale * zoom_state.scale;
    let scaled_size = image_size * total_scale;

    let mut image_rect = egui::Rect::from_center_size(available_rect.center(), scaled_size);
    image_rect.min += zoom_state.offset;
    image_rect.max += zoom_state.offset;

    image_rect
}

pub fn display_image(ui: &mut egui::Ui, rect: egui::Rect) -> egui::Response {
    ui.put(
        rect,
        egui::Image::new(egui::include_image!("../test_chart_vertical_ils_plate.png"))
            .sense(egui::Sense::drag().union(egui::Sense::click())),
    )
}

pub fn draw_pixel_coordinates(
    point: &PixelCoordinate,
    ui: &egui::Ui,
    image_response: &egui::Response,
    image_size: (u32, u32),
) {
    // for point in &appdata.data.pixel_coords {
    let norm_x = point.x / image_size.0 as f32;
    let norm_y = point.y / image_size.1 as f32;

    let image_pos = image_response.rect.min
        + egui::vec2(
            norm_x * image_response.rect.width(),
            norm_y * image_response.rect.height(),
        );

    let dot_radius = 4.0;
    ui.painter()
        .circle_filled(image_pos, dot_radius, egui::Color32::BLUE);

    if ui.rect_contains_pointer(egui::Rect::from_center_size(
        image_pos,
        egui::Vec2::splat(dot_radius * 3.0),
    )) {
        ui.painter()
            .circle_stroke(image_pos, dot_radius * 1.5, (1.5, egui::Color32::WHITE));
    }
    // }
}
// TODO: review AI slop below
pub fn add_point(
    image_response: &egui::Response,
    image_size: (u32, u32),
) -> Option<PixelCoordinate> {
    if let Some(pos) = image_response.interact_pointer_pos() {
        if image_response.clicked() {
            let offset = pos - image_response.rect.min;
            let x = (offset.x / image_response.rect.width() * image_size.0 as f32)
                .clamp(0.0, image_size.0 as f32);
            let y = (offset.y / image_response.rect.height() * image_size.1 as f32)
                .clamp(0.0, image_size.1 as f32);

            return Some(PixelCoordinate { x, y });
        }
    }
    None
}

pub fn update_cursor_icon(ctx: &egui::Context, image_response: &egui::Response) {
    let cursor_icon = if image_response.dragged() {
        egui::CursorIcon::Move
    // TODO: Implement this cursor hover logic so that it makes sense
    // } else if image_response.hovered() {
    //     egui::CursorIcon::Default
    } else {
        egui::CursorIcon::Default
    };

    ctx.set_cursor_icon(cursor_icon);
}

pub fn show_reset_button(appdata: &mut LivechartApp, ctx: &egui::Context) {
    egui::Area::new(Id::new("resetview"))
        .fixed_pos(egui::pos2(10.0, ctx.screen_rect().bottom() - 40.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            if ui.button("Reset View").clicked() {
                appdata.data.view_state = None;
            }
        });
}
