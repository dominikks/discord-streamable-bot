![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/dominikks/discord-streamable-bot/build.yaml?branch=main)
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/dominikks/discord-streamable-bot)
![GitHub](https://img.shields.io/github/license/dominikks/discord-streamable-bot)

# discord-streamable-bot

A discord bot that automatically downloads streamable clips posted to a text channel.

## Usage

The bot automatically looks for streamable links in all text channels it has access to.
If a link is found, the clips is downloaded to the clips folder (`/app/clips`).

## Installation

The app is available via Docker.

You need to get a Bot token from the [Discord Developer Portal](https://discord.com/developers/applications).
To get one, go to the portal, add an application, go to the bot tab, create a bot and copy the token.
Also check the box for "Message Content Intent" under priviledged intents (otherwise the bot cannot extract the links from your messages).
Then, you can run the bot:

```
docker run -e DISCORD_TOKEN=<token> -v ./clips:/app/clips ghcr.io/dominikks/discord-streamable-bot
```

Clips are saved under `/app/clips` in the container, so feel free to mount that somewhere.
By default, the app runs with UID 1000, so make sure the mounted folder is owned by a user with that UID (e.g. `chown 1000 ./clips`).

To add the bot to your Discord server, you can use the following link.
You can get the client id from the "General Information" tab of your application in the Discord Developer Portal.

```
https://discordapp.com/oauth2/authorize?client_id=<client_id>&scope=bot&permissions=2112
```

Congratulations!
Your bot is now active in your server and searchs for streamable links in all text channels it has access to.

## Development

You need to have the rust toolchain.
Then, simply execute

```
DISCORD_TOKEN=<token> cargo run
```

to run the app.

### Testing

Run the tests with:

```bash
cargo test
```

Or with output:

```bash
cargo test -- --nocapture
```

### Linting and Formatting

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy
```
