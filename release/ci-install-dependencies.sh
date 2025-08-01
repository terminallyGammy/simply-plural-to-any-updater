#!/bin/bash

set -euo pipefail

sudo apt update

# Tauri
(cargo --list | grep -q tauri) || cargo install tauri-cli --version "2.7.1" --locked
sudo apt-get install \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# MinGW toolchains for Windows targets
sudo apt-get install -y gcc-mingw-w64-x86-64 gcc-mingw-w64-i686
