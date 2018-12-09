FROM rustlang/rust:nightly

WORKDIR /usr/src/vigil
COPY ./res/assets/ ./res/assets/

RUN apt-get update
RUN apt-get install -y libstrophe-dev
RUN rustup install nightly-2018-08-24
RUN rustup default nightly-2018-08-24
RUN cargo install vigil-server --all-features
CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
