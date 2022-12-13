FROM rust:latest

WORKDIR /app

RUN apt-get update \
    && apt-get install -y cmake build-essential \
    && cargo install diesel_cli --no-default-features --features "postgres sqlite" \
    && apt-get clean