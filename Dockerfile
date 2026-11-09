FROM rust:1-bookworm AS BUILDER

WORKDIR  /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home appuser

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

USER appuser
EXPOSE 3000

CMD ["backend"]