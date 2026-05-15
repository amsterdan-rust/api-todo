FROM rust:1.93-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/todo-api /app/todo-api

ENV RUST_LOG=todo_api=info,tower_http=info

CMD ["/app/todo-api"]
