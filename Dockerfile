FROM rust:1.41-stretch as builder

ADD . /src
WORKDIR /src

RUN apt update && \
    cargo build --verbose --release && \
    cargo install

FROM debian:stretch
COPY --from=builder /usr/local/cargo/bin/file_system_worker /usr/bin

RUN apt update && apt install -y libssl1.1 ca-certificates
ENV AMQP_QUEUE=job_file_system
CMD file_system_worker
