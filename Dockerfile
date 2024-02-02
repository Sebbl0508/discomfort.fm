ARG RUST_VERSION=1.75

FROM rust:${RUST_VERSION}-slim-bookworm AS build
WORKDIR /app

# Need dependencies for build
RUN apt-get update && apt-get install -y libopus-dev libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

RUN cargo init . --name discomfort-fm

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only dependencies
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./migrations ./migrations

RUN rm ./target/release/deps/discomfort_fm*
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=build /app/target/release/discomfort-fm .

RUN apt-get update && apt-get install --no-install-recommends -y openssl libopus0 libopusfile0 yt-dlp && rm -rf /var/lib/apt/lists/*

CMD ["./discomfort-fm"]