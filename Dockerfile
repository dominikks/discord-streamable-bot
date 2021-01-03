# Deploy
FROM gcr.io/distroless/cc
LABEL maintainer="dominik@kdtk.de"

USER nonroot
WORKDIR /app
ADD --chown=nonroot clips /app/
VOLUME /app/clips

ENV RUST_LOG=info
CMD ["./discord-streamable-bot"]

ADD --chown=nonroot discord-streamable-bot /app/