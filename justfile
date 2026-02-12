# https://just.systems

set shell := ["sh", "-c"]
set windows-shell := ["powershell", "-NoLogo", "-Command"]

set dotenv-load := true


default:
    @just --list

run:
    cargo run
