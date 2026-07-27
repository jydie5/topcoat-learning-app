FROM rust:1.97-bookworm AS builder

WORKDIR /app
COPY . .

RUN cargo install topcoat-cli --version 0.4.0 --locked \
    && topcoat asset bundle --release

FROM debian:bookworm-slim

RUN useradd --create-home --uid 10001 app
WORKDIR /app

COPY --from=builder /app/target/release/topcoat-learning-app /app/topcoat-learning-app
COPY --from=builder /app/target/assets /app/assets

ENV HOST=0.0.0.0
ENV PORT=3000
ENV APP_ENV=production
ENV DATABASE_URL=sqlite:/data/topcoat-todos.db

RUN mkdir /data && chown -R app:app /app /data
USER app

EXPOSE 3000
CMD ["/app/topcoat-learning-app"]
