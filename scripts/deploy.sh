#!/bin/sh
# Deploys Notes to https://notes.dafifi.net: checks, builds, signs with your
# local key (it never leaves this machine) and uploads the signed site to
# the VM, where nginx serves it as is (no restart).
#
#   1. Refuses a tree with uncommitted changes, or Cargo.toml and
#      rapidkoi.toml disagreeing on the version.
#   2. cargo test, fmt --check and clippy -D warnings.
#   3. Pulls the live site into site/: the VM holds the versions published
#      so far and the index's sequence, which must only go up.
#   4. Builds the wasm and runs Rapidkoi's `publish`. A version already
#      published with other bytes is refused: bump the version in both files.
#   5. Uploads builds and the landing page first, then the key, then the
#      index, so no one ever sees an index naming a build that isn't there;
#      then removes the builds publish pruned.
#   6. Checks the live index is the one just signed.
#
# Overrides: RAPIDKOI (Rapidkoi's ./main), VM (ssh target).
set -eu
cd "$(dirname "$0")/.."

RAPIDKOI="${RAPIDKOI:-../rapidkoi/main}"
VM="${VM:-root@174.138.43.152}"
HOST="$(sed -n 's/^host = "\(.*\)"/\1/p' rapidkoi.toml)"
REMOTE="/opt/rapidkoi/sites/$HOST"
WELL=".well-known/rapidkoi"

say() { printf '\n== %s\n' "$*"; }

say "checking the tree and versions"
if [ -n "$(git status --porcelain)" ]; then
    echo "uncommitted changes: commit them first, so what's deployed is a commit" >&2
    exit 1
fi
cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
app_version="$(sed -n 's/^version = "\(.*\)"/\1/p' rapidkoi.toml | head -1)"
if [ "$cargo_version" != "$app_version" ]; then
    echo "Cargo.toml says $cargo_version, rapidkoi.toml says $app_version: make them the same" >&2
    exit 1
fi

say "testing"
cargo test -q
cargo fmt --check
cargo clippy -q --all-targets -- -D warnings

say "pulling the live site from $VM:$REMOTE"
mkdir -p site
rsync -a --delete "$VM:$REMOTE/" site/

say "building and signing Notes $app_version"
cargo build -q --release --target wasm32-unknown-unknown
out="$("$RAPIDKOI" publish)"
echo "$out"
case "$out" in
    *"nothing changed"*)
        echo "already live: nothing to upload"
        exit 0
        ;;
esac

say "uploading to $HOST"
rsync -a "site/$WELL/builds/" "$VM:$REMOTE/$WELL/builds/"
rsync -a site/index.html "$VM:$REMOTE/index.html"
rsync -a "site/$WELL/key" "$VM:$REMOTE/$WELL/key"
rsync -a "site/$WELL/index" "$VM:$REMOTE/$WELL/index"
rsync -a --delete "site/$WELL/builds/" "$VM:$REMOTE/$WELL/builds/"

say "checking https://$HOST"
live="$(mktemp)"
trap 'rm -f "$live"' EXIT
curl -sf --max-time 20 -o "$live" "https://$HOST/$WELL/index"
if cmp -s "$live" "site/$WELL/index"; then
    echo "live: Notes $app_version at rapidkoi://$HOST"
else
    echo "the live index isn't the one just signed: check the VM" >&2
    exit 1
fi
