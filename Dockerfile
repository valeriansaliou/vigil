FROM rustlang/rust:nightly AS build

RUN apt-get update
RUN cargo install vigil-server

FROM debian:stretch-slim

WORKDIR /usr/src/vigil

COPY ./res/assets/ ./res/assets/
COPY --from=build /usr/local/cargo/bin/vigil /usr/local/bin/vigil

CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
