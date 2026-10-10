#!/bin/sh
# Windows x86-64 GNU release build script (POSIX sh). Runs inside the Compose "rust"
# service with the working directory at /rust-youtube-streamer and
# CARGO_TARGET_DIR=/rust-streamer-target, as docker-compose.yml provides. Cross-compiles
# with the tracked lockfile and copies only the final .exe into the gitignored
# root-level dist/windows/ directory; intermediate Cargo output stays in the named
# Docker volume, Cargo diagnostics are passed through, and nothing is deleted.

expected_project_directory="/rust-youtube-streamer"
expected_target_triple="x86_64-pc-windows-gnu"
release_binary_name="rust-youtube-streamer-service.exe"
artifact_directory="dist/windows"
artifact_path="$artifact_directory/$release_binary_name"

print_message() {
    printf '%s\n' "$1"
}

fail_with_message() {
    printf '%s\n' "build-windows: error: $1" >&2
    exit 1
}

current_directory_matches_project_root() {
    [ "$(pwd -P)" = "$expected_project_directory" ]
}

cargo_target_directory_is_set() {
    [ -n "$CARGO_TARGET_DIR" ]
}

path_is_non_empty_file() {
    [ -s "$1" ]
}

if ! current_directory_matches_project_root; then
    fail_with_message "expected the current working directory to be $expected_project_directory (the documented Compose working directory); got: $(pwd -P)"
fi

if ! cargo_target_directory_is_set; then
    fail_with_message "CARGO_TARGET_DIR is unset or empty; it must point at the release output directory (docker-compose.yml sets it to /rust-streamer-target)"
fi

release_binary_path="$CARGO_TARGET_DIR/$expected_target_triple/release/$release_binary_name"

print_message "Building Windows x86-64 GNU release executable using the tracked lockfile: cargo build --release --locked --target $expected_target_triple"

cargo build --release --locked --target "$expected_target_triple"
build_exit_code=$?
if [ "$build_exit_code" -ne 0 ]; then
    fail_with_message "cargo build --release --locked --target $expected_target_triple failed with exit code $build_exit_code; the cargo diagnostics above are not suppressed"
fi

print_message "Release build succeeded."

if ! path_is_non_empty_file "$release_binary_path"; then
    fail_with_message "expected release executable not found or empty: $release_binary_path"
fi

if ! mkdir -p "$artifact_directory"; then
    fail_with_message "could not create the dist/windows/ directory: $artifact_directory"
fi

if ! cp "$release_binary_path" "$artifact_path"; then
    fail_with_message "failed to copy $release_binary_path to $artifact_path"
fi

if ! path_is_non_empty_file "$artifact_path"; then
    fail_with_message "final artifact is missing or empty after the copy: $artifact_path"
fi

print_message "Build succeeded: saved the Windows x86-64 GNU release executable to $artifact_path ($(wc -c < "$artifact_path") bytes)."
print_message "The GNU runtime is statically linked: the observed import audit (x86_64-w64-mingw32-objdump) found no MinGW runtime DLLs, and only standard Windows system DLLs are required (msvcrt.dll, kernel32.dll, ntdll.dll, userenv.dll, ws2_32.dll, api-ms-win-core-synch-l1-2-0.dll, bcryptprimitives.dll)."
print_message "Cross-compilation does not prove native Windows runtime behavior: the .exe must be tested separately on Windows. FFmpeg remains an independently installed external prerequisite and is not bundled."
exit 0
