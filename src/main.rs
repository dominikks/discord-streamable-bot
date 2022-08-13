use tracing_subscriber::fmt;

mod discord_client;
mod streamable_client;

use discord_client::DiscordClient;

#[tokio::main]
async fn main() {
    let format = fmt::format();
    let subscriber = fmt().event_format(format).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let mut client = DiscordClient::new().await;
    client.run().await;
}
