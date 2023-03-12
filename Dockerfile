FROM rust as builder
WORKDIR /usr/src/crypto-analyzer
COPY . .
RUN cargo install --path=.

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/crypto-analyzer /usr/local/bin/crypto-analyzer
CMD ["crypto-analyzer"]