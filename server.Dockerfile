FROM ubuntu:latest

WORKDIR /app

# ensure rust connections to web works
RUN apt-get update && apt-get install -y openssl ca-certificates

# Tauri dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

COPY ./mounted/SP2Any-Linux .

RUN chmod +x ./*

ENTRYPOINT ["./SP2Any-Linux"]
