pub mod discord_client;
pub mod streamable_client;

// Re-export the extract function for testing
pub use discord_client::extract_streamable_shortcode;
