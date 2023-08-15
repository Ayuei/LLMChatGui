#[macro_use]
extern crate log;

use anyhow::Result;
use serenity::{framework::StandardFramework, prelude::*};
use tokio;
use ChatBotGui::backend::discord::Handler;
use ChatBotGui::backend::model::LlmModel;
use ChatBotGui::frontend::gui::ChatGui;

//#[tokio::main]
//async fn main() -> Result<()> {
//    env_logger::init();
//
//    let model = LlmModel::load("./model/stablebeluga-7b.ggmlv3.q4_K_M.bin", "./model/tokenizer.model");
//    let framework = StandardFramework::new().configure(|c| c.prefix("!"));
//    let intents = GatewayIntents::default();
//
//    let mut client = Client::builder("MTEzNTE5MjMxMzAxNTA0MjIyOQ.GowBGz.zypLJ9uf2IWv4tnEo2NcDWCu6_nzRj2s1suxoQ", intents)
//        .framework(framework)
//        .event_handler(Handler::new(model))
//        .await?;
//
//    if let Err(why) = client.start().await {
//        println!("Client error: {why:?}");
//    }
//
//    Ok(())
//}

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "LLM ChatGui",
        native_options,
        Box::new(|cc| Box::new(ChatGui::new(cc))),
    );

    Ok(())
}
