FROM rust:1-bullseye

COPY Cargo.* *.rs ./

RUN cargo build --release

CMD ["./target/release/sps_status"]
