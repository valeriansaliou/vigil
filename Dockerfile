FROM rustlang/rust:nightly AS build

WORKDIR /usr/src/app
COPY . .

RUN cargo clean && cargo build --release

# vigil dockerfile
FROM debian:stretch-slim

WORKDIR /usr/src/vigil

COPY ./res/assets/ ./res/assets/
COPY --from=build /usr/src/app/target/release/vigil /usr/local/bin/vigil

RUN apt-get update
RUN apt-get install -y libssl-dev libcurl3

CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
