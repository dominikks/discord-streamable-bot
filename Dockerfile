############################################################
### Stage 1: Build
FROM clux/muslrust:stable@sha256:f2a743194641985fe49ed340369d71fdaefbbdda20871514b5dd086319c454d6 AS builder
WORKDIR /app

### Dep caching start
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release
### Dep caching end

COPY ./ .
RUN touch src/main.rs
RUN cargo build --release

############################################################
### Stage 2: Compose
FROM debian:stable-slim@sha256:13e0a2e78aa0554ad5ff66e4e260c6dbcdff807eb1f39d43b9fb0288aae88b04 AS composer

RUN groupadd --gid 1000 discordbot \
  && useradd -u 1000 --system --gid 1000 discordbot \
  && mkdir -p /app/clips \
  && chown -R discordbot:discordbot /app

COPY --chown=discordbot:discordbot --from=builder /app/target/x86_64-unknown-linux-musl/release/discord-streamable-bot /app/

############################################################
### Stage 3: Final image
FROM gcr.io/distroless/cc:latest@sha256:58e12be899275afaa05f1b03051ab19037ed7f252528e57a3a9ad7e220a62578
LABEL maintainer="dominik@kus.software"

COPY --from=composer /etc/passwd /etc/
COPY --from=composer --chown=1000:1000 /app /app

USER discordbot
WORKDIR /app
VOLUME /app/clips

ENV RUST_LOG=info
CMD ["/app/discord-streamable-bot"]