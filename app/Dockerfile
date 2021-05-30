FROM ekidd/rust-musl-builder:stable as builder

RUN USER=root cargo new --bin telegram
WORKDIR ./telegram

ADD . ./

RUN cargo build --release

FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8000

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=builder /home/rust/src/telegram/target/x86_64-unknown-linux-musl/release/webhook ${APP}/webhook
COPY --from=builder /home/rust/src/telegram/target/x86_64-unknown-linux-musl/release/scheduler ${APP}/scheduler

WORKDIR ${APP}

# ENTRYPOINT ["./webhook"]