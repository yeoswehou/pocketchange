FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y libpq-dev
RUN cargo build --release

COPY entrypoint.sh /usr/local/bin/entrypoint.sh

ENTRYPOINT ["entrypoint.sh"]

FROM debian:buster-slim

RUN apt-get update && apt-get install -y libpq5
COPY --from=builder /app/target/release/backend /app/backend

CMD ["/app/backend"]