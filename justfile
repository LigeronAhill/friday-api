# https://just.systems

set shell := ["sh", "-c"]
set windows-shell := ["powershell", "-NoLogo", "-Command"]

set dotenv-load := true


default:
    @just --list

run:
    cargo run

build-image:
    docker build -t ligeronahill/friday_server:latest .

run-compose:
    docker-compose up -d
