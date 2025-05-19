use egui::{Color32, Id, Pos2, Sense, Vec2};
use image::GenericImageView;

use crate::{components::*, structs::LivechartAppData};

impl LivechartAppData {
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

impl eframe::App for LivechartAppData {
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
                if !self.pixels_coords.is_empty() {
                    // TODO? Avoid cloning to put newest on top
                    let mut display_list = self.pixels_coords.clone();
                    display_list.reverse();
                    for point in display_list {
                        // let label =
                        //     ui.label(format!("Selected point: ({:.1}, {:.1})", point.x, point.y));
                        let response = ui.add_sized(
                            [ui.available_width(), 30.0],
                            egui::Label::new(format!("x: {} y: {}", point.x, point.y)), // .sense(Sense::hover()),console.log();
                        );

                        // Draw highlight if hovered
                        if response.hovered() {
                            let rect = response.rect;
                            let painter = ui.painter();
                            painter.rect_filled(rect, 0.0, Color32::DARK_GRAY);
                            // Redraw text over the highlight
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{:?}", point),
                                egui::TextStyle::Body.resolve(ui.style()),
                                Color32::WHITE,
                            );
                        }

                        // Draw a separator after each item except the last
                        if self.pixels_coords.iter().peekable().peek().is_some() {
                            ui.add(egui::Separator::default());
                        }
                    }
                } else {
                    ui.label("Click on the image to select a point.");
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Load image dimensions
            let image_size = image::open("./test_chart.png").unwrap().dimensions();
            let image_size_vec = Vec2::new(image_size.0 as f32, image_size.1 as f32);

            // Initialize or update zoom state
            let zoom_state = self.zoom_state.get_or_insert(crate::structs::ZoomState {
                scale: 1.0,
                offset: Vec2::ZERO,
            });

            // Handle zoom input (mouse wheel and pinch gestures)
            let zoom_response = ui.input(|i| {
                let mut zoom_delta = i.zoom_delta() - 1.0; // -1.0 to +1.0 range
                if i.pointer.primary_down() {
                    zoom_delta = 0.0; // Disable zoom while panning
                }
                zoom_delta
            });

            // Handle pan input
            let pan_response = ui.input(|i| {
                if i.pointer.primary_down() {
                    i.pointer.delta() * Vec2::new(1.0, 1.0)
                } else {
                    Vec2::ZERO
                }
            });

            // Update zoom and pan
            if zoom_response != 0.0 {
                let zoom_factor = 1.0 + zoom_response * 0.1; // Adjust zoom sensitivity
                zoom_state.scale *= zoom_factor;
                zoom_state.scale = zoom_state.scale.clamp(0.1, 10.0); // Limit zoom range
            }

            if pan_response != Vec2::ZERO {
                zoom_state.offset += pan_response;
            }

            // Calculate available space with padding
            let available_rect = ui.available_rect_before_wrap();
            let padding = 20.0;
            let max_size = available_rect.size() - Vec2::splat(padding * 2.0);

            // Calculate base scale (without zoom)
            let base_scale = (max_size.x / image_size_vec.x).min(max_size.y / image_size_vec.y);
            let total_scale = base_scale * zoom_state.scale;
            let scaled_size = image_size_vec * total_scale;

            // Apply pan offset
            let mut image_rect = egui::Rect::from_center_size(available_rect.center(), scaled_size);
            image_rect.min += zoom_state.offset;
            image_rect.max += zoom_state.offset;

            // Display the image
            let imageresponse = ui.put(
                image_rect,
                egui::Image::new(egui::include_image!("../test_chart.png"))
                    .sense(egui::Sense::drag().union(egui::Sense::click())),
            );

            // Draw the selected points
            for point in &self.pixels_coords {
                let image_pos = Pos2::new(
                    (point.x / image_size.0 as f32) * imageresponse.rect.width(),
                    (point.y / image_size.1 as f32) * imageresponse.rect.height(),
                ) + imageresponse.rect.min.to_vec2();

                let painter = ui.painter_at(imageresponse.rect);
                painter.circle_filled(image_pos, 4.0 / zoom_state.scale, Color32::BLUE);
            }

            paint_red_crosshair(ui, &imageresponse);

            // Handle point selection
            if let Some(pos) = imageresponse.interact_pointer_pos() {
                if imageresponse.clicked() {
                    let offset = pos - imageresponse.rect.min;
                    self.pixels_coords.push(
                        Pos2::new(
                            (offset.x / imageresponse.rect.width() * image_size.0 as f32)
                                .clamp(0.0, image_size.0 as f32),
                            (offset.y / imageresponse.rect.height() * image_size.1 as f32)
                                .clamp(0.0, image_size.1 as f32),
                        )
                        .into(),
                    );
                }
            }

            let mut cursor_icon = egui::CursorIcon::Default;

            // Change cursor when dragging
            if imageresponse.dragged() {
                cursor_icon = egui::CursorIcon::Move;
            } else if imageresponse.hovered() {
                cursor_icon = egui::CursorIcon::Default;
            }

            // Set the cursor icon
            ctx.set_cursor_icon(cursor_icon);

            egui::Area::new(Id::new("resetview"))
                .fixed_pos(egui::pos2(10.0, ctx.screen_rect().bottom() - 40.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    if ui.button("Reset View").clicked() {
                        self.zoom_state = None;
                    }
                });
        });
    }
}
