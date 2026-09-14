FROM rust:1-alpine3.20 AS builder
WORKDIR /app

RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev openssl-libs-static pulseaudio-dev

ENV OPENSSL_STATIC=1
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release \
    && cp target/release/spotifyAudioD /app/spotifyAudioD

FROM alpine:3.20
WORKDIR /app
RUN apk add --no-cache ca-certificates libpulse curl
COPY --from=builder /app/spotifyAudioD /app/spotifyAudioD

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./spotifyAudioD"]