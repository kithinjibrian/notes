#!/bin/sh
# Builds Notes, packs it into notes.rkb with Rapidkoi's ./main, and serves it
# on 127.0.0.1:8002. Open it with: ../rapidkoi/main rapidkoi://localhost:8002
# (Reload in the app's ⓘ after running this again). Its database is named by
# host and port, so the same port keeps the same notes. Ctrl+C stops it.
set -eu
RAPIDKOI="${RAPIDKOI:-../rapidkoi/main}"
cargo build --release --target wasm32-unknown-unknown
"$RAPIDKOI" pack target/wasm32-unknown-unknown/release/notes.wasm \
    --name Notes --version "$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)" \
    --description "Notes, kept on this device" \
    --asset search.svg=assets/search.svg --asset trash.svg=assets/trash.svg -o notes.rkb --force
exec "$RAPIDKOI" serve notes.rkb --port "${PORT:-8002}"
