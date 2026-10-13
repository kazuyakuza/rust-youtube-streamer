//! Console output contract tests: the exact brochure printed around the
//! authorization URL and the success line. The suites have no stdout-capture
//! helper (no capture crate, no fd-level redirect), so each test re-runs this
//! test binary as a child process — the suites' established
//! `std::process::Command` mechanism — with a print-mode environment variable;
//! the child prints and the parent asserts on the captured stdout. Offline;
//! no network, no Google account.

use std::process::Command;

use super::{print_authorization_url, print_success};

const PRINT_MODE_ENV: &str = "RUST_STREAMER_AUTH_CONSOLE_PRINT_MODE";
const AUTHORIZATION_URL_MODE: &str = "authorization_url";
const SUCCESS_MODE: &str = "success";
const CLIENT_ID_FIXTURE: &str = "1234567890.apps.googleusercontent.com";
const REDIRECT_URI_FIXTURE: &str = "http://127.0.0.1:52144";
const CREDENTIALS_PATH_FIXTURE: &str = "credentials/tokens.json";
const BROCHURE_TEST_NAME: &str =
    "app::auth::console::console_tests::print_authorization_url_uses_the_documented_brochure";
const SUCCESS_TEST_NAME: &str =
    "app::auth::console::console_tests::print_success_names_the_credentials_path_and_scope";

fn print_mode_requested(mode: &str) -> bool {
    std::env::var(PRINT_MODE_ENV).is_ok_and(|value| value == mode)
}

fn captured_stdout_of_print_child(test_name: &str, mode: &str) -> String {
    let output = Command::new(std::env::current_exe().expect("the test binary path"))
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env(PRINT_MODE_ENV, mode)
        .output()
        .expect("the child test process runs");
    assert!(
        output.status.success(),
        "the child test must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the child stdout is UTF-8")
}

fn fixture_authorization_url() -> String {
    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={CLIENT_ID_FIXTURE}&response_type=code"
    )
}

#[test]
fn print_authorization_url_uses_the_documented_brochure() {
    if print_mode_requested(AUTHORIZATION_URL_MODE) {
        print_authorization_url(&fixture_authorization_url(), REDIRECT_URI_FIXTURE);
        return;
    }
    let stdout = captured_stdout_of_print_child(BROCHURE_TEST_NAME, AUTHORIZATION_URL_MODE);
    let expected = format!(
        "To authorize YouTube access, open the following URL in a browser and approve the authorization prompt:\n\n{}\n\nThe application is waiting for the browser callback at {REDIRECT_URI_FIXTURE} (loopback only).\n",
        fixture_authorization_url()
    );
    assert!(
        stdout.contains(&expected),
        "brochure mismatch; captured stdout:\n{stdout}"
    );
}

#[test]
fn print_success_names_the_credentials_path_and_scope() {
    if print_mode_requested(SUCCESS_MODE) {
        print_success(CREDENTIALS_PATH_FIXTURE);
        return;
    }
    let stdout = captured_stdout_of_print_child(SUCCESS_TEST_NAME, SUCCESS_MODE);
    let expected = format!(
        "authorization succeeded: YouTube credentials saved at {CREDENTIALS_PATH_FIXTURE}; the required YouTube scope was granted\n"
    );
    assert!(
        stdout.contains(&expected),
        "success line mismatch; captured stdout:\n{stdout}"
    );
}
