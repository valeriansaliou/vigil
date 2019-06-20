FROM rustlang/rust:nightly AS build

WORKDIR /app

COPY . /app

RUN rustup --version
RUN rustup install nightly-2019-05-14 && \
    rustup default nightly-2019-05-14

RUN rustc --version && \
    rustup --version && \
    cargo --version

RUN cargo clean && cargo build --release

FROM debian:stretch-slim

WORKDIR /usr/src/vigil

COPY ./res/assets/ ./res/assets/
COPY --from=build /app/target/release/vigil /usr/local/bin/vigil

RUN apt-get update
RUN apt-get install -y libssl-dev libcurl3

CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
