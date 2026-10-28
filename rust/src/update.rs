//! Update check and explicit `.deb` install.
//!
//! [`qobject::Updater::check_for_updates`] only reads GitHub and fills the dialog.
//! [`qobject::Updater::install_update`] is the only path that downloads or
//! installs, and it returns without doing either unless a newer trusted package
//! is waiting from a check the user already ran. Neither function is called
//! from `main` or at startup.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, phase)]
        #[qproperty(QString, current_version)]
        #[qproperty(QString, latest_version)]
        #[qproperty(QString, message)]
        #[qproperty(bool, update_available)]
        #[qproperty(bool, busy)]
        #[namespace = "pdfpc"]
        type Updater = super::UpdaterRust;

        /// Read GitHub releases and tags. Does not download or install.
        #[qinvokable]
        #[cxx_name = "checkForUpdates"]
        fn check_for_updates(self: Pin<&mut Self>);

        /// Download and install the package found by the last check.
        /// No-op unless the user invoked this while an update is waiting.
        #[qinvokable]
        #[cxx_name = "installUpdate"]
        fn install_update(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for Updater {}
}

use cxx_qt::CxxQtType;
use cxx_qt::Threading;
use cxx_qt_lib::QString;
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;

const PROJECT_REPO: &str = "tedeler/pdfpc_qt6";
const RELEASES_LATEST: &str = "https://api.github.com/repos/tedeler/pdfpc_qt6/releases/latest";
const TAGS_URL: &str = "https://api.github.com/repos/tedeler/pdfpc_qt6/tags?per_page=100";
const DOWNLOAD_PREFIX: &str = "https://github.com/tedeler/pdfpc_qt6/releases/download/";
const MAX_DEB_BYTES: u64 = 64 * 1024 * 1024;

pub struct UpdaterRust {
    phase: QString,
    current_version: QString,
    latest_version: QString,
    message: QString,
    update_available: bool,
    busy: bool,
    /// Set only after a check finds a newer trusted package. Not a QML property.
    pending: Option<PendingUpdate>,
    job: u64,
}

impl Default for UpdaterRust {
    fn default() -> Self {
        Self {
            phase: QString::from("idle"),
            current_version: QString::from(env!("PDFPC_VERSION")),
            latest_version: QString::default(),
            message: QString::default(),
            update_available: false,
            busy: false,
            pending: None,
            job: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingUpdate {
    pub version: String,
    pub tag: String,
    pub deb_name: String,
    pub deb_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebAsset {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseFound {
    pub version: String,
    pub tag: String,
    pub deb: Option<DebAsset>,
    pub rejected_untrusted_url: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckOutcome {
    pub current: String,
    pub latest: String,
    pub message: String,
    pub pending: Option<PendingUpdate>,
    pub failed: bool,
}

impl CheckOutcome {
    fn failed(current: &str, latest: &str, message: impl Into<String>) -> Self {
        Self {
            current: current.to_string(),
            latest: latest.to_string(),
            message: message.into(),
            pending: None,
            failed: true,
        }
    }

    pub fn allows_install(&self) -> bool {
        self.pending.is_some() && !self.failed
    }
}

pub fn debian_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    }
}

pub fn parse_version(raw: &str) -> Option<Vec<u64>> {
    if raw.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for part in raw.split('.') {
        if part.is_empty() || part.len() > 6 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        parts.push(part.parse().ok()?);
    }
    if parts.is_empty() || parts.len() > 4 {
        None
    } else {
        Some(parts)
    }
}

/// `v1.05` / `V1.05` / `1.05` → `1.05`. A single optional prefix only.
pub fn version_from_tag(tag: &str) -> Option<String> {
    let raw = tag.trim();
    let version = raw
        .strip_prefix('v')
        .or_else(|| raw.strip_prefix('V'))
        .unwrap_or(raw);
    parse_version(version)?;
    Some(version.to_string())
}

pub fn cmp_versions(a: &[u64], b: &[u64]) -> Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

pub fn expected_deb_name(version: &str, arch: &str) -> String {
    format!("pdfpc-qt6_{version}_{arch}.deb")
}

pub fn expected_deb_url(tag: &str, deb_name: &str) -> String {
    format!("{DOWNLOAD_PREFIX}{tag}/{deb_name}")
}

pub fn safe_deb_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("pdfpc-qt6_") else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(".deb") else {
        return false;
    };
    let Some((version, arch)) = rest.rsplit_once('_') else {
        return false;
    };
    (arch == "amd64" || arch == "arm64") && parse_version(version).is_some()
}

pub fn is_trusted_deb_url(url: &str, tag: &str, deb_name: &str) -> bool {
    !tag.contains('/')
        && !tag.contains('\\')
        && !deb_name.contains('/')
        && version_from_tag(tag).is_some()
        && safe_deb_name(deb_name)
        && url == expected_deb_url(tag, deb_name)
        && url.starts_with(DOWNLOAD_PREFIX)
        && url.contains(PROJECT_REPO)
}

pub fn install_allowed(current: &str, pending: &PendingUpdate) -> Result<(), String> {
    let current_v = parse_version(current.trim())
        .ok_or_else(|| "The running version number is not recognized.".to_string())?;
    let latest_v = parse_version(&pending.version)
        .ok_or_else(|| "The release version is not recognized.".to_string())?;
    if cmp_versions(&latest_v, &current_v) != Ordering::Greater {
        return Err("No newer version to install.".into());
    }
    let arch = arch_of_deb_name(&pending.deb_name)
        .ok_or_else(|| "Refusing to install this package.".to_string())?;
    if pending.deb_name != expected_deb_name(&pending.version, arch) {
        return Err("Refusing to install this package.".into());
    }
    if !is_trusted_deb_url(&pending.deb_url, &pending.tag, &pending.deb_name) {
        return Err("Refusing to download this file.".into());
    }
    Ok(())
}

fn arch_of_deb_name(name: &str) -> Option<&str> {
    let rest = name.strip_prefix("pdfpc-qt6_")?.strip_suffix(".deb")?;
    let (_, arch) = rest.rsplit_once('_')?;
    if arch == "amd64" || arch == "arm64" {
        Some(arch)
    } else {
        None
    }
}

pub fn parse_latest_release(body: &str, arch: &str) -> Result<ReleaseFound, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| "GitHub returned data that is not JSON.".to_string())?;
    if let Some(msg) = github_message(&value) {
        if value.get("tag_name").is_none() {
            return Err(msg);
        }
    }
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let version =
        version_from_tag(&tag).ok_or_else(|| format!("Unrecognized release tag '{tag}'."))?;
    let deb_name = expected_deb_name(&version, arch);
    let expected = expected_deb_url(&tag, &deb_name);
    let mut deb = None;
    let mut rejected = false;
    if let Some(assets) = value.get("assets").and_then(|v| v.as_array()) {
        for asset in assets {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let url = asset
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name != deb_name {
                continue;
            }
            if url == expected && is_trusted_deb_url(url, &tag, name) {
                deb = Some(DebAsset {
                    name: name.to_string(),
                    url: url.to_string(),
                });
                rejected = false;
                break;
            }
            rejected = true;
        }
    }
    let rejected_untrusted_url = rejected && deb.is_none();
    Ok(ReleaseFound {
        version,
        tag,
        deb,
        rejected_untrusted_url,
    })
}

pub fn newest_tag_version(body: &str) -> Result<Option<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| "GitHub returned data that is not JSON.".to_string())?;
    if let Some(msg) = github_message(&value) {
        return Err(msg);
    }
    let tags = value
        .as_array()
        .ok_or_else(|| "GitHub tag list has an unexpected shape.".to_string())?;
    let mut best: Option<(Vec<u64>, String)> = None;
    for tag in tags {
        let name = tag.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let Some(version) = version_from_tag(name) else {
            continue;
        };
        let Some(parts) = parse_version(&version) else {
            continue;
        };
        let replace = match &best {
            None => true,
            Some((have, _)) => cmp_versions(&parts, have) == Ordering::Greater,
        };
        if replace {
            best = Some((parts, version));
        }
    }
    Ok(best.map(|(_, version)| version))
}

fn github_message(value: &Value) -> Option<String> {
    let msg = value.get("message")?.as_str()?.trim();
    if msg.is_empty() {
        return None;
    }
    if msg.to_ascii_lowercase().contains("rate limit") {
        Some("GitHub rate limit reached. Try again in a few minutes.".into())
    } else {
        Some(format!("GitHub: {msg}"))
    }
}

pub fn evaluate(
    current_raw: &str,
    release: Option<&ReleaseFound>,
    newest_tag: Option<&str>,
    arch: &str,
) -> CheckOutcome {
    let current_raw = current_raw.trim();
    let Some(current) = parse_version(current_raw) else {
        return CheckOutcome::failed(
            current_raw,
            "",
            "The running version number is not recognized.",
        );
    };

    let release_parts = release.and_then(|r| parse_version(&r.version));
    let tag_version = newest_tag
        .and_then(|t| version_from_tag(t).or_else(|| parse_version(t).map(|_| t.to_string())));
    let tag_parts = tag_version.as_deref().and_then(parse_version);

    let latest = match (
        release,
        tag_version.as_deref(),
        release_parts.as_ref(),
        tag_parts.as_ref(),
    ) {
        (Some(r), Some(tag), Some(rv), Some(tv)) => {
            if cmp_versions(tv, rv) == Ordering::Greater {
                tag.to_string()
            } else {
                r.version.clone()
            }
        }
        (Some(r), _, _, _) => r.version.clone(),
        (None, Some(tag), _, _) => tag.to_string(),
        _ => String::new(),
    };
    if latest.is_empty() {
        return CheckOutcome::failed(
            current_raw,
            "",
            "No GitHub release or version tag was found.",
        );
    }
    let Some(latest_parts) = parse_version(&latest) else {
        return CheckOutcome::failed(
            current_raw,
            "",
            "The latest version number is not recognized.",
        );
    };

    let pending = release.and_then(|r| trusted_pending(r, &current));
    let release_is_newer = release_parts
        .as_ref()
        .is_some_and(|rv| cmp_versions(rv, &current) == Ordering::Greater);
    let tag_ahead_of_release = match (release_parts.as_ref(), tag_parts.as_ref()) {
        (Some(rv), Some(tv)) => cmp_versions(tv, rv) == Ordering::Greater,
        (None, Some(tv)) => cmp_versions(tv, &current) == Ordering::Greater,
        _ => false,
    };

    let message = if let Some(pending) = &pending {
        if tag_ahead_of_release {
            format!(
                "The latest version is {latest}. The newest package that can be installed is {pkg}. Nothing is downloaded or installed until you click Install and restart.",
                pkg = pending.version
            )
        } else {
            format!(
                "Version {ver} is available (this copy is {current_raw}). Nothing is downloaded or installed until you click Install and restart.",
                ver = pending.version
            )
        }
    } else if cmp_versions(&latest_parts, &current) == Ordering::Greater {
        if release_is_newer {
            if release.is_some_and(|r| r.rejected_untrusted_url) {
                format!(
                    "Version {latest} is available, but its package URL is not the official GitHub download. Nothing will be installed."
                )
            } else {
                format!(
                    "Version {latest} is available, but the release has no pdfpc-qt6 package for {arch}."
                )
            }
        } else {
            format!("The latest version is {latest}. There is no newer installable package yet.")
        }
    } else if cmp_versions(&latest_parts, &current) == Ordering::Less {
        format!("This build ({current_raw}) is newer than the latest GitHub version ({latest}).")
    } else {
        format!("You are up to date. The current version and the latest version are both {current_raw}.")
    };

    CheckOutcome {
        current: current_raw.to_string(),
        latest,
        message,
        pending,
        failed: false,
    }
}

fn trusted_pending(release: &ReleaseFound, current: &[u64]) -> Option<PendingUpdate> {
    let parts = parse_version(&release.version)?;
    if cmp_versions(&parts, current) != Ordering::Greater {
        return None;
    }
    let deb = release.deb.as_ref()?;
    if !is_trusted_deb_url(&deb.url, &release.tag, &deb.name) {
        return None;
    }
    let arch = arch_of_deb_name(&deb.name)?;
    if deb.name != expected_deb_name(&release.version, arch) {
        return None;
    }
    Some(PendingUpdate {
        version: release.version.clone(),
        tag: release.tag.clone(),
        deb_name: deb.name.clone(),
        deb_url: deb.url.clone(),
    })
}

pub fn install_argv(deb: &Path, as_root: bool) -> Vec<String> {
    let deb = deb.to_string_lossy().into_owned();
    if as_root {
        vec![
            "/usr/bin/apt-get".into(),
            "install".into(),
            "-y".into(),
            deb,
        ]
    } else {
        // pkexec drops the environment, so debconf's frontend is an argument.
        vec![
            "/usr/bin/pkexec".into(),
            "/usr/bin/env".into(),
            "DEBIAN_FRONTEND=noninteractive".into(),
            "/usr/bin/apt-get".into(),
            "install".into(),
            "-y".into(),
            deb,
        ]
    }
}

fn api_url_allowed(url: &str) -> bool {
    url == RELEASES_LATEST || url == TAGS_URL
}

fn http_get(url: &str) -> Result<String, String> {
    if !api_url_allowed(url) {
        return Err("Refusing to query this URL.".into());
    }
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "2",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-time",
            "20",
            "-A",
            "pdfpc-qt6",
            "-H",
            "Accept: application/vnd.github+json",
            "--",
        ])
        .arg(url)
        .output()
        .map_err(|e| format!("Could not run curl ({e}). Is curl installed?"))?;
    let body = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        return Ok(body);
    }
    if let Ok(value) = serde_json::from_str::<Value>(&body) {
        if let Some(msg) = github_message(&value) {
            return Err(msg);
        }
    }
    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if err.is_empty() {
        Err("Could not reach GitHub.".into())
    } else {
        Err(format!("Could not reach GitHub ({err})."))
    }
}

fn run_check(current: &str, arch: &str) -> CheckOutcome {
    let release_body = http_get(RELEASES_LATEST);
    let tags_body = http_get(TAGS_URL);
    let release = match &release_body {
        Ok(body) => match parse_latest_release(body, arch) {
            Ok(found) => Some(found),
            Err(err) => {
                if tags_body.is_err() {
                    return CheckOutcome::failed(current, "", err);
                }
                None
            }
        },
        Err(err) => {
            if let Err(tags_err) = &tags_body {
                return CheckOutcome::failed(current, "", format!("{err} {tags_err}"));
            }
            None
        }
    };
    let newest_tag = match &tags_body {
        Ok(body) => newest_tag_version(body).ok().flatten(),
        Err(_) => None,
    };
    if release.is_none() && newest_tag.is_none() {
        return CheckOutcome::failed(current, "", "No GitHub release or version tag was found.");
    }
    evaluate(current, release.as_ref(), newest_tag.as_deref(), arch)
}

fn current_uid() -> u32 {
    extern "C" {
        fn geteuid() -> u32;
    }
    unsafe { geteuid() }
}

fn update_dir() -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("pdfpc-qt6-update-{}", current_uid()));
    dir
}

fn prepare_dest(deb_name: &str) -> Result<PathBuf, String> {
    if !safe_deb_name(deb_name) {
        return Err("Refusing to save this package name.".into());
    }
    let dir = update_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Could not create a temporary directory ({e})."))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    let dest = dir.join(deb_name);
    if dest.parent() != Some(dir.as_path()) {
        return Err("Refusing to save the package outside the temporary directory.".into());
    }
    let _ = fs::remove_file(&dest);
    Ok(dest)
}

fn download_deb(url: &str, dest: &Path) -> Result<(), String> {
    if !url.starts_with(DOWNLOAD_PREFIX) {
        return Err("Refusing to download this file.".into());
    }
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "2",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--max-time",
            "120",
            "--max-filesize",
            "67108864",
            "-A",
            "pdfpc-qt6",
            "--output",
        ])
        .arg(dest)
        .arg("--")
        .arg(url)
        .output()
        .map_err(|e| format!("Could not run curl ({e}). Is curl installed?"))?;
    if !output.status.success() {
        let _ = fs::remove_file(dest);
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err.is_empty() {
            return Err("Could not download the package.".into());
        }
        return Err(format!("Could not download the package ({err})."));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dest, fs::Permissions::from_mode(0o600));
    }
    if let Err(err) = verify_deb_file(dest) {
        let _ = fs::remove_file(dest);
        return Err(err);
    }
    Ok(())
}

fn verify_deb_file(path: &Path) -> Result<(), String> {
    let meta = fs::metadata(path).map_err(|e| format!("Downloaded package is missing ({e})."))?;
    if meta.len() == 0 || meta.len() > MAX_DEB_BYTES {
        return Err("Downloaded package has an unexpected size.".into());
    }
    let mut file =
        fs::File::open(path).map_err(|e| format!("Could not read the download ({e})."))?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)
        .map_err(|_| "Downloaded file is not a Debian package.".to_string())?;
    if &magic != b"!<arch>\n" {
        return Err("Downloaded file is not a Debian package.".into());
    }
    Ok(())
}

fn install_deb(dest: &Path) -> Result<(), String> {
    verify_deb_file(dest)?;
    let name = dest.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if !safe_deb_name(name) || dest.parent() != Some(update_dir().as_path()) {
        return Err("Refusing to install this file.".into());
    }
    let as_root = current_uid() == 0;
    if !as_root && !Path::new("/usr/bin/pkexec").is_file() {
        return Err(
            "pkexec is not installed, so the package cannot be installed from here.".into(),
        );
    }
    if !Path::new("/usr/bin/apt-get").is_file() {
        return Err(
            "apt-get is not installed, so the package cannot be installed from here.".into(),
        );
    }
    let argv = install_argv(dest, as_root);
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    if as_root {
        cmd.env("DEBIAN_FRONTEND", "noninteractive");
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Could not start the installer ({e})."))?;
    if output.status.success() {
        return Ok(());
    }
    let code = output.status.code();
    let detail = command_tail(&output.stderr, &output.stdout);
    if code == Some(126) || code == Some(127) {
        return Err(format!(
            "Installation was cancelled or not authorized. Nothing was changed. {detail}"
        ));
    }
    Err(format!("Installation failed. {detail}"))
}

fn command_tail(stderr: &[u8], stdout: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(stderr).trim().to_string();
    if text.is_empty() {
        text = String::from_utf8_lossy(stdout).trim().to_string();
    }
    const MAX: usize = 400;
    if text.chars().count() > MAX {
        text = text.chars().take(MAX).collect();
        text.push('…');
    }
    if text.is_empty() {
        "The installer exited with an error.".into()
    } else {
        text
    }
}

fn relaunch() -> Result<(), String> {
    let bin = Path::new("/usr/bin/pdfpc-qt6");
    if !bin.is_file() {
        return Err(
            "The package installed, but /usr/bin/pdfpc-qt6 is missing. Start it from the menu."
                .into(),
        );
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    Command::new(bin)
        .args(args)
        .spawn()
        .map_err(|e| format!("The update is installed, but pdfpc-qt6 could not restart ({e})."))?;
    std::process::exit(0);
}

fn report_install_error(qt: &cxx_qt::CxxQtThread<qobject::Updater>, job: u64, err: String) {
    eprintln!("pdfpc: update install failed: {err}");
    let _ = qt.queue(move |mut obj| {
        if obj.rust().job != job {
            return;
        }
        obj.as_mut().set_busy(false);
        obj.as_mut().set_phase(QString::from("error"));
        obj.as_mut().set_message(QString::from(&err));
    });
}

impl qobject::Updater {
    pub fn check_for_updates(mut self: Pin<&mut Self>) {
        if *self.busy() {
            return;
        }
        let job = {
            let mut rust = self.as_mut().rust_mut();
            rust.job = rust.job.wrapping_add(1);
            rust.pending = None;
            rust.job
        };
        self.as_mut().set_busy(true);
        self.as_mut().set_update_available(false);
        self.as_mut().set_phase(QString::from("checking"));
        self.as_mut().set_latest_version(QString::default());
        self.as_mut()
            .set_message(QString::from("Checking GitHub releases and tags…"));
        let current = self.current_version().to_string();
        let arch = debian_arch().to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            eprintln!("pdfpc: update check requested");
            let outcome = run_check(&current, &arch);
            let _ = qt.queue(move |mut obj| {
                if obj.rust().job != job {
                    return;
                }
                obj.as_mut().publish(outcome);
            });
        });
    }

    pub fn install_update(mut self: Pin<&mut Self>) {
        if *self.busy() || !*self.update_available() {
            return;
        }
        let pending = self.rust().pending.clone();
        let current = self.current_version().to_string();
        let Some(pending) = pending else {
            self.as_mut().set_update_available(false);
            self.as_mut()
                .set_message(QString::from("No update is ready to install."));
            return;
        };
        if let Err(err) = install_allowed(&current, &pending) {
            self.as_mut().set_update_available(false);
            self.as_mut().rust_mut().pending = None;
            self.as_mut().set_phase(QString::from("error"));
            self.as_mut().set_message(QString::from(&err));
            return;
        }
        let job = {
            let mut rust = self.as_mut().rust_mut();
            rust.job = rust.job.wrapping_add(1);
            rust.job
        };
        self.as_mut().set_busy(true);
        self.as_mut().set_phase(QString::from("downloading"));
        self.as_mut().set_message(QString::from(&format!(
            "Downloading pdfpc-qt6 {}…",
            pending.version
        )));
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            eprintln!("pdfpc: install requested by user for {}", pending.version);
            let dest = match prepare_dest(&pending.deb_name) {
                Ok(path) => path,
                Err(err) => {
                    report_install_error(&qt, job, err);
                    return;
                }
            };
            if let Err(err) = install_allowed(&current, &pending) {
                report_install_error(&qt, job, err);
                return;
            }
            if let Err(err) = download_deb(&pending.deb_url, &dest) {
                report_install_error(&qt, job, err);
                return;
            }
            let _ = qt.queue({
                let version = pending.version.clone();
                move |mut obj| {
                    if obj.rust().job != job {
                        return;
                    }
                    obj.as_mut().set_phase(QString::from("installing"));
                    obj.as_mut().set_message(QString::from(&format!(
                        "Installing pdfpc-qt6 {version}… Confirm the authorization dialog. pdfpc-qt6 restarts afterwards."
                    )));
                }
            });
            if let Err(err) = install_deb(&dest) {
                let _ = fs::remove_file(&dest);
                report_install_error(&qt, job, err);
                return;
            }
            let _ = fs::remove_file(&dest);
            eprintln!("pdfpc: update installed, restarting");
            if let Err(err) = relaunch() {
                report_install_error(&qt, job, err);
            }
        });
    }

    fn publish(mut self: Pin<&mut Self>, outcome: CheckOutcome) {
        let allows = outcome.allows_install();
        eprintln!(
            "pdfpc: update check current={} latest={} install={}",
            outcome.current,
            if outcome.latest.is_empty() {
                "—"
            } else {
                &outcome.latest
            },
            if allows { "offered" } else { "not offered" }
        );
        self.as_mut().set_busy(false);
        self.as_mut().set_phase(QString::from(if outcome.failed {
            "error"
        } else {
            "ready"
        }));
        self.as_mut()
            .set_current_version(QString::from(&outcome.current));
        self.as_mut()
            .set_latest_version(QString::from(&outcome.latest));
        self.as_mut().set_message(QString::from(&outcome.message));
        self.as_mut().set_update_available(allows);
        self.as_mut().rust_mut().pending = if allows { outcome.pending } else { None };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_json(tag: &str, version: &str, arch: &str, url: &str) -> String {
        format!(
            r#"{{"tag_name":"{tag}","assets":[{{"name":"pdfpc-qt6_{version}_{arch}.deb","browser_download_url":"{url}"}}]}}"#
        )
    }

    fn trusted_release(version: &str) -> ReleaseFound {
        let tag = format!("v{version}");
        let name = expected_deb_name(version, "amd64");
        let url = expected_deb_url(&tag, &name);
        ReleaseFound {
            version: version.into(),
            tag,
            deb: Some(DebAsset { name, url }),
            rejected_untrusted_url: false,
        }
    }

    #[test]
    fn versions_compare_as_integer_components() {
        assert_eq!(
            cmp_versions(
                &parse_version("1.05").unwrap(),
                &parse_version("1.5").unwrap()
            ),
            Ordering::Equal
        );
        assert_eq!(
            cmp_versions(
                &parse_version("1.10").unwrap(),
                &parse_version("1.05").unwrap()
            ),
            Ordering::Greater
        );
        assert_eq!(
            cmp_versions(
                &parse_version("1.05").unwrap(),
                &parse_version("0.05").unwrap()
            ),
            Ordering::Greater
        );
        assert_eq!(version_from_tag("V0.02").as_deref(), Some("0.02"));
        assert!(parse_version("1.05-rc1").is_none());
        assert!(version_from_tag("vv1.05").is_none());
    }

    #[test]
    fn equal_release_does_not_offer_install() {
        let release = trusted_release("1.05");
        let outcome = evaluate("1.05", Some(&release), Some("1.05"), "amd64");
        assert!(!outcome.allows_install());
        assert!(outcome.pending.is_none());
        assert_eq!(outcome.latest, "1.05");
        assert!(outcome.message.contains("1.05"));
        assert!(outcome.message.contains("up to date"));
    }

    #[test]
    fn newer_release_offers_install_of_the_official_asset_only() {
        let release = trusted_release("1.06");
        let outcome = evaluate("1.05", Some(&release), Some("v1.06"), "amd64");
        assert!(outcome.allows_install());
        let pending = outcome.pending.unwrap();
        assert_eq!(pending.version, "1.06");
        assert_eq!(
            pending.deb_url,
            "https://github.com/tedeler/pdfpc_qt6/releases/download/v1.06/pdfpc-qt6_1.06_amd64.deb"
        );
        assert!(install_allowed("1.05", &pending).is_ok());
        assert!(install_allowed("1.06", &pending).is_err());
        assert!(outcome.message.contains("1.06"));
        assert!(outcome.message.contains("1.05"));
    }

    #[test]
    fn foreign_asset_url_is_not_installable() {
        let body = release_json(
            "v1.06",
            "1.06",
            "amd64",
            "https://evil.example/pdfpc-qt6_1.06_amd64.deb",
        );
        let release = parse_latest_release(&body, "amd64").unwrap();
        assert!(release.deb.is_none());
        assert!(release.rejected_untrusted_url);
        let outcome = evaluate("1.05", Some(&release), None, "amd64");
        assert!(!outcome.allows_install());
        assert_eq!(outcome.latest, "1.06");
        assert!(outcome.message.contains("Nothing will be installed"));
    }

    #[test]
    fn newer_tag_without_package_does_not_offer_install() {
        let release = trusted_release("1.05");
        let outcome = evaluate("1.05", Some(&release), Some("1.06"), "amd64");
        assert!(!outcome.allows_install());
        assert_eq!(outcome.latest, "1.06");
        assert!(outcome.message.contains("no newer installable package"));
    }

    #[test]
    fn newer_tag_still_offers_the_older_published_package() {
        let release = trusted_release("1.05");
        let outcome = evaluate("1.04", Some(&release), Some("1.06"), "amd64");
        assert!(outcome.allows_install());
        assert_eq!(outcome.latest, "1.06");
        assert_eq!(outcome.pending.unwrap().version, "1.05");
    }

    #[test]
    fn picks_the_package_for_this_architecture() {
        let body = r#"{"tag_name":"v1.06","assets":[
            {"name":"pdfpc-qt6_1.06_amd64.deb","browser_download_url":"https://github.com/tedeler/pdfpc_qt6/releases/download/v1.06/pdfpc-qt6_1.06_amd64.deb"},
            {"name":"pdfpc-qt6_1.06_arm64.deb","browser_download_url":"https://github.com/tedeler/pdfpc_qt6/releases/download/v1.06/pdfpc-qt6_1.06_arm64.deb"}
        ]}"#;
        let arm = parse_latest_release(body, "arm64").unwrap();
        assert_eq!(arm.deb.unwrap().name, "pdfpc-qt6_1.06_arm64.deb");
        let none = parse_latest_release(body, "i386").unwrap();
        assert!(none.deb.is_none());
    }

    #[test]
    fn rejects_package_names_that_are_not_a_single_filename() {
        assert!(!safe_deb_name("../pdfpc-qt6_1.05_amd64.deb"));
        assert!(!safe_deb_name("pdfpc-qt6_1.05_amd64.deb;rm.deb"));
        assert!(!safe_deb_name("pdfpc-qt6_1.05_amd64.deb/evil"));
        assert!(safe_deb_name("pdfpc-qt6_1.05_amd64.deb"));
    }

    #[test]
    fn install_command_is_not_a_shell_string() {
        let path = Path::new("/tmp/pdfpc-qt6-update-1/pdfpc-qt6_1.06_amd64.deb");
        let argv = install_argv(path, false);
        assert_eq!(argv[0], "/usr/bin/pkexec");
        assert_eq!(*argv.last().unwrap(), path.to_str().unwrap());
        assert!(argv.iter().all(|arg| !arg.contains(';')));
        let root = install_argv(path, true);
        assert_eq!(root[0], "/usr/bin/apt-get");
        assert!(!root.iter().any(|arg| arg == "/usr/bin/pkexec"));
    }

    #[test]
    fn newest_tag_ignores_names_that_are_not_versions() {
        let body = r#"[{"name":"nightly"},{"name":"V0.02"},{"name":"v1.05"},{"name":"v1.04"}]"#;
        assert_eq!(newest_tag_version(body).unwrap().as_deref(), Some("1.05"));
    }

    #[test]
    fn running_build_ahead_of_github_is_not_an_update() {
        let release = trusted_release("1.05");
        let outcome = evaluate("1.06", Some(&release), Some("1.05"), "amd64");
        assert!(!outcome.allows_install());
        assert_eq!(outcome.current, "1.06");
        assert_eq!(outcome.latest, "1.05");
    }

    #[test]
    fn queries_stay_on_this_repository() {
        assert!(api_url_allowed(RELEASES_LATEST));
        assert!(api_url_allowed(TAGS_URL));
        assert!(!api_url_allowed(
            "https://api.github.com/repos/other/pdfpc_qt6/releases/latest"
        ));
        assert!(PROJECT_REPO.contains("tedeler/pdfpc_qt6"));
    }

    #[test]
    #[ignore]
    fn live_github_release_and_tags() {
        let outcome = run_check("0.01", "amd64");
        assert!(!outcome.failed, "{}", outcome.message);
        assert!(outcome.allows_install(), "{}", outcome.message);
        let pending = outcome.pending.unwrap();
        assert!(install_allowed("0.01", &pending).is_ok());
        assert!(pending.deb_url.starts_with(DOWNLOAD_PREFIX));
        let same = run_check(&pending.version, "amd64");
        assert!(!same.allows_install(), "{}", same.message);
    }
}
