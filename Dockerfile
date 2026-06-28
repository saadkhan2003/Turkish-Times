# Stage 1: Build Rust binary
FROM rust:bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY migrations/ ./migrations/

RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/turkish-times /app/
COPY templates/ /app/templates/
COPY public/ /app/public/

EXPOSE 8000

ENV PORT=8000
ENV APP_URL=http://localhost:8000
ENV DATABASE_URL=sqlite:data/database.sqlite

CMD ["./turkish-times"]
