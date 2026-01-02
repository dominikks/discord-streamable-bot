use dotenv::dotenv;
use tracing_subscriber::{fmt, EnvFilter};

use discord_streamable_bot::discord_client::DiscordClient;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenv().ok();

    let format = fmt::format();
    let filter = EnvFilter::from_default_env();
    let subscriber = fmt().event_format(format).with_env_filter(filter).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let mut client = DiscordClient::new().await;
    client.run().await;
}
