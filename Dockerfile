FROM rust:latest

WORKDIR /usr/src/downloader_bot
COPY . .

RUN cargo install --path .

CMD ["downloader_bot"]
