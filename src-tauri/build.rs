fn main() {
    dotenv::dotenv().ok();
    const REQUIRED_VARS: [&str; 5] = [
        "OFFICE365_CLIENT_ID",
        "OFFICE365_CLIENT_SECRET",
        "OFFICE365_TENANT",
        "GMAIL_CLIENT_ID",
        "GMAIL_CLIENT_SECRET",
    ];

    const OPTIONAL_VARS: [&str; 2] = ["ACTIVATION_SERVICE_URL", "MID_SECRET"];

    for key in REQUIRED_VARS {
        match std::env::var(key) {
            Ok(val) => println!("cargo:rustc-env={}={}", key, val),
            Err(_) => {
                panic!("Environment variable {} is not set", key);
            }
        }
    }

    for key in OPTIONAL_VARS {
        if let Ok(val) = std::env::var(key) {
            println!("cargo:rustc-env={}={}", key, val);
        }
    }

    tauri_build::build()
}
