# Local dev-check toolchain: pinned base rust:1.82 + baked-in rustfmt/clippy (the base image ships without them).
FROM rust:1.82
RUN rustup component add rustfmt clippy
