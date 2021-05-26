FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin telegram
WORKDIR ./telegram
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/x86_64-unknown-linux-musl/release/deps/telegram*
RUN cargo build --release


FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8000

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/telegram/target/x86_64-unknown-linux-musl/release/telegram ${APP}/telegram

WORKDIR ${APP}

ENTRYPOINT ["./telegram"]