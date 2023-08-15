use anyhow::Result;
use async_std::stream::StreamExt;
use flume;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::{self, async_trait};
use std::time::Duration;

use super::model::{spawn_model_thread, Request, Token};
const UPDATE_INTERVAL: Duration = Duration::from_millis(250);

pub struct Handler {
    request_tx: flume::Sender<Request>,
    cancel_tx: flume::Sender<MessageId>, //finished_req_tx: flume::Sender<Token>,
                                         //finished_req_rx: flume::Receiver<Token>,
}

pub fn remove_prompt(in_str: &str, prompt: &str, num_dots: usize) -> String {
    match in_str.strip_prefix(prompt) {
        Some(v) => v.trim().to_owned(),
        None => {
            let mut s = String::new();
            s.push_str("Thinking");

            for _ in 1..(num_dots % 10) {
                s.push('.');
            }
            s
        }
    }
}

pub async fn generate(handler: &Handler, ctx: Context, msg: Message) -> Result<()> {
    // Start the generation process
    let (token_tx, token_rx) = flume::unbounded::<Token>();
    let request = Request::from_discord_msg(msg.clone(), token_tx);
    let prompt = &request.prompt.clone();

    handler.request_tx.send_async(request).await?;

    let mut tok_stream = token_rx.into_stream();
    let mut message = String::new();
    let mut last_update = std::time::Instant::now();

    // Initial message handle
    let mut gen_msg_handle = msg.reply(&ctx.http, "Queued...").await?;
    let mut num_itr = 0;

    while let Some(token) = tok_stream.next().await {
        match token {
            Token::Token(t) => {
                println!("Received {}", t);
                message += &t;

                let formatted_msg = &remove_prompt(&message, prompt, num_itr);

                // Let's not hit the rate limit
                if !formatted_msg.is_empty() && last_update.elapsed() > UPDATE_INTERVAL {
                    gen_msg_handle
                        .edit(&ctx, |m| m.content(formatted_msg))
                        .await?;
                    last_update = std::time::Instant::now();
                }
            }
            Token::Error(e) => {
                println!("Generation stopped: {}", e);
                gen_msg_handle
                    .edit(&ctx, |m| m.content("Request cancelled!"))
                    .await?;
                tokio::time::sleep(Duration::from_secs(3)).await;
                gen_msg_handle.delete(&ctx).await?;
                return Ok(());
            }
        }

        num_itr += 1;
    }

    let formatted_msg = &remove_prompt(&message, prompt, num_itr);

    if !formatted_msg.is_empty() {
        gen_msg_handle
            .edit(&ctx, |m| m.content(formatted_msg))
            .await?;
    }

    Ok(())
}

impl Handler {
    pub fn new(model: super::model::LlmModel) -> Handler {
        let (request_tx, request_rx) = flume::bounded::<Request>(1);
        let (cancel_tx, cancel_rx) = flume::unbounded::<MessageId>();

        spawn_model_thread(request_rx, Box::new(model.model), cancel_rx);

        Handler {
            request_tx,
            cancel_tx,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Resumed");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        println!("Message received!");
        match msg.mentions_me(&ctx.http).await {
            Ok(m) => {
                if m {
                    println!("Mention detected");
                    if let Err(e) = generate(self, ctx, msg).await {
                        eprintln!("Some error occured during generation: {}", e);
                    }
                }
            }
            Err(err) => eprintln!("Serenity encountered an error {}!", err.to_string()),
        }
    }

    async fn message_delete(
        &self,
        _: Context,
        _: ChannelId,
        msg_id: MessageId,
        _: Option<GuildId>,
    ) {
        match self.cancel_tx.send_async(msg_id).await {
            Ok(_) => (),
            Err(e) => eprintln!("Could not send cancellation request {}", e),
        }
    }
}
