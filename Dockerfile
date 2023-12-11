FROM rust:1.74 as build

WORKDIR /app

COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src

RUN cargo build --release

FROM alpine
WORKDIR /app
COPY --from=build /app/target/release/echo-rs /app/echo-rs
CMD [ "./app/echo-rs" ]
