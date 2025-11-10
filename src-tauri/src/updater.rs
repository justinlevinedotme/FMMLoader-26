use serde::{Deserialize, Serialize};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_REPO: &str = "justinlevinedotme/FMMLoader-26";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub has_update: bool,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

pub fn check_for_updates() -> Result<UpdateInfo, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "FMMLoader26")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    if !response.status().is_success() {
        return Ok(UpdateInfo {
            has_update: false,
            current_version: CURRENT_VERSION.to_string(),
            latest_version: CURRENT_VERSION.to_string(),
            download_url: String::new(),
        });
    }

    let release: GitHubRelease = response
        .json()
        .map_err(|e| format!("Failed to parse release data: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v');
    let has_update = compare_versions(CURRENT_VERSION, latest_version);

    Ok(UpdateInfo {
        has_update,
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_version.to_string(),
        download_url: release.html_url,
    })
}

fn compare_versions(current: &str, latest: &str) -> bool {
    let current_parts: Vec<&str> = current.split('.').collect();
    let latest_parts: Vec<&str> = latest.split('.').collect();

    let max_len = current_parts.len().max(latest_parts.len());

    for i in 0..max_len {
        let current_part = current_parts.get(i).unwrap_or(&"0");
        let latest_part = latest_parts.get(i).unwrap_or(&"0");

        let current_num = current_part.parse::<u32>().unwrap_or(0);
        let latest_num = latest_part.parse::<u32>().unwrap_or(0);

        if latest_num > current_num {
            return true;
        } else if latest_num < current_num {
            return false;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("0.1.0", "0.2.0"));
        assert!(compare_versions("0.1.0", "1.0.0"));
        assert!(compare_versions("1.0.0", "1.0.1"));
        assert!(!compare_versions("1.0.0", "1.0.0"));
        assert!(!compare_versions("1.0.1", "1.0.0"));
        assert!(!compare_versions("2.0.0", "1.9.9"));
    }
}
