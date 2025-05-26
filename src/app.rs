use egui::Vec2;
use image::GenericImageView;

use crate::{
    components::*,
    structs::{CoordinatePair, LiveChartAppData, ZoomState},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct LivechartApp {
    pub data: LiveChartAppData,
}

//TODO Also clamp saved point positions to prevent overflow on image or dont to display that something with the placement went wrong

impl LivechartApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for LivechartApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                custom_theme_switch(ui);

                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
            });
        });

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        egui::SidePanel::right("sidebar").show(ctx, |ui: &mut egui::Ui| {
            ui.heading("Points");
            egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                if !self.data.points.is_empty() {
                    // TODO? Avoid cloning to put newest on top
                    let mut display_list = self.data.points.clone();
                    display_list.reverse();
                    let display_iter = display_list.iter().peekable();

                    // GOODEXAMPLE: Treating the last element of an iterator different than the rest.
                    //  while let Some(point) = display_iter.next() {
                    //     label_and_delete_button(ui, point, &mut self.data.pixel_coords);
                    //     if display_iter.peek().is_some() {
                    //         ui.separator();
                    //     }
                    // }

                    for point in display_iter {
                        label_with_delete_button(ui, point, &mut self.data.points);
                        ui.separator();
                    }
                } else {
                    ui.label("Click on the image to select a point.");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Load image dimensions once (could be cached if needed)
            let image_size = image::open("./test_chart_vertical_ils_plate.png")
                .unwrap()
                .dimensions();
            let image_size_vec = Vec2::new(image_size.0 as f32, image_size.1 as f32);

            // Initialize or update zoom state
            let zoom_state = self.data.view_state.get_or_insert(ZoomState::default());

            // Handle user input
            handle_zoom_input(ui, zoom_state);
            handle_pan_input(ui, zoom_state);

            // Calculate display parameters
            let display_params = calculate_display_parameters(ui, image_size_vec, zoom_state);

            // Display the image and get the response
            let image_response = display_image(ui, display_params);

            // Handle point selection
            if let Some(coord) = add_point(&image_response, image_size) {
                self.data.points.push(CoordinatePair {
                    pixels: coord,
                    real: None,
                });
            }

            // Draw all the points on the image
            for point in &self.data.points {
                draw_pixel_coordinates(&point.pixels, ui, &image_response, image_size);
            }

            // Draw crosshair
            paint_crosshair(ui, &image_response /*ctx*/);

            // Update cursor icon based on interaction
            update_cursor_icon(ctx, &image_response);

            // Show reset view button
            show_reset_button(self, ctx);
        });
    }
}

// impl MyApp {
//     // Increment the counter and update the label
//     fn increment_counter(&mut self) {
//         self.counter += 1;
//         self.update_label_text();
//     }

//     // Update the label to reflect the current counter value
//     fn update_label_text(&mut self) {
//         self.label_text = format!("Counter value: {}", self.counter);
//     }
// }

fn label_with_delete_button(
    ui: &mut egui::Ui,
    point: &CoordinatePair,
    pixel_coords: &mut Vec<CoordinatePair>,
) {
    let label = ui.label(format!(
        "Selected point: ({}, {})",
        point.pixels.x.round(),
        point.pixels.y.round()
    ));

    if ui
        .add_sized(
            Vec2 {
                x: ui.available_width() * 0.2,
                y: label.rect.height(),
            },
            egui::Button::new("Delete"),
        )
        .clicked()
    {
        //TODO: more efficient
        if let Some(index_to_delete) = pixel_coords.iter().position(|p| p == point) {
            pixel_coords.remove(index_to_delete);
        }
    }
}
