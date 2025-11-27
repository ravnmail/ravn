use crate::navigation::NavigationUrl;

/// Navigate to a RAVN URL
#[tauri::command]
pub async fn navigate_to_url(url: String) -> Result<String, String> {
    log::debug!("[Navigation Command] Parsing URL: {}", url);
    let nav_url = NavigationUrl::parse(&url)?;
    let router_path = nav_url.to_router_path();
    log::debug!(
        "[Navigation Command] Mapped to router path: {}",
        router_path
    );
    Ok(router_path)
}

/// Build a RAVN URL from path and query
#[tauri::command]
pub async fn build_ravn_url(path: String, query: Option<String>) -> Result<String, String> {
    log::debug!(
        "[Navigation Command] Building URL from path: {} with query: {:?}",
        path,
        query
    );
    let ravn_url = NavigationUrl::build(&path, query.as_deref());
    log::debug!("[Navigation Command] Built URL: {}", ravn_url);
    Ok(ravn_url)
}

/// Open an external URL in the system's default browser
#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    log::debug!("[Navigation Command] Opening external URL: {}", url);
    opener::open(&url).map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}
