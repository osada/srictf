## Please execuite this in the project directory
## % docker build -t srictf .
## % # docker run -p 8080:8080 srictf
## % # docker image
## % docker tag ...TAG-ID... osada/srictf:latest
## % docker image ls
## % docker push osada/srictf:latest

## For builder
FROM rust:1.48 AS builder

WORKDIR /srictf

COPY Cargo.toml Cargo.toml
RUN mkdir src
RUN echo "fn main(){}" > src/main.rs
RUN cargo build --release
COPY ./src ./src
COPY ./templates ./templates
RUN rm -f target/release/deps/srictf*
RUN cargo build --release

## For Release image
FROM debian:buster-20230227

COPY --from=builder /srictf/target/release/srictf /usr/local/bin/srictf
CMD ["srictf"]
