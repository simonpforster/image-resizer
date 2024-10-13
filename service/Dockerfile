FROM rust:1.81.0-bookworm as chef

RUN cargo install cargo-chef
WORKDIR /image-resizer
ADD .. .

FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /image-resizer/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin service

FROM rust:1.81.0-bookworm as runner

COPY --from=builder /image-resizer/target/release/service /usr/local/bin

ENV RUST_BACKTRACE=full

EXPOSE 8080

CMD ["/usr/local/bin/service"]
