# Local dev-check toolchain: pinned base rust:1.82 + baked-in rustfmt/clippy (the base image ships without them).
# gcc-mingw-w64-x86-64 provides the Windows GNU cross toolchain: dlltool is required at COMPILE time by
# windows-sys raw-dylib imports, gcc is the fallback linker route, objdump audits the exe imports.
FROM rust:1.82
RUN rustup component add rustfmt clippy
RUN apt-get update && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64
RUN rustup target add x86_64-pc-windows-gnu
