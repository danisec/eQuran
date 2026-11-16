//! Prepares the on-device Supertonic runtime without root.
//!
//! The packaged setup script writes its venv next to itself, but after an
//! install that directory belongs to root (or to a protected per-machine
//! location on Windows), so a normal user cannot write there. The app points
//! `VENV_DIR` at its own data directory instead — a location it already
//! searches for the venv interpreter — so a reader can enable the voice from
//! Settings without elevation, and the installed package stays untouched.
//!
//! The script and its launcher are platform-specific: POSIX bundles `setup.sh`
//! run through `bash`, Windows bundles `setup.ps1` run through Windows
//! PowerShell (which ships with every supported Windows release).

use std::path::{Path, PathBuf};

/// The bundled setup script and the writable venv it should populate.
///
/// Resolution has two independent halves: the script comes from the resource
/// (or checkout) that owns it, while the venv lives under a root that is
/// writable at runtime. Keeping them separate is what lets an installed
/// package be prepared by an unprivileged process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSetup {
    script: PathBuf,
    venv_dir: PathBuf,
}

impl RuntimeSetup {
    /// Builds the plan when the platform's setup script exists under
    /// `project_roots`.
    ///
    /// `writable_root` is where the venv is created; it becomes the last entry
    /// of the app's search roots, so the interpreter the script builds there is
    /// the one later discovery finds. Returns `None` when no script is bundled,
    /// which is the signal that there is nothing the app can run.
    pub fn discover(project_roots: &[PathBuf], writable_root: &Path) -> Option<Self> {
        let script = project_roots
            .iter()
            .map(|root| root.join("tts").join(setup_script_name()))
            .find(|path| path.is_file())?;
        Some(Self {
            script,
            venv_dir: writable_root.join("tts").join(".venv"),
        })
    }

    /// The bundled setup script that does the actual install.
    pub fn script(&self) -> &Path {
        &self.script
    }

    /// Builds a plan directly, for callers that already resolved both paths.
    pub fn new(script: PathBuf, venv_dir: PathBuf) -> Self {
        Self { script, venv_dir }
    }

    /// The venv the script must create, which discovery later resolves.
    pub fn venv_dir(&self) -> &Path {
        &self.venv_dir
    }

    /// Runs the bundled script with the venv redirected to the writable root.
    ///
    /// `VENV_DIR` is the whole point: the script otherwise writes next to
    /// itself, which fails once the package lives in a protected directory.
    /// stdout/stderr are piped so the caller can stream install progress.
    pub fn command(&self) -> tokio::process::Command {
        let mut command = shell_command(&self.script);
        command
            .env("VENV_DIR", &self.venv_dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        command
    }
}

/// The setup script bundled for the current platform: `setup.sh` on POSIX,
/// `setup.ps1` on Windows.
pub fn setup_script_name() -> &'static str {
    if cfg!(windows) {
        "setup.ps1"
    } else {
        "setup.sh"
    }
}

/// Launches the bundled setup script with the shell that can actually run it.
///
/// `-ExecutionPolicy Bypass` is scoped to this single invocation so a default
/// Windows machine (Restricted policy) can still run the unsigned script the
/// installer shipped. `-NoProfile` keeps an unrelated PowerShell profile from
/// injecting into the install.
#[cfg(windows)]
fn shell_command(script: &Path) -> tokio::process::Command {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut command = tokio::process::Command::new("powershell.exe");
    command
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(script);
    // The app is a GUI subsystem binary, so a child console process would
    // otherwise flash a terminal window during the (long) install.
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(windows))]
fn shell_command(script: &Path) -> tokio::process::Command {
    let mut command = tokio::process::Command::new("bash");
    command.arg(script);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        std::fs::write(path, "").expect("file");
    }

    #[test]
    fn discover_uses_the_first_root_that_bundles_the_script() {
        let base = tempfile::tempdir().expect("tempdir");
        let resource = base.path().join("resource");
        let checkout = base.path().join("checkout");
        touch(&resource.join("tts").join(setup_script_name()));
        touch(&checkout.join("tts").join(setup_script_name()));
        let writable = base.path().join("app-data");

        let setup = RuntimeSetup::discover(&[resource.clone(), checkout], &writable)
            .expect("script should be found");

        assert_eq!(
            setup.script(),
            resource.join("tts").join(setup_script_name())
        );
        // The venv always lands under the writable root, never the resource.
        assert_eq!(setup.venv_dir(), writable.join("tts").join(".venv"));
    }

    #[test]
    fn discover_returns_none_without_a_bundled_script() {
        let base = tempfile::tempdir().expect("tempdir");
        let roots = vec![base.path().join("resource"), base.path().join("checkout")];

        assert!(RuntimeSetup::discover(&roots, base.path()).is_none());
    }
}
