ARG RUST_BUILDER_IMAGE=rust:1.90-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_BUILDER_IMAGE} AS builder

WORKDIR /build

COPY bridge/Cargo.toml bridge/Cargo.lock ./
COPY bridge/.cargo ./.cargo
COPY bridge/crates ./crates

RUN cargo build --release --locked -p mm-bridge

FROM ${RUNTIME_IMAGE} AS runtime

# Runtime-зависимости:
# - libssl3: для динамической линковки OpenSSL (используется reqwest/hyper-tls)
# - ca-certificates: корневые сертификаты для HTTPS-запросов к Mattermost
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       libssl3 \
       ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --create-home --home-dir /home/mmbridge mmbridge \
    && mkdir -p /data \
    && chown -R mmbridge:mmbridge /data /home/mmbridge

COPY --from=builder /build/target/release/mm-bridge /usr/local/bin/mm-bridge
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh

RUN chmod +x /usr/local/bin/entrypoint.sh

USER mmbridge

ENV MM_BRIDGE_LISTEN=0.0.0.0:8765
ENV MM_WEB_LISTEN=0.0.0.0:8766
ENV MM_DATA_DIR=/data
ENV RUST_LOG=mm_bridge=info,mm_web=info,warn

EXPOSE 8765 8766

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["mm-bridge"]
