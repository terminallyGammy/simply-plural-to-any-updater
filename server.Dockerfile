FROM debian:12

WORKDIR /app

RUN apt update && apt install -y openssl

COPY ./target/SP2VRC-Linux ./

RUN chmod +x ./*

CMD ["./SP2VRC-Linux"]
