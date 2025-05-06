FROM ubuntu:latest

COPY ./target/release/sps_status ./sps_status

CMD ["./sps_status"]
