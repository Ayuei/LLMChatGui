use serde::{Deserialize, Serialize};
use glob::glob;
use egui::Ui;
use serenity::model::prelude::MessageId;
use std::time::{Instant, Duration};

const MATCH_LLAMA: &str = "*.bin";

#[derive(Serialize, Deserialize, Default)]
pub struct ModelListing {
    pub(crate) base_dir: String,
    pub(crate) use_local_llm: bool,
    #[serde(skip)]
    last_checked: Option<time::Instant>, 
    selected: String,
}

impl ModelListing {
    pub fn new(path: &str) -> ModelListing {
        return ModelListing {
            base_dir: path.into(),
            ..Default::default()
        };
    }

    pub fn get_listing(&mut self) -> Vec<String> {
        let now = Instant::now();

        if self.last_checked.is_some() && (self.last_checked.unwrap().elapsed()) > Duration::from_secs(10) {
            glob(&format!("{}/{}",&self.base_dir, MATCH_LLAMA))
                .unwrap()
                .filter_map(|p| p.ok())
                .map(|path| path.display().to_string())
                .collect()
        } else {
            Vec::new()
        }

        //for entry in 
        //    match entry {
        //        Ok(path) => {
        //            //ui.selectable_value(
        //            //    &mut self.selected,
        //                path.display().to_string()
        //                //path.display().to_string(),
        //            //);
        //        }
        //        Err(_) => (),
        //    }
        //}
    }
}

#[derive(Serialize, Deserialize)]
pub struct GuiPrompt {
    pub(crate) system_prompt: String,
    pub(crate) prompt_template: String,
}

impl Default for GuiPrompt {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            prompt_template: String::from("SYSTEM: {SYSTEM}\n\nUSER: {USER}\n\nASSISTANT:"),
        }
    }
}

impl GuiPrompt {
    pub fn default_id() -> MessageId {
        MessageId(0)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct GuiConfig {
    pub(crate) request_url: String,
    pub(crate) model_list: ModelListing,
    pub(crate) prompt: GuiPrompt,
    pub(crate) run_once: bool,
    pub(crate) is_open: bool,
}

impl GuiConfig {

    //pub fn

    pub fn local_ui(&mut self, ui: &mut Ui) {
        if ui.button("Open").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                if path.exists() {
                    self.model_list.base_dir = path.display().to_string();
                    ui.close_menu();
                }
            }
        }
    }

    pub fn requests_ui(&mut self, ui: &mut Ui) {
        ui.label(egui::RichText::new("Request Details").strong());
        ui.horizontal(|ui| {
            ui.label("URL");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.request_url),
            );

            if self.run_once == true {
                response.request_focus();
                self.run_once = false;
            }

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if self.is_open == true {
                    self.is_open = false;
                }
            }

            ui.label("Port");
            //ui.add(egui::TextEdit::singleline(&mut self.config.port)
            //    .desired_width(30.0));
        });

        ui.add_space(4.0);
        ui.separator();
        ui.label("Prompt Template");
        ui.text_edit_multiline(&mut self.prompt.prompt_template)
            .on_hover_text_at_pointer("Use {SYSTEM} for system prompt and {USER} for user prompt.");
        ui.label("System Prompt");
        ui.text_edit_multiline(&mut self.prompt.system_prompt)
            .on_hover_text_at_pointer("System prompt for LLM");
        //ui.label("User Prompt");
        //ui.add(egui::TextEdit::singleline(&mut self.prompt.user_prompt));

        if ui.button("Save").clicked() {
            self.is_open = false;
        }
    }

    fn get_listings_ui(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.use_local_llm, "Use Local?");

        if self.use_local_llm {
            egui::ComboBox::from_label("Which LLM?")
                .selected_text(format!("{:?}", self.selected))
                .show_ui(ui, |ui| {

                }
            );
        } 
    }
}
