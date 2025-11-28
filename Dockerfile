FROM rust:latest AS build

ARG PREBUILT_TAG
ARG TARGETPLATFORM

ENV PREBUILT_TAG=$PREBUILT_TAG

WORKDIR /app
COPY . /app

RUN case ${TARGETPLATFORM} in \
    "linux/amd64")  echo "x86_64" > .arch && echo "x86_64-unknown-linux-musl" > .toolchain ;; \
    "linux/arm64")  echo "aarch64" > .arch && echo "aarch64-unknown-linux-musl" > .toolchain ;; \
    *)              echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

# Run full build?
RUN if [ -z "$PREBUILT_TAG" ]; then \
    apt-get update && \
        apt-get install -y musl-tools && \
        rustup target add $(cat .toolchain) \
    ; fi
RUN if [ -z "$PREBUILT_TAG" ]; then \
    cargo build --release --target $(cat .toolchain) && \
        mkdir -p ./vigil/ && \
        mv ./target/$(cat .toolchain)/release/vigil ./vigil/ && \
        cp -rp ./res ./vigil/
    ; fi

# Pull pre-built binary?
RUN if [ ! -z "$PREBUILT_TAG" ]; then \
    wget https://github.com/valeriansaliou/vigil/releases/download/$PREBUILT_TAG/$PREBUILT_TAG-$(cat .arch).tar.gz && \
        tar -xzf $PREBUILT_TAG-$(cat .arch).tar.gz \
    ; fi

FROM alpine:latest

WORKDIR /usr/src/vigil

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /app/vigil/vigil /usr/local/bin/vigil
COPY --from=build /app/vigil/res/assets/ ./res/assets/

CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
