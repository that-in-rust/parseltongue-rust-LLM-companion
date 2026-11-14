# pinned-itoa-source-tree

Byte-identical files of [dtolnay/itoa 1.0.18](https://github.com/dtolnay/itoa/tree/af77385d0daf4d0e949e81f2588be2e44f69f086),
commit `af77385d0daf4d0e949e81f2588be2e44f69f086`, licensed MIT OR Apache-2.0 (licence files beside this one).
Test input for the Phase 1 real-source acceptance tests (SPEC REQ-RUST-033.0); nothing here is
built into the product.

```text
 src/lib.rs, src/u128_ext.rs      the whole library; digests checked against
                                  docs/phase-20261111/phase-one-benchmark-pin.json by the test
 benches/bench.rs                 present only because the manifest declares the bench target
 upstream-cargo-manifest-file.toml   upstream Cargo.toml, unchanged; renamed so Cargo does not
                                  treat this folder as a package. The test copies it to Cargo.toml
                                  in a temporary directory.
 qualification-cargo-lock-file.toml  upstream tracks no lockfile; this one was generated once
                                  (53 packages) and is used unchanged. Copied to Cargo.lock.
```

Do not edit these files. A change to the pin replaces them and the pin manifest together.
