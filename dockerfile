FROM rust:1.82-slim as builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

COPY --from=builder /usr/src/assets /usr/app/assets
COPY --from=builder /usr/src/data/sample.sqlite /usr/app/data/sample.sqlite
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/admin_panel-cli /usr/app/admin_panel-cli

ENV LOCO_ENV=production
CMD ["/usr/app/admin_panel-cli", "start", "-b", "[::]"]