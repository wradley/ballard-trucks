FROM rust:1.93-trixie AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:trixie
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/ballard-trucks-backend /app/ballard-trucks-backend
USER appuser
CMD ["/app/ballard-trucks-backend"]
