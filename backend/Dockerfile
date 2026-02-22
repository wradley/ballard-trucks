FROM rust:1.93-trixie AS builder
WORKDIR /app
COPY backend ./backend
COPY db ./db
WORKDIR /app/backend
RUN cargo build --release

FROM debian:trixie
RUN useradd -m appuser
WORKDIR /app
COPY --from=builder /app/backend/target/release/ballard-trucks-backend /app/ballard-trucks-backend
RUN mkdir -p /data && chown appuser:appuser /data
USER appuser
ENV SQLITE_PATH=/data/ballard.sqlite
CMD ["/app/ballard-trucks-backend"]
