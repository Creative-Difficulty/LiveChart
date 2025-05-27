use egui::{Id, Response, ThemePreference, Ui, Vec2};

use crate::app::LivechartApp;
use crate::structs::{CoordinatePair, PixelCoordinate, ViewState};

impl LivechartApp {
    // Paint red line:
    pub fn paint_crosshair(
        &self,
        ui: &egui::Ui,
        imagething: &Response, /*ctx: &egui::Context*/
    ) {
        if let Some(pos) = imagething.hover_pos() {
            let painter = ui.painter_at(imagething.rect);

            let stroke = egui::Stroke::new(1.5, egui::Color32::BLACK);

            painter.line_segment(
                [
                    egui::pos2(pos.x, imagething.rect.top()),
                    egui::pos2(pos.x, imagething.rect.bottom()),
                ],
                stroke,
            );

            painter.line_segment(
                [
                    egui::pos2(imagething.rect.left(), pos.y),
                    egui::pos2(imagething.rect.right(), pos.y),
                ],
                stroke,
            );
        }
    }

    pub fn custom_theme_switch(&self, ui: &mut egui::Ui) {
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

    pub fn handle_zoom_input(&mut self, ui: &egui::Ui, image_response: &egui::Response) {
        let viewstate = self.data.view_state.get_or_insert(ViewState::default());
        let zoom_delta = ui.input(|i| {
            let mut delta = i.zoom_delta() - 1.0;
            // Disable zoom while dragging image
            if image_response.dragged() {
                delta = 0.0;
            }

            delta
        });

        if zoom_delta != 0.0 {
            let zoom_factor = 1.0 + zoom_delta * 0.1;
            viewstate.scale *= zoom_factor;
            viewstate.scale = viewstate.scale.clamp(0.1, 10.0);
        }
    }

    pub fn handle_pan_input(&mut self, ui: &egui::Ui, image_response: &egui::Response) {
        let viewstate = self.data.view_state.get_or_insert(ViewState::default());

        let pan_delta = ui.input(|i| {
            if image_response.dragged() {
                i.pointer.delta()
            } else {
                Vec2::ZERO
            }
        });

        if pan_delta != Vec2::ZERO {
            viewstate.offset += pan_delta;
        }
    }

    pub fn display_zoom_pan(&mut self, ui: &egui::Ui, image_size: Vec2) -> egui::Rect {
        let view_state = self.data.view_state.get_or_insert(ViewState::default());

        // Available space in the UI (including sidebars (???) etc.)
        let available_rect = ui.max_rect();
        let padding = 10.0;
        let max_size = available_rect.size() - Vec2::splat(padding * 2.0);

        // Calculate the base scale to fully fit the image with some padding
        let base_scale = (max_size.x / image_size.x).min(max_size.y / image_size.y);
        // Start fully zoomed in (as large as possible while fully fitting)
        let total_scale = base_scale * view_state.scale;
        let scaled_size = image_size * total_scale;

        // Calculate the image rectangle centered in the available space
        let center = available_rect.center();
        let mut image_rect = egui::Rect::from_center_size(center, scaled_size);

        // Allow extra panning beyond the edges of the image
        let extra_pan_margin = 100.0; // Amount of extra space allowed for panning
        let max_offset_x =
            ((scaled_size.x - available_rect.width()) / 2.0 + extra_pan_margin).max(0.0);
        let max_offset_y =
            ((scaled_size.y - available_rect.height()) / 2.0 + extra_pan_margin).max(0.0);

        // Clamp the user's panning offset
        view_state.offset.x = view_state.offset.x.clamp(-max_offset_x, max_offset_x);
        view_state.offset.y = view_state.offset.y.clamp(-max_offset_y, max_offset_y);

        // Apply offset to the image rectangle
        image_rect.min += view_state.offset;
        image_rect.max += view_state.offset;

        image_rect
    }

    pub fn display_image(&self, ui: &mut egui::Ui, rect: egui::Rect) -> egui::Response {
        ui.put(
            rect,
            egui::Image::new(egui::include_image!("../test_chart_vertical_ils_plate.png"))
                .sense(egui::Sense::drag().union(egui::Sense::click())),
        )
    }

    pub fn draw_pixel_coordinates(
        &self,
        point: &PixelCoordinate,
        ui: &egui::Ui,
        image_response: &egui::Response,
        image_size: (u32, u32),
    ) {
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
                .circle_stroke(image_pos, dot_radius * 1.5, (2.0, egui::Color32::RED));
        }
    }
    // TODO: review AI slop below
    pub fn add_point(
        &self,
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

                if !self
                    .data
                    .points
                    .iter()
                    .any(|coordpair| coordpair.pixels == PixelCoordinate { x, y })
                {
                    return Some(PixelCoordinate { x, y });
                } else {
                    return None;
                }
            }
        }
        None
    }

    pub fn reset_view_button(&mut self, ctx: &egui::Context) {
        egui::Area::new(Id::new("resetview"))
            .fixed_pos(egui::pos2(10.0, ctx.screen_rect().bottom() - 40.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if ui.button("Reset View").clicked() {
                    self.data.view_state = None;
                }
            });
    }

    pub fn hide_point_selection_sidebar_button(&mut self, ctx: &egui::Context, ui: &Ui) {
        egui::Area::new(Id::new("hide_ps_sidebar"))
            .fixed_pos(egui::pos2(
                //TODO fix layouting
                ui.max_rect().right(),
                ui.max_rect().top(),
            ))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if ui
                    .button(format!(
                        "{} sidebar",
                        if self
                            .data
                            .view_state
                            .get_or_insert(ViewState::default())
                            .ps_sidebar_shown
                        {
                            "Hide"
                        } else {
                            "Show"
                        },
                    ))
                    .clicked()
                {
                    self.data
                        .view_state
                        .get_or_insert(ViewState::default())
                        .ps_sidebar_shown = !self
                        .data
                        .view_state
                        .get_or_insert(ViewState::default())
                        .ps_sidebar_shown;
                }
            });
    }
    pub fn label_with_delete_button_for_single_point(
        &mut self,
        ui: &mut egui::Ui,
        point: &CoordinatePair,
    ) {
        let label = ui.label(format!(
            "Selected point: ({}, {})",
            point.pixels.x.round(),
            point.pixels.y.round()
        ));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                if let Some(index_to_delete) = self
                    .data
                    .points
                    .iter()
                    .position(|p| p.pixels == point.pixels)
                {
                    self.data.points.remove(index_to_delete);
                }
            }
        });
    }

    pub fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("sidebar")
            .default_width(ctx.screen_rect().width() * 0.2) // initial sidebar width
            .resizable(false)
            .show(ctx, |ui: &mut egui::Ui| {
                egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Points");

                    ui.set_width(ctx.screen_rect().width() * 0.2);

                    egui::containers::scroll_area::ScrollArea::vertical().show(ui, |ui| {
                        if !self.data.points.is_empty() {
                            // self.data.points.reverse();

                            // GOODEXAMPLE: Treating the last element of an iterator different than the rest.
                            //  while let Some(point) = display_iter.next() {
                            //     label_and_delete_button(ui, point, &mut self.data.pixel_coords);
                            //     if display_iter.peek().is_some() {
                            //         ui.separator();
                            //     }
                            // }

                            //TODO: why do i have to clone here?
                            for point in self.data.points.clone().iter().rev() {
                                ui.horizontal_wrapped(|ui| {
                                    self.label_with_delete_button_for_single_point(ui, point);
                                });
                                ui.separator();
                            }
                        } else {
                            ui.label("Click on the chart to select a point.");
                        }
                    });
                })
            });
    }
}
