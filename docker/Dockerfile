ARG RUST_BUILDER_IMAGE=nexus.spacecorp.ru/rust:1.90-bookworm
ARG RUNTIME_IMAGE=nexus.spacecorp.ru/ci/debian:bookworm-ca

FROM ${RUST_BUILDER_IMAGE} AS builder

# Corporate CA so cargo trusts nexus.local over HTTPS.
COPY docker/certs/corporate-ca.crt /usr/local/share/ca-certificates/corporate-ca.crt
RUN update-ca-certificates

WORKDIR /build

COPY bridge/Cargo.toml bridge/Cargo.lock ./
COPY bridge/.cargo ./.cargo
COPY bridge/crates ./crates

# NOTE: bridge/rust-toolchain.toml is intentionally NOT copied into the image.

RUN TOOLCHAIN="$(ls /usr/local/rustup/toolchains | head -n1)" \
    && echo "Using preinstalled toolchain: ${TOOLCHAIN}" \
    && RUSTUP_TOOLCHAIN="${TOOLCHAIN}" cargo build --release --locked -p mm-bridge

FROM ${RUNTIME_IMAGE} AS runtime

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
