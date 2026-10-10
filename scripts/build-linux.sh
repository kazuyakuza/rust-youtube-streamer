#!/bin/sh
# Linux release build script (POSIX sh). Runs inside the Compose "rust" service with
# the working directory at /rust-youtube-streamer and CARGO_TARGET_DIR=/rust-streamer-target,
# as docker-compose.yml provides. Builds with the tracked lockfile and copies only the
# final executable into the gitignored root-level dist/ directory; intermediate Cargo
# output stays in the named Docker volume and Cargo diagnostics are passed through.

expected_project_directory="/rust-youtube-streamer"
release_binary_name="rust-youtube-streamer-service"
artifact_directory="dist"
artifact_path="$artifact_directory/$release_binary_name"

print_message() {
    printf '%s\n' "$1"
}

fail_with_message() {
    printf '%s\n' "build-linux: error: $1" >&2
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

release_binary_path="$CARGO_TARGET_DIR/release/$release_binary_name"

print_message "Building Linux release executable using the tracked lockfile: cargo build --release --locked"

cargo build --release --locked
build_exit_code=$?
if [ "$build_exit_code" -ne 0 ]; then
    fail_with_message "cargo build --release --locked failed with exit code $build_exit_code; cargo diagnostics above are not suppressed"
fi

print_message "Release build succeeded."

if ! path_is_non_empty_file "$release_binary_path"; then
    fail_with_message "expected release binary not found or empty: $release_binary_path"
fi

if ! mkdir -p "$artifact_directory"; then
    fail_with_message "could not create the dist/ directory: $artifact_directory"
fi

if ! cp "$release_binary_path" "$artifact_path"; then
    fail_with_message "failed to copy $release_binary_path to $artifact_path"
fi

if ! path_is_non_empty_file "$artifact_path"; then
    fail_with_message "final artifact is missing or empty after the copy: $artifact_path"
fi

print_message "Build succeeded: saved the Linux release executable to $artifact_path ($(wc -c < "$artifact_path") bytes)."
print_message "The artifact is a Linux executable built inside the Linux Rust container; it is not a Windows .exe."
exit 0
