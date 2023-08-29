// TODO: Add sound?
use anyhow::Result;
use chrono::prelude::DateTime;
use chrono::Local;
use egui::{Color32, FontFamily, FontId, Align};
use epaint::text::LayoutJob;
use serde::{Deserialize, Serialize};

use super::panels::config::GuiConfig;

const USER_COLOUR: Color32 = Color32::DARK_GRAY;
const ASSISTANT_COLOR: Color32 = Color32::DARK_GREEN;

#[derive(PartialEq)]
enum View {
    Main,
    Config,
}

impl Default for View {
    fn default() -> Self {
        Self::Main
    }
}

pub fn get_current_time() -> String {
    let local: DateTime<Local> = Local::now();
    local.format("%H:%M:%S").to_string()
}

#[derive(Serialize, Deserialize)]
pub struct ScrollBuffer<T> {
    internal: Vec<LayoutJob>,
    flush: String,

    #[serde(skip)]
    rx: Option<flume::Receiver<T>>,
}

impl<T> ScrollBuffer<T> {
    fn new(rx: flume::Receiver<T>) -> Self {
        ScrollBuffer {
            internal: Vec::new(),
            flush: String::new(),
            rx: Some(rx),
        }
    }

    fn size(&self) -> usize {
        self.internal.len()
    }
}

pub(crate) fn convert_text_to_layout_job(
    prefix: &str,
    text: &str,
    background_color: egui::Color32,
) -> epaint::text::LayoutJob {
    let mut job = LayoutJob::default();
    let text_color = Color32::WHITE;

    job.append(
        format!("[{}]:  ", get_current_time()).as_str(),
        0.0,
        epaint::text::TextFormat {
            font_id: FontId::new(14.0, FontFamily::Proportional),
            color: text_color,
            ..Default::default()
        },
    );

    if prefix.len() > 0 {
        job.append(
            prefix,
            0.0,
            epaint::text::TextFormat {
                font_id: FontId::new(14.0, FontFamily::Proportional),
                color: text_color,
                background: background_color,
                ..Default::default()
            },
        );
    }

    job.append(
        text,
        8.5,
        epaint::text::TextFormat {
            font_id: FontId::new(14.0, FontFamily::Proportional),
            color: text_color,
            ..Default::default()
        },
    );

    job
}

impl<T> ScrollBuffer<T>
where
    T: Serialize,
{
    fn flush_buffer(&mut self) -> Result<()> {
        if self.flush.len() > 0 {
            let job: epaint::text::LayoutJob =
                convert_text_to_layout_job("User", self.flush.as_str(), USER_COLOUR);

            self.internal.push(job);
            self.flush = String::from("");
        };

        Ok(())
    }
}

impl<T> Default for ScrollBuffer<T> {
    fn default() -> Self {
        Self {
            internal: Vec::new(),
            flush: String::new(),
            rx: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChatGui {
    pub(crate) scroll_buffer: ScrollBuffer<LayoutJob>,
    pub(crate) gui_config: GuiConfig,

    #[serde(skip)]
    scroll_tx: Option<flume::Sender<LayoutJob>>,
    pub(crate) config_open: bool,

    #[serde(skip)]
    view: View
}

impl Default for ChatGui {
    fn default() -> Self {
        let (tx, rx) = flume::unbounded();
        let scroll_buffer = ScrollBuffer::<egui::text::LayoutJob>::new(rx);

        ChatGui {
            scroll_buffer,
            gui_config: GuiConfig::default(),
            scroll_tx: Some(tx),
            config_open: false,
            view: View::Main,
        }
    }
}

//impl ChatGui {
//    fn scroll_buffer(&mut self) -> &ScrollBuffer<LayoutJob> {
//        if self.scroll_buffer.is_none() {
//            let (tx, rx) = flume::unbounded();
//            let scroll_buffer = ScrollBuffer::<egui::text::LayoutJob>::new(rx);
//            self.scroll_buffer = Some(scroll_buffer);
//            self.scroll_tx = Some(tx);
//        }
//
//        &self.scroll_buffer.unwrap()
//    }
//}

impl ChatGui {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //match cc.storage {
        //    Some(storage) => {
        //        println!("Loaded prev");
        //        let app: ChatGui = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //        app.reload()
        //    }
        //    None => {
        //        println!("Loaded default");
        //
        //    }
        //}
        ChatGui::default().reload()
    }

    //fn config_window(&mut self, ui: &mut egui::Ui) {
    //}

    fn title_bar(&self, ui: &mut egui::Ui) {
        let mut title = LayoutJob{
            halign: Align::Center,
            ..Default::default()
        };

        title.append(
            "LLM Gui",
            0.0,
            epaint::text::TextFormat {
                font_id: FontId::new(20.0, FontFamily::Proportional),
                color: egui::Color32::RED,
                ..Default::default()
            },
        );

        ui.label(title);
    }

    fn top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Options", |ui| {
                    self.gui_config.local_ui(ui);
                    //if ui.button("Open").clicked() {
                    //    if let Some(path) = rfd::FileDialog::new().pick_file() {
                    //        *label = path.display().to_string();
                    //        ui.close_menu();
                    //    }
                    //}
                    let response = ui.button("Configuration");

                    if response.clicked() {
                        self.config_open = true;
                        self.gui_config.run_once = true;
                        ui.close_menu();
                    }

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });
    }

    fn config_window(&mut self, ui: &mut egui::Ui) {
        self.gui_config.model_list.get_listing_ui(ui);
        //self.gui_config.requests_ui(ui);
    }

    fn scrolling_window(&mut self, ui: &mut egui::Ui) {
        let text_style = egui::TextStyle::Body;
        let scroll_height = partial_min_max::max(ui.available_height() - 54.0, 0.0);
        let row_height = ui.text_style_height(&text_style);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .max_height(scroll_height)
            .show_rows(
                ui,
                row_height,
                self.scroll_buffer.size(),
                |ui, row_range| {
                    for row in row_range {
                        ui.label(self.scroll_buffer.internal[row].clone());
                    }
                },
            );
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal_top(|ui| {
            ui.label("> ");

            let response = ui
                .add(
                    egui::TextEdit::singleline(&mut self.scroll_buffer.flush)
                        .desired_width(partial_min_max::max(ui.available_width() - 70.0, 0.0)),
                )
                .on_hover_text_at_pointer("Enter Text");

            //if !&self.config_open {
            //    response.request_focus();
            //}

            if ui.button("Enter").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.scroll_buffer
                    .flush_buffer()
                    .expect("Something went wrong with the scroll buffer");

                //if !&self.config_open {
                //    response.request_focus();
                //}
            }
        });

         ui.add_space(10.0);
    }

    fn main_window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Control Central Panel Padding
        let side_padding = 25.0;
        let top_bottom_padding = 5.0;

        let frame = egui::containers::Frame::central_panel(&ctx.style()).inner_margin(
            egui::style::Margin {
                left: side_padding.clone(),
                right: side_padding,
                top: top_bottom_padding.clone(),
                bottom: top_bottom_padding,
            },
        );

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            //self.title_bar(ui);

            let mut title = LayoutJob::default();

            title.append(
                "📄 Main",
                0.0,
                epaint::text::TextFormat {
                    font_id: FontId::new(20.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );

            let mut config= LayoutJob::default();

            config.append(
                "💾 Config",
                0.0,
                epaint::text::TextFormat {
                    font_id: FontId::new(20.0, FontFamily::Proportional),
                    ..Default::default()
                },
            );

            ui.separator();
            ui.horizontal(|ui| {
                // Main Window 
                ui.selectable_value(&mut self.view, View::Main, title);
                ui.separator();
                // Config
                ui.selectable_value(&mut self.view, View::Config, config); 
            });

            ui.separator();
            ui.add_space(10.0);

            match self.view {
                View::Main => self.scrolling_window(ui),
                View::Config => self.config_window(ui), 
            }
        });
    }

    fn check_errors(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {}

    fn reload(mut self) -> Self {
        // Reload after spinning up from a serialise
        let (tx, rx) = flume::unbounded();
        let mut scroll_buffer = ScrollBuffer::<egui::text::LayoutJob>::new(rx);
        scroll_buffer.internal = self.scroll_buffer.internal;

        self.scroll_buffer = scroll_buffer;
        self.scroll_tx = Some(tx);

        self
    }
}

impl eframe::App for ChatGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        //#[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        //self.top_panel(ctx, frame);
        self.main_window(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
