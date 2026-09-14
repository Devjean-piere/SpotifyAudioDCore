FROM --platform=$BUILDPLATFORM rust:1-alpine3.20 AS builder
WORKDIR /app

ARG TARGETARCH
RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev openssl-libs-static pulseaudio-dev

# TARGETARCH ist "amd64" oder "arm64" – auf das passende Rust-Target mappen
RUN case "$TARGETARCH" in \
        amd64) echo x86_64-unknown-linux-musl > /tmp/target ;; \
        arm64) echo aarch64-unknown-linux-musl > /tmp/target ;; \
        *) echo "unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac
RUN rustup target add "$(cat /tmp/target)"

ENV OPENSSL_STATIC=1
ENV PKG_CONFIG_ALLOW_CROSS=1
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --verbose --target "$(cat /tmp/target)" \
    && cp target/"$(cat /tmp/target)"/release/spotifyAudioD /app/spotifyAudioD

FROM --platform=$TARGETPLATFORM alpine:3.20
WORKDIR /app
RUN apk add --no-cache ca-certificates libpulse curl
COPY --from=builder /app/spotifyAudioD /app/spotifyAudioD

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./spotifyAudioD"]