FROM rust:latest AS builder
COPY .. /project/
WORKDIR /project
RUN cargo build
CMD ["cargo", "run"]