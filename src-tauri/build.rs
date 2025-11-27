fn main() {
    dotenv::dotenv().ok();
    const VARS: [&str; 5] = [
        "OFFICE365_CLIENT_ID",
        "OFFICE365_CLIENT_SECRET",
        "OFFICE365_TENANT",
        "GMAIL_CLIENT_ID",
        "GMAIL_CLIENT_SECRET",
    ];

    for key in VARS {
        match std::env::var(key) {
            Ok(val) => println!("cargo:rustc-env={}={}", key, val),
            Err(_) => {
                panic!("Environment variable {} is not set", key);
            }
        }
    }

    tauri_build::build()
}
