FROM busybox as composer
RUN addgroup --gid 1000 discordbot \
  && adduser -u 1000 -S -G discordbot discordbot \
  && mkdir -p /app/clips \
  && chown -R discordbot:discordbot /app

ADD --chown=discordbot:discordbot discord-streamable-bot /app/
RUN chmod +x /app/discord-streamable-bot

# Deploy
FROM gcr.io/distroless/cc
LABEL maintainer="dominik@kdtk.de"

COPY --from=composer /etc/passwd /etc/passwd
COPY --from=composer --chown=discordbot:discordbot /app /app

USER discordbot
WORKDIR /app
VOLUME /app/clips

ENV RUST_LOG=info
CMD ["/app/discord-streamable-bot"]