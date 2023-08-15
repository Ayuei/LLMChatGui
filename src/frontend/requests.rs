use super::gui::ChatGui;
use egui::Ui;

fn send_request() {}

impl ChatGui {
    fn requests_ui(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new("Request Details").strong());
        ui.horizontal(|ui| {
            ui.label("URL");
            ui.add(egui::TextEdit::singleline(&mut self.gui_settings.request_url)
                .desired_width(140.0));
            ui.label("Port");
            //ui.add(egui::TextEdit::singleline(&mut self.config.port)
            //    .desired_width(30.0));
        });
        ui.add_space(4.0);

        if ui.button("Send").clicked() {
        }
    }
}

