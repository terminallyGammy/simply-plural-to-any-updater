FROM rust:1-bullseye

WORKDIR /app

# docker build caching based on
# https://stackoverflow.com/a/58474618
# dummy build
COPY Cargo.* ./
RUN mkdir src && echo "fn main() {}" > src/dummy.rs
RUN cargo build --bin download_only --release

# proper build
COPY Cargo.* ./
COPY src src
RUN cargo build --release

CMD ["./target/release/sps_status"]
