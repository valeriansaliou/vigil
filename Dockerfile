FROM rust:latest AS build

ARG TARGETPLATFORM

WORKDIR /app
COPY . /app

RUN case ${TARGETPLATFORM} in \
    "linux/amd64")  echo "x86_64-unknown-linux-musl" > .toolchain ;; \
    "linux/arm64")  echo "aarch64-unknown-linux-musl" > .toolchain ;; \
    *)              echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

RUN apt-get update
RUN apt-get install -y musl-tools

RUN rustup target add $(cat .toolchain)

RUN cargo build --release --target $(cat .toolchain)
RUN mv ./target/$(cat .toolchain)/release/vigil ./

FROM alpine:latest

WORKDIR /usr/src/vigil

COPY ./res/assets/ ./res/assets/
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /app/vigil /usr/local/bin/vigil

CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
