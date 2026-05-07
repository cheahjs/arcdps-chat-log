use arc_util::settings::HasSettings;
use arcdps::imgui::{Condition, StyleColor, Ui};
use log::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

const GITHUB_REPO: &str = "cheahjs/arcdps-chat-log";
const RELEASES_URL: &str = "https://github.com/cheahjs/arcdps-chat-log/releases/latest";
const USER_AGENT: &str = "arcdps-chat-log";

/// Version represented as [major, minor, patch, build]
pub type Version = [u16; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Unknown,
    UpdateAvailable,
    Dismissed,
    UpdateInProgress,
    UpdateSuccessful,
    UpdateError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateSettings {
    /// Whether to check for updates at all on startup.
    pub check_enabled: bool,
    /// Whether the update check should consider prereleases.
    pub include_prereleases: bool,
}

impl UpdateSettings {
    pub fn new() -> Self {
        Self {
            check_enabled: true,
            include_prereleases: false,
        }
    }
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct UpdateState {
    pub current_version: Option<Version>,
    pub install_path: PathBuf,
    pub settings: UpdateSettings,
    status: Arc<Mutex<UpdateStatus>>,
    pub new_version: Arc<Mutex<Option<Version>>>,
    pub download_url: Arc<Mutex<Option<String>>>,
    pub error_message: Arc<Mutex<Option<String>>>,
    tasks: Vec<JoinHandle<()>>,
}

impl HasSettings for UpdateState {
    type Settings = UpdateSettings;

    const SETTINGS_ID: &'static str = "update";

    fn current_settings(&self) -> Self::Settings {
        self.settings.clone()
    }

    fn load_settings(&mut self, loaded: Self::Settings) {
        self.settings = loaded;
    }
}

impl UpdateState {
    pub fn new(current_version: Option<Version>, install_path: PathBuf) -> Self {
        Self {
            current_version,
            install_path,
            settings: UpdateSettings::new(),
            status: Arc::new(Mutex::new(UpdateStatus::Unknown)),
            new_version: Arc::new(Mutex::new(None)),
            download_url: Arc::new(Mutex::new(None)),
            error_message: Arc::new(Mutex::new(None)),
            tasks: Vec::new(),
        }
    }

    pub fn status(&self) -> UpdateStatus {
        *self.status.lock().unwrap()
    }

    pub fn set_status(&self, status: UpdateStatus) {
        *self.status.lock().unwrap() = status;
    }

    pub fn finish_pending_tasks(&mut self) {
        for task in self.tasks.drain(..) {
            if let Err(e) = task.join() {
                error!("error joining update task: {:?}", e);
            }
        }
    }
}

pub fn get_current_version() -> Version {
    parse_version(env!("CARGO_PKG_VERSION")).unwrap_or([0, 0, 0, 0])
}

/// Get the path to the current DLL by querying the module containing this function.
pub fn get_dll_path() -> Option<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::{
        GetModuleHandleExW, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    };

    unsafe {
        let mut module: HMODULE = HMODULE::default();
        let self_addr = get_dll_path as *const ();

        if GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            windows::core::PCWSTR::from_raw(self_addr as *const u16),
            &mut module,
        )
        .is_err()
        {
            error!("GetModuleHandleExW failed");
            return None;
        }

        let mut path_buffer = [0u16; 260];
        let path_len = GetModuleFileNameW(Some(module), &mut path_buffer);
        if path_len == 0 {
            error!("GetModuleFileNameW failed");
            return None;
        }

        let path_str = String::from_utf16_lossy(&path_buffer[..path_len as usize]);
        Some(PathBuf::from(path_str))
    }
}

/// Clear leftover update files from a previous run.
pub fn clear_old_files(dll_path: &Path) {
    for ext in ["dll.tmp", "dll.old"] {
        let p = dll_path.with_extension(ext);
        if let Err(e) = fs::remove_file(&p) {
            if e.kind() != std::io::ErrorKind::NotFound {
                warn!("failed to remove {}: {}", p.display(), e);
            }
        }
    }
}

pub fn is_newer(repo_version: &Version, current_version: &Version) -> bool {
    (current_version[0], current_version[1], current_version[2])
        < (repo_version[0], repo_version[1], repo_version[2])
}

pub fn version_to_string(version: &Version) -> String {
    format!("{}.{}.{}", version[0], version[1], version[2])
}

pub fn parse_version(tag: &str) -> Option<Version> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let parts: Vec<&str> = tag.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let parse_part = |s: &str| -> Option<u16> {
        let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse().ok()
    };
    Some([
        parse_part(parts[0])?,
        parse_part(parts[1])?,
        parse_part(parts[2])?,
        0,
    ])
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
    #[serde(default)]
    #[allow(dead_code)]
    prerelease: bool,
}

pub fn check_for_update(state: &mut UpdateState, include_prereleases: bool) {
    let status = Arc::clone(&state.status);
    let new_version = Arc::clone(&state.new_version);
    let download_url = Arc::clone(&state.download_url);
    let current_version = state.current_version;

    let handle = thread::spawn(move || {
        let url = if include_prereleases {
            format!("https://api.github.com/repos/{}/releases", GITHUB_REPO)
        } else {
            format!(
                "https://api.github.com/repos/{}/releases/latest",
                GITHUB_REPO
            )
        };

        info!("checking for updates at {}", url);

        let response = match minreq::get(&url)
            .with_header("User-Agent", USER_AGENT)
            .with_header("Accept", "application/vnd.github.v3+json")
            .send()
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("failed to fetch releases: {}", e);
                return;
            }
        };

        if !(200..300).contains(&response.status_code) {
            error!(
                "release lookup returned HTTP {} {}: {}",
                response.status_code,
                response.reason_phrase,
                response.as_str().unwrap_or("<non-utf8 body>")
            );
            return;
        }

        let body = match response.as_str() {
            Ok(s) => s,
            Err(e) => {
                error!("failed to read release response body: {}", e);
                return;
            }
        };

        let release: GitHubRelease = if include_prereleases {
            let releases: Vec<GitHubRelease> = match serde_json::from_str(body) {
                Ok(r) => r,
                Err(e) => {
                    error!("failed to parse releases JSON: {}", e);
                    return;
                }
            };
            match releases
                .into_iter()
                .find(|r| r.assets.iter().any(|a| a.name.ends_with(".dll")))
            {
                Some(r) => r,
                None => {
                    error!("no releases with .dll assets found");
                    return;
                }
            }
        } else {
            match serde_json::from_str(body) {
                Ok(r) => r,
                Err(e) => {
                    error!("failed to parse release JSON: {}", e);
                    return;
                }
            }
        };

        let release_version = match parse_version(&release.tag_name) {
            Some(v) => v,
            None => {
                error!("failed to parse version from tag: {}", release.tag_name);
                return;
            }
        };

        if let Some(current) = current_version {
            if !is_newer(&release_version, &current) {
                info!(
                    "current version {} is up to date (latest: {})",
                    version_to_string(&current),
                    version_to_string(&release_version)
                );
                return;
            }
        }

        let dll_url = match release
            .assets
            .iter()
            .find(|a| a.name.ends_with(".dll"))
            .map(|a| a.browser_download_url.clone())
        {
            Some(u) => u,
            None => {
                error!("no .dll asset found in release");
                return;
            }
        };

        info!(
            "update available: {} -> {} ({})",
            current_version.map_or("unknown".to_string(), |v| version_to_string(&v)),
            version_to_string(&release_version),
            dll_url
        );

        *new_version.lock().unwrap() = Some(release_version);
        *download_url.lock().unwrap() = Some(dll_url);
        *status.lock().unwrap() = UpdateStatus::UpdateAvailable;
    });

    state.tasks.push(handle);
}

pub fn perform_update(state: &mut UpdateState) {
    if state.status() != UpdateStatus::UpdateAvailable {
        warn!("cannot perform update: status is {:?}", state.status());
        return;
    }

    state.set_status(UpdateStatus::UpdateInProgress);

    let install_path = state.install_path.clone();
    let download_url = state.download_url.lock().unwrap().clone();
    let status = Arc::clone(&state.status);
    let error_message = Arc::clone(&state.error_message);

    let Some(url) = download_url else {
        error!("no download URL available");
        *status.lock().unwrap() = UpdateStatus::UpdateError;
        *error_message.lock().unwrap() = Some("no download URL available".to_string());
        return;
    };

    let handle = thread::spawn(move || {
        let tmp_path = install_path.with_extension("dll.tmp");
        let old_path = install_path.with_extension("dll.old");

        info!("downloading update from {}", url);

        let response = match minreq::get(&url)
            .with_header("User-Agent", USER_AGENT)
            .send()
        {
            Ok(resp) => resp,
            Err(e) => {
                error!("failed to download update: {}", e);
                *status.lock().unwrap() = UpdateStatus::UpdateError;
                *error_message.lock().unwrap() = Some(format!("download failed: {}", e));
                return;
            }
        };

        // GitHub asset URLs follow a redirect to the CDN; minreq follows it transparently
        // but we still need to verify we got a 2xx so we don't write an error page over the DLL.
        if !(200..300).contains(&response.status_code) {
            let msg = format!(
                "download returned HTTP {} {}",
                response.status_code, response.reason_phrase
            );
            error!("{}", msg);
            *status.lock().unwrap() = UpdateStatus::UpdateError;
            *error_message.lock().unwrap() = Some(msg);
            return;
        }

        let mut file = match fs::File::create(&tmp_path) {
            Ok(f) => f,
            Err(e) => {
                error!("failed to create temp file: {}", e);
                *status.lock().unwrap() = UpdateStatus::UpdateError;
                *error_message.lock().unwrap() = Some(format!("failed to create temp file: {}", e));
                return;
            }
        };

        if let Err(e) = file.write_all(response.as_bytes()) {
            error!("failed to write to temp file: {}", e);
            *status.lock().unwrap() = UpdateStatus::UpdateError;
            *error_message.lock().unwrap() = Some(format!("failed to write temp file: {}", e));
            return;
        }
        drop(file);

        if let Err(e) = fs::rename(&install_path, &old_path) {
            error!(
                "failed to rename {} to {}: {}",
                install_path.display(),
                old_path.display(),
                e
            );
            *status.lock().unwrap() = UpdateStatus::UpdateError;
            *error_message.lock().unwrap() = Some(format!("failed to backup current file: {}", e));
            return;
        }

        if let Err(e) = fs::rename(&tmp_path, &install_path) {
            error!(
                "failed to rename {} to {}: {}",
                tmp_path.display(),
                install_path.display(),
                e
            );
            let _ = fs::rename(&old_path, &install_path);
            *status.lock().unwrap() = UpdateStatus::UpdateError;
            *error_message.lock().unwrap() = Some(format!("failed to install new file: {}", e));
            return;
        }

        info!("update completed successfully");
        *status.lock().unwrap() = UpdateStatus::UpdateSuccessful;
    });

    state.tasks.push(handle);
}

pub fn draw_update_window(ui: &Ui, state: &mut UpdateState) {
    let status = state.status();

    if status == UpdateStatus::Unknown || status == UpdateStatus::Dismissed {
        return;
    }

    let mut open = true;

    ui.window("Chat Log Update")
        .opened(&mut open)
        .collapsible(false)
        .always_auto_resize(true)
        .size([350.0, 0.0], Condition::FirstUseEver)
        .build(|| {
            if let Some(current) = state.current_version {
                let _red = ui.push_style_color(StyleColor::Text, [1.0, 0.3, 0.3, 1.0]);
                ui.text("A new version of Chat Log is available!");
                ui.text(format!("Current version: {}", version_to_string(&current)));
            }

            if let Some(new_ver) = *state.new_version.lock().unwrap() {
                let _green = ui.push_style_color(StyleColor::Text, [0.3, 1.0, 0.3, 1.0]);
                ui.text(format!("New version: {}", version_to_string(&new_ver)));
            }

            ui.separator();

            if ui.button("Open Releases Page") {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", RELEASES_URL])
                    .spawn();
            }

            match status {
                UpdateStatus::UpdateAvailable => {
                    ui.same_line();
                    if ui.button("Install Update") {
                        perform_update(state);
                    }
                }
                UpdateStatus::UpdateInProgress => {
                    ui.text("Downloading update...");
                }
                UpdateStatus::UpdateSuccessful => {
                    let _green = ui.push_style_color(StyleColor::Text, [0.3, 1.0, 0.3, 1.0]);
                    ui.text("Update successful! Restart the game to apply.");
                }
                UpdateStatus::UpdateError => {
                    let _red = ui.push_style_color(StyleColor::Text, [1.0, 0.3, 0.3, 1.0]);
                    if let Some(ref msg) = *state.error_message.lock().unwrap() {
                        ui.text(format!("Update failed: {}", msg));
                    } else {
                        ui.text("Update failed!");
                    }
                }
                _ => {}
            }
        });

    if !open && status != UpdateStatus::UpdateInProgress {
        state.set_status(UpdateStatus::Dismissed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some([1, 2, 3, 0]));
        assert_eq!(parse_version("v1.2.3"), Some([1, 2, 3, 0]));
        assert_eq!(parse_version("v10.20.30"), Some([10, 20, 30, 0]));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3-beta"), Some([1, 2, 3, 0]));
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer(&[2, 0, 0, 0], &[1, 0, 0, 0]));
        assert!(is_newer(&[1, 1, 0, 0], &[1, 0, 0, 0]));
        assert!(is_newer(&[1, 0, 1, 0], &[1, 0, 0, 0]));
        assert!(!is_newer(&[1, 0, 0, 0], &[1, 0, 0, 0]));
        assert!(!is_newer(&[1, 0, 0, 0], &[2, 0, 0, 0]));
    }

    #[test]
    fn test_version_to_string() {
        assert_eq!(version_to_string(&[1, 2, 3, 0]), "1.2.3");
        assert_eq!(version_to_string(&[10, 20, 30, 0]), "10.20.30");
    }
}
