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

/// Extracts the shortcode from a streamable URL in a message.
/// Returns None if no valid streamable URL is found.
fn extract_streamable_shortcode(content: &str) -> Option<&str> {
    lazy_static! {
        static ref STREAMABLE_REGEX: Regex =
            Regex::new(r"https://streamable\.com/([a-z0-9]+)").unwrap();
    }

    STREAMABLE_REGEX
        .captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, _ctx, ready))]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    #[instrument(skip(self, ctx))]
    async fn message(&self, ctx: Context, msg: Message) {
        if let Some(shortcode) = extract_streamable_shortcode(&msg.content) {
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

        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
        let client = Client::builder(&token, intents)
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

#[cfg(test)]
mod tests {
    use super::*;

    // Parameterized test cases for extract_streamable_shortcode
    #[test]
    fn test_extract_streamable_shortcode() {
        let test_cases = vec![
            // (input, expected_output, description)
            (
                "Check out this clip: https://streamable.com/abc123",
                Some("abc123"),
                "valid url with text",
            ),
            (
                "https://streamable.com/first and https://streamable.com/second",
                Some("first"),
                "multiple urls - extracts first",
            ),
            ("Just a regular message", None, "no url"),
            ("https://streamable.com/", None, "incomplete url"),
            (
                "https://streamable.com/ABC123",
                None,
                "uppercase not allowed",
            ),
            (
                "https://streamable.com/xyz789",
                Some("xyz789"),
                "alphanumeric",
            ),
            (
                "Hey everyone! Check this out: https://streamable.com/test123 - awesome!",
                Some("test123"),
                "url embedded in text",
            ),
            (
                "https://streamable.com/a1b2c3d4",
                Some("a1b2c3d4"),
                "longer code",
            ),
            ("streamable.com/test", None, "missing https"),
            (
                "https://streamable.com/test-123",
                Some("test"),
                "hyphen stops matching",
            ),
        ];

        for (input, expected, description) in test_cases {
            let result = extract_streamable_shortcode(input);
            assert_eq!(
                result, expected,
                "Failed test case: {} - input: '{}'",
                description, input
            );
        }
    }
}
