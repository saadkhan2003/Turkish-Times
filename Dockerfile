FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libsqlite3-0 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/turkish-times /app/
COPY templates/ /app/templates/
COPY public/ /app/public/

EXPOSE 8000

ENV PORT=8000
ENV APP_URL=http://localhost:8000
ENV DATABASE_URL=sqlite://data/database.sqlite?mode=rwc

CMD ["./turkish-times"]
