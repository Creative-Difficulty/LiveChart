use egui::Vec2;
use image::GenericImageView;

use crate::structs::{CoordinatePair, LiveChartAppData};

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
                self.custom_theme_switch(ui);

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
        if let Some(viewstate) = &self.data.view_state {
            if viewstate.ps_sidebar_shown {
                self.sidebar(ctx);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Load image dimensions once (could be cached if needed)
            let image_size = image::open("./test_chart_vertical_ils_plate.png")
                .unwrap()
                .dimensions();
            let image_size_vec = Vec2::new(image_size.0 as f32, image_size.1 as f32);

            // Initialize or update zoom state
            // let zoom_state = self.data.view_state.get_or_insert(ZoomState::default());

            // Calculate display parameters
            let display_params = self.display_zoom_pan(ui, image_size_vec);

            // Display the image and get the response
            let image_response = self.display_image(ui, display_params);

            if image_response.contains_pointer() {
                // Handle user input
                self.handle_pan_input(ui, &image_response);
                self.handle_zoom_input(ui, &image_response);
            }

            // Handle point creation
            if let Some(coord) = self.add_point(&image_response, image_size) {
                self.data.points.push(CoordinatePair {
                    pixels: coord,
                    real: None,
                });
            }

            // Draw all the points on the image
            for point in &self.data.points {
                self.draw_pixel_coordinates(&point.pixels, ui, &image_response, image_size);
            }

            // Draw crosshair
            self.paint_crosshair(ui, &image_response);
            ui.response()
                .on_hover_and_drag_cursor(egui::CursorIcon::Move);

            // Show reset view button
            self.reset_view_button(ctx);
            self.hide_point_selection_sidebar_button(ctx, ui);
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
