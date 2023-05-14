FROM rust as builder
WORKDIR /usr/src/crypto_analyzer_yew
COPY . .
RUN ls
RUN cargo install --path=./crypto_analyzer_yew

FROM ubuntu
COPY --from=builder /usr/local/cargo/bin/crypto_analyzer_yew /usr/local/bin/crypto_analyzer_yew
CMD ["crypto_analyzer_yew"]