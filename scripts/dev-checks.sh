#!/bin/sh
# Container-side development checks runner (POSIX sh; cwd must be /rust-youtube-streamer).
# Writes ONLY under logs/checks/. Set FORCE_FAIL to fmt-check|check|test|clippy to make
# exactly that check fail with exit 7 (failure-path validation); leave unset in normal use.

log_file=""
results=""
passed_count=0
failed_count=0

emit() {
    printf '%s\n' "$1"
    printf '%s\n' "$1" >> "$log_file"
}

run_check() {
    run_check_name=$1
    run_check_command=$2
    run_check_start=$(date +%s)
    if [ "$FORCE_FAIL" = "$run_check_name" ]; then
        run_check_command="echo intentional failure: FORCE_FAIL=$FORCE_FAIL 1>&2; exit 7"
    fi
    run_check_output=$(sh -c "$run_check_command" 2>&1)
    run_check_exit=$?
    run_check_duration=$(( $(date +%s) - run_check_start ))
    emit "== $run_check_name == start"
    emit "$run_check_output"
    if [ "$run_check_exit" -eq 0 ]; then
        run_check_status=PASS
        passed_count=$((passed_count + 1))
    else
        run_check_status=FAIL
        failed_count=$((failed_count + 1))
    fi
    run_check_result="$run_check_status $run_check_name exit=$run_check_exit ($run_check_duration s)"
    results="${results}
$run_check_result"
}

print_header() {
    emit "UTC time: $(date -u)"
    emit "Cargo: $(cargo --version)"
    emit "Log file: $log_file"
}

print_summary() {
    emit "--- Summary ---"
    emit "$results"
    if [ "$failed_count" -eq 0 ]; then
        overall_status="ALL CHECKS PASSED"
    else
        overall_status="$failed_count CHECK(S) FAILED"
    fi
    emit "$overall_status"
    emit "Log file: $log_file"
}

run_fmt_check() { run_check fmt-check "cargo fmt --check"; }
run_cargo_check() { run_check check "cargo check --locked"; }
run_cargo_test() { run_check test "cargo test --locked"; }
run_cargo_clippy() { run_check clippy "cargo clippy --locked -- -D warnings"; }

mkdir -p logs/checks
log_file="logs/checks/$(date -u +%Y%m%dT%H%M%SZ).log"
print_header
run_fmt_check
run_cargo_check
run_cargo_test
run_cargo_clippy
print_summary
if [ "$failed_count" -eq 0 ]; then
    exit 0
fi
exit 1
