# https://just.systems

set shell := ["pwsh", "-c"]

export ORTGRAPH_USERNAME := "info@floorstudio.ru"
export ORTGRAPH_PASSWORD := "4lp3B7"
export MAIL_HOST := "mail.safira.club"
export MAIL_USER := "mail@safira.club"
export MAIL_PASS := "80ORbD4s7Sj5ukUV"
export MS_TOKEN := "6c5060f77e856df689f4d52419a607a3ac4f2e42"
export SAFIRA_CK := "ck_9584ef901debf952fbfd8a2f43c5f94901613a89"
export SAFIRA_CS := "cs_a3d2902f8b5f819e8473309df7c041ea1f16fc0e"
export SAFIRA_HOST := "https://safira.club"
export WINSTON_TOKEN := "6541293177:AAE4B6ZRD96g3tFvPI1GsiDhMsbIyglmNfs"
export DATABASE_URL := "postgres://postgres:postgres@localhost:5432/friday_api"

default:
    @just --list

run:
    cargo run
