############################################################
### Stage 1: Build
FROM clux/muslrust:stable@sha256:12e02bbe193e3bd9255f9b5a9312f0039a0c79b9e015cf5e81eeab83a1eb8e0f AS builder
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
FROM debian:stable-slim@sha256:fbfb4059d2127120aa05b6e6064523d2102552f0cf0026ddf0701c12f1f8f3e0 AS composer

RUN groupadd --gid 1000 discordbot \
  && useradd -u 1000 --system --gid 1000 discordbot \
  && mkdir -p /app/clips \
  && chown -R discordbot:discordbot /app

COPY --chown=discordbot:discordbot --from=builder /app/target/x86_64-unknown-linux-musl/release/discord-streamable-bot /app/

############################################################
### Stage 3: Final image
FROM gcr.io/distroless/cc:latest@sha256:809d04afa1be033c441f76bb0051878c342fb76d0ea236bc6dc94b655fa0fbd6
LABEL maintainer="dominik@kus.software"

COPY --from=composer /etc/passwd /etc/
COPY --from=composer --chown=1000:1000 /app /app

USER discordbot
WORKDIR /app
VOLUME /app/clips

ENV RUST_LOG=info
CMD ["/app/discord-streamable-bot"]