FROM rust:1.84-slim AS builder
WORKDIR /usr/src/backend

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get update && apt-get install -y pkg-config libssl-dev libudev-dev

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
	&& apt-get install -y ca-certificates netcat-openbsd \
	&& rm -rf /var/lib/apt/lists/*

# The compiled binary is named after the package: epitalk-backend
COPY --from=builder /usr/src/backend/target/release/epitalk-backend /usr/local/bin/backend
COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY scripts/wait-for-it.sh /usr/local/bin/wait-for-it.sh
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/bin/wait-for-it.sh

ENV RUST_LOG=info
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["/usr/local/bin/backend"]
