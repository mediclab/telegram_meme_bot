FROM rust:latest

WORKDIR /app

RUN apt-get update && apt-get install -y cmake build-essential && apt-get clean