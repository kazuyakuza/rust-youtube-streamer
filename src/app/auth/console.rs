//! Console output of the auth flow: plain stdout prints that reach the
//! interactive operator. Deliberately NOT tracing events and NOT
//! test-assertable through the tracing layer (plan D11: no auto browser
//! launch, no interactive input — the console prints instructions and the
//! authorization URL, and nothing else).

pub(super) fn print_authorization_url(url: &str, redirect_uri: &str) {
    println!("To authorize YouTube access, open the following URL in a browser and approve the authorization prompt:");
    println!();
    println!("{url}");
    println!();
    println!(
        "The application is waiting for the browser callback at {redirect_uri} (loopback only)."
    );
}

pub(super) fn print_success(credentials_path: &str) {
    println!("authorization succeeded: YouTube credentials saved at {credentials_path}; the required YouTube scope was granted");
}
