############################################################
### Stage 1: Build
FROM clux/muslrust:stable as builder
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
FROM debian:stable-slim as composer

RUN addgroup --gid 1000 discordbot \
  && adduser -u 1000 --system --gid 1000 discordbot \
  && mkdir -p /app/clips \
  && chown -R discordbot:discordbot /app

COPY --chown=discordbot:discordbot --from=builder /app/target/x86_64-unknown-linux-musl/release/discord-streamable-bot /app/

############################################################
### Stage 3: Final image
FROM gcr.io/distroless/cc
LABEL maintainer="dominik@kus.software"

COPY --from=composer /etc/passwd /etc/
COPY --from=composer --chown=1000:1000 /app /app

USER discordbot
WORKDIR /app
VOLUME /app/clips

ENV RUST_LOG=info
CMD ["/app/discord-streamable-bot"]