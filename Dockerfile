FROM rustlang/rust:nightly

WORKDIR /usr/src/vigil
COPY ./res/assets/ .

RUN cargo install vigil-server
CMD [ "vigil", "-c", "/etc/vigil.cfg" ]

EXPOSE 8080
