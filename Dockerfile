FROM rust:bookworm as builder
# use bookworm https://community.fly.io/t/rust-server-missing-libssl-so-3-on-new-deploy/15114/2

WORKDIR /app

RUN apt-get update && apt-get install -y libpq-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

# Copy the entrypoint script and make sure it's executable
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
COPY populate.sh /usr/local/bin/populate.sh
RUN chmod +x /usr/local/bin/populate.sh

# Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libpq5 openssl curl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /usr/local/bin/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY --from=builder /usr/local/bin/populate.sh /usr/local/bin/populate.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

CMD ["/app/backend"]
