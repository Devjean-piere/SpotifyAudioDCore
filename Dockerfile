FROM --platform=$BUILDPLATFORM rust:1-alpine3.20 AS builder
WORKDIR /app

ARG TARGETARCH
# 1. Benötigte Pakete + den passenden Cross-Compiler je nach Zielarchitektur hinzufügen
RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev openssl-libs-static pulseaudio-dev \
    gcc-aarch64-unknown-linux-musl musl-dev-aarch64

# TARGETARCH auf das passende Rust-Target mappen
RUN case "$TARGETARCH" in \
        amd64) echo x86_64-unknown-linux-musl > /tmp/target ;; \
        arm64) echo aarch64-unknown-linux-musl > /tmp/target ;; \
        *) echo "unsupported arch: $TARGETARCH" && exit 1 ;; \
    esac
RUN rustup target add "$(cat /tmp/target)"

ENV OPENSSL_STATIC=1
ENV PKG_CONFIG_ALLOW_CROSS=1

# 2. Cargo mitteilen, welcher Linker für aarch64 genutzt werden soll
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-unknown-linux-musl-gcc

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --verbose --target "$(cat /tmp/target)" \
    && cp target/"$(cat /tmp/target)"/release/spotifyAudioD /app/spotifyAudioD

FROM alpine:3.20
WORKDIR /app
RUN apk add --no-cache ca-certificates libpulse curl
COPY --from=builder /app/spotifyAudioD /app/spotifyAudioD

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./spotifyAudioD"]