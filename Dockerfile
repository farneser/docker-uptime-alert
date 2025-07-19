FROM rust:latest as build

RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup update stable && rustup default stable

WORKDIR /build

COPY . ./

RUN cargo build --release

FROM ubuntu as run

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=build /build/target/release/docker-uptime-alert /app/docker-uptime-alert

RUN chmod +x /app/docker-uptime-alert

EXPOSE 3000

CMD ["/app/docker-uptime-alert"]
