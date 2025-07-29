FROM ubuntu:latest

WORKDIR /app

RUN apt update && apt install -y openssl ca-certificates # ensure rust connections to web works

COPY ./target/SP2VRC-Linux ./

RUN chmod +x ./*

CMD ["./SP2VRC-Linux", "--webserver"]
