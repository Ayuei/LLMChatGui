use llm;
use log::info;
use rand::SeedableRng;
use std::{collections::HashSet, io::Write};

use serenity::model::prelude::{Message, MessageId};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum GenerationError {
    #[error("The generation was cancelled.")]
    Cancelled,
    #[error("{0}")]
    Custom(String),
}

impl GenerationError {
    pub fn custom(s: impl Into<String>) -> Self {
        Self::Custom(s.into())
    }
}

pub struct LlmModel {
    // redis connection
    // Loaded Model
    pub model: llm::models::Llama,
}

pub struct Request {
    message_id: MessageId,
    pub prompt: String,
    tok_stream_tx: flume::Sender<Token>,
}

impl Request {
    pub fn from_discord_msg(msg: Message, sender: flume::Sender<Token>) -> Request {
        let mut content = msg.content.clone();

        for mention in &msg.mentions {
            content = content.replace(&format!("<@{}>", &mention.id.to_string()), "")
        }

        let prompt_str = format!("{system_prompt}### User: {message}\n\n### Assistant:\n",
            system_prompt="### System:\nYou are Stable Beluga, an AI that follows instructions extremely well. Help as much as you can. Remember, be safe, and don't do anything illegal.\n\n",
            message=content,
        );

        println!("PROMPT: {}", prompt_str);

        Request {
            message_id: msg.id,
            prompt: prompt_str,
            tok_stream_tx: sender,
        }
    }
}

pub enum Token {
    Token(String),
    Error(GenerationError),
}

impl LlmModel {
    pub fn load(path: &str, tokenizer_path: &str) -> LlmModel {
        let llama = llm::load::<llm::models::Llama>(
            std::path::Path::new(path),
            llm::TokenizerSource::Embedded,
            Default::default(),
            llm::load_progress_callback_stdout,
        )
        .unwrap_or_else(|err| panic!("Failed to load model: {err}"));

        LlmModel { model: llama }
    }
}

pub fn spawn_model_thread(
    request_q: flume::Receiver<Request>,
    model: Box<dyn llm::Model>,
    cancel_rx: flume::Receiver<MessageId>,
) {
    async_std::task::spawn(async move {
        while let Ok(req) = request_q.recv_async().await {
            match process_inference_request(&req, model.as_ref(), cancel_rx.clone()) {
                Ok(_) => (),
                Err(e) => {
                    if let Err(err) = req.tok_stream_tx.send(Token::Error(e)) {
                        eprintln!("Send error {}", err);
                    }
                }
            }
        }
    });
}

pub fn process_inference_request(
    request: &Request,
    model: &dyn llm::Model,
    cancel_rx: flume::Receiver<MessageId>,
) -> Result<(), GenerationError> {
    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut session = model.start_session(Default::default());

    let params = llm::InferenceParameters {
        ..Default::default()
    };

    session
        .infer(
            model,
            &mut rng,
            &llm::InferenceRequest {
                prompt: (&request.prompt).into(),
                parameters: &params,
                play_back_previous_tokens: false,
                maximum_token_count: None,
            },
            &mut Default::default(),
            move |t| {
                let cancellation_requests: HashSet<_> = cancel_rx.drain().collect();
                if cancellation_requests.contains(&request.message_id) {
                    return Err(GenerationError::Cancelled);
                }

                match t {
                    llm::InferenceResponse::SnapshotToken(t)
                    | llm::InferenceResponse::PromptToken(t)
                    | llm::InferenceResponse::InferredToken(t) => {
                        println!("Generated Token: {}", t);

                        request.tok_stream_tx.send(Token::Token(t)).map_err(|_| {
                            GenerationError::custom("Failed to send token to channel.")
                        })?
                    }
                    llm::InferenceResponse::EotToken => return Err(GenerationError::Cancelled),
                }

                Ok(llm::InferenceFeedback::Continue)
            },
        )
        .map(|_| ())
        .map_err(|e| GenerationError::custom(e.to_string()))
}
