FROM rust:1.80-bookworm AS build

RUN cargo new tau
WORKDIR /tau
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build -r
RUN rm src/*.rs target/release/tau

COPY . .
RUN touch src/main.rs
RUN cargo build -r

FROM debian:bookworm-slim
COPY --from=build /tau/target/release/tau .
EXPOSE 2023

CMD ./tau
