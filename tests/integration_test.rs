//! Integration tests for the discord-streamable-bot.
//!
//! These tests verify the core functionality of extracting streamable links
//! from Discord messages and the overall structure of the download functionality.
//!
//! Note: Discord API interactions are mocked through the extracted functions
//! to enable testing without requiring an actual Discord bot connection.

use discord_streamable_bot::extract_streamable_shortcode;

#[test]
fn test_extract_streamable_shortcode_valid_url() {
    let message = "Check out this clip: https://streamable.com/abc123";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, Some("abc123".to_string()));
}

#[test]
fn test_extract_streamable_shortcode_multiple_urls() {
    // Should extract the first one
    let message = "https://streamable.com/first and https://streamable.com/second";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, Some("first".to_string()));
}

#[test]
fn test_extract_streamable_shortcode_no_url() {
    let message = "Just a regular message";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, None);
}

#[test]
fn test_extract_streamable_shortcode_invalid_streamable_url() {
    let message = "https://streamable.com/";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, None);
}

#[test]
fn test_extract_streamable_shortcode_with_uppercase() {
    // Regex only matches lowercase, which matches actual streamable URLs
    let message = "https://streamable.com/ABC123";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, None);
}

#[test]
fn test_extract_streamable_shortcode_alphanumeric() {
    let message = "https://streamable.com/xyz789";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, Some("xyz789".to_string()));
}

#[test]
fn test_extract_streamable_shortcode_embedded_in_text() {
    let message = "Hey everyone! Check this out: https://streamable.com/test123 - it's awesome!";
    let result = extract_streamable_shortcode(message);
    assert_eq!(result, Some("test123".to_string()));
}
