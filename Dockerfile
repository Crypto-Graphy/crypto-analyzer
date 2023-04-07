FROM rust as builder
WORKDIR /usr/src/crypto_analyzer_server
COPY . .
RUN cargo install --path=./crypto_analyzer_server

FROM ubuntu
RUN apt-get update
RUN apt-get install -y libpq-dev # diesel dependency for postgres
COPY --from=builder /usr/local/cargo/bin/crypto_analyzer_server /usr/local/bin/crypto_analyzer_server
CMD ["crypto_analyzer_server"]