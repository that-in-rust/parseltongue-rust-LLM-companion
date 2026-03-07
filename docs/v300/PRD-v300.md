# User Segment x Diferentiation

- Rust Open Source Library
    - Maintainers
    - Contributors

# Big Rocks


- Big-Rock-01: the scope and dependencies
    - language Rust 21
    - treesitter for
        - C C++ Javascript Typescript Python Java Go
    - rustcompiler


- Big-Rock-02: the primary-key
    - language|||kind|||scope|||name|||file_path|||discriminator
    - language: rust
    - kind: fn
    - scope: auth::service
    - name: authenticate_user
    - file_path: src/auth/service.rs
    - discriminator: sig_v3

- Big-Rock-03: code-graph-building
    - parse folder names
    - folders become entities of type folder, with distance from  
    - rust-ecosystem files
        - rust code
        - rust config
            - toml
        - rust tests
    - non-rust files
