FROM rust:1.81.0-bookworm AS chef

RUN cargo install cargo-chef
WORKDIR /image-resizer
ADD .. .

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV RUSTFLAGS="-C target-cpu=haswell -C opt-level=3 -C target-feature=+avx,+avx2,+sse2 --cfg reqwest_unstable"

COPY --from=planner /image-resizer/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin service

FROM rust:1.81.0-bookworm AS runner

COPY --from=builder --chown=65534 /image-resizer/target/release/service /usr/local/bin

ENV RUST_BACKTRACE=full

EXPOSE 8080

USER 65534

ENV RUSTFLAGS="--cfg reqwest_unstable"

CMD ["/usr/local/bin/service"]
