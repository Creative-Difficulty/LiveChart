use egui::{Response, ThemePreference};

// Paint red line:
pub fn paint_red_crosshair(ui: &mut egui::Ui, imagething: &Response) {
    if imagething.hovered() && imagething.hover_pos().is_some() {
        let pos = imagething.hover_pos();
        let painter = ui.painter_at(imagething.rect);

        painter.line_segment(
            [
                egui::pos2(pos.unwrap().x, imagething.rect.top()),
                egui::pos2(pos.unwrap().x, imagething.rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::RED),
        );

        painter.line_segment(
            [
                egui::pos2(imagething.rect.left(), pos.unwrap().y),
                egui::pos2(imagething.rect.right(), pos.unwrap().y),
            ],
            egui::Stroke::new(1.0, egui::Color32::RED),
        );
    }
}

pub fn custom_theme_switch(ui: &mut egui::Ui) {
    if std::convert::Into::<ThemePreference>::into(ui.ctx().theme()) == ThemePreference::Dark {
        if ui
            .add(egui::Button::new("☀").frame(true))
            .on_hover_text("Switch to light mode")
            .clicked()
        {
            ui.ctx().set_theme(ThemePreference::Light);
        }
    } else if ui
        .add(egui::Button::new("🌙").frame(true))
        .on_hover_text("Switch to dark mode")
        .clicked()
    {
        ui.ctx().set_theme(ThemePreference::Dark);
    }
}
