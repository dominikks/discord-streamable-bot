use crate::streamable_client::download_clip;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::ReactionType;
use serenity::prelude::*;
use std::convert::TryFrom;
use std::env;
use tokio::join;
use tracing::{error, info, instrument};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ready))]
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    #[instrument(skip(self, ctx))]
    async fn message(&self, ctx: Context, msg: Message) {
        lazy_static! {
            static ref STREAMABLE_REGEX: Regex =
                Regex::new(r"https://streamable\.com/([a-z0-9]+)").unwrap();
        }

        if let Some(capture) = STREAMABLE_REGEX.captures(&msg.content) {
            let shortcode = capture.get(1).unwrap().as_str();
            let reaction = msg
                .react(&ctx.http, ReactionType::try_from("⏬").unwrap())
                .await
                .unwrap();

            info!(?shortcode, "Downloading streamable clip");
            let reaction_ftr = match download_clip(shortcode, &msg.author.name).await {
                Ok(()) => {
                    info!("Download successful");
                    msg.react(&ctx.http, ReactionType::try_from("✅").unwrap())
                }
                Err(e) => {
                    error!(?e, "Download failed");
                    msg.react(&ctx.http, ReactionType::try_from("❌").unwrap())
                }
            };

            let (del_res, react_res) = join!(reaction.delete(&ctx.http), reaction_ftr);
            del_res.unwrap();
            react_res.unwrap();
        }
    }
}

pub struct DiscordClient {
    pub client: Client,
}

impl DiscordClient {
    #[instrument]
    pub async fn new() -> DiscordClient {
        let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");
        let client = Client::builder(&token)
            .event_handler(Handler)
            .await
            .expect("Error creating client");

        DiscordClient { client }
    }

    #[instrument(skip(self))]
    pub async fn run(&mut self) {
        if let Err(why) = self.client.start().await {
            info!("Client ended: {:?}", why);
        }
    }
}
