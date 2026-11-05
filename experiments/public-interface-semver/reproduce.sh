#!/bin/sh
set -eu
cd "$(dirname "$0")"
repo=work/semver
nightly=nightly-2026-04-01
expected=6ed8561154715b2c34df417a2052597d586f2c43
mkdir -p work
if [ ! -d "$repo/.git" ]; then
    git clone --depth 1 --single-branch --branch 1.0.27 https://github.com/dtolnay/semver.git "$repo"
fi
test "$(git -C "$repo" rev-parse HEAD)" = "$expected"
cp workspace.toml work/Cargo.toml
cp rustdoc.Cargo.lock work/Cargo.lock
for profile in default serde; do
    set --
    if [ "$profile" = serde ]; then set -- --features serde; fi
    cargo "+$nightly" rustdoc --locked --lib --manifest-path "$repo/Cargo.toml" \
        --target-dir "work/$profile" "$@" -- -Z unstable-options --output-format json
done
python3 build_graph.py
python3 -m unittest test_graph
cargo run --locked --quiet --manifest-path consumer/Cargo.toml --target-dir work/consumer-target
