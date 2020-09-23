FROM rust:1.46-buster as builder

ADD . /src
WORKDIR /src

RUN apt-get update && \
    cargo build --verbose --release && \
    cargo install --path .

FROM debian:buster
COPY --from=builder /usr/local/cargo/bin/file_system_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates
ENV AMQP_QUEUE=job_file_system
CMD file_system_worker
