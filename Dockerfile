FROM rust:1.79.0-alpine3.20 AS sinitc

RUN apk add musl
RUN apk add musl-dev
RUN rustup target add x86_64-unknown-linux-musl

COPY sinitc /srv/sinitc
WORKDIR /srv/sinitc

RUN cargo build --release


FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

ARG TINI_VERSION=v0.19.0
ADD https://github.com/krallin/tini/releases/download/${TINI_VERSION}/tini-static-amd64 /sbin/tini

RUN chmod +x /sbin/tini \
    && apt-get update \
    && apt-get install -y curl \
    && curl https://raw.githubusercontent.com/thetredev/dotfiles/main/vm/install-docker.sh | bash || true \
    && rm -rf /var/lib/apt/lists/*

COPY --link=true ./examples/services /etc/sinitc
COPY --link=true --from=sinitc /srv/sinitc/target/x86_64-unknown-linux-musl/release/sinitc /sbin/sinitc

ENTRYPOINT [ "/sbin/tini", "--", "/sbin/sinitc" ]
CMD [ "init", "/bin/bash" ]
