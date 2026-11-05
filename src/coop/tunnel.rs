//! Owned, temporary internet tunnel for development builds. No account, service
//! installation, firewall change or router configuration. Drop stops the helper.
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const HELPER_VERSION: &str = "2026.10.2";
const HELPER_HASH: &str = "12c7c887f8aaab779aef1dfa9b60673530a5adb88ecc55dd6df8f56e0b70f199";
const HELPER_SIZE: u64 = 55_329_336;

fn hidden(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

fn verified(path: &Path) -> Result<()> {
    ensure!(
        fs::metadata(path)?.len() == HELPER_SIZE,
        "The internet helper has an unexpected size."
    );
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    ensure!(
        format!("{:x}", hash.finalize()) == HELPER_HASH,
        "Internet helper verification failed. Do not run this file."
    );
    Ok(())
}

/// Run on a background setup thread. All script text and the download URL are
/// constants; paths are passed via environment values, never interpolated code.
pub fn ensure_helper(directory: &Path, cancel: &AtomicBool) -> Result<PathBuf> {
    ensure!(
        cfg!(windows),
        "Automatic internet hosting currently supports Windows."
    );
    fs::create_dir_all(directory)?;
    let target = directory.join(format!("cloudflared-{HELPER_VERSION}.exe"));
    if target.exists() {
        verified(&target)?;
        return Ok(target);
    }
    let partial = directory.join(format!(
        "download-{}-{}.partial",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let script = "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com/cloudflare/cloudflared/releases/download/2026.10.2/cloudflared-windows-amd64.exe' -OutFile $env:LG_COOP_HELPER_DOWNLOAD";
    let mut child = hidden(Command::new("powershell.exe").args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        script,
    ]))
    .env("LG_COOP_HELPER_DOWNLOAD", &partial)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .context("Could not start downloading the internet helper.")?;
    let started = Instant::now();
    let result = (|| {
        loop {
            ensure!(!cancel.load(Ordering::Relaxed), "Hosting cancelled.");
            ensure!(
                started.elapsed() < Duration::from_secs(120),
                "The internet helper download timed out. Try hosting again."
            );
            if let Some(status) = child.try_wait()? {
                ensure!(status.success(), "The internet helper could not be downloaded. Check your connection and try again.");
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        verified(&partial)?;
        if target.exists() {
            verified(&target)?;
        } else {
            fs::rename(&partial, &target)?;
        }
        Ok(target)
    })();
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&partial);
    result
}

pub struct Tunnel {
    child: Child,
    log: PathBuf,
    origin: Option<String>,
    started: Instant,
    registered_at: Option<Instant>,
}
impl Tunnel {
    pub fn start(helper: &Path, local_origin: &str, directory: &Path) -> Result<Self> {
        verified(helper)?;
        // Never let this helper expose an arbitrary URL supplied by another player.
        let port = local_origin
            .strip_prefix("http://127.0.0.1:")
            .context("The host listener must be local.")?
            .parse::<u16>()?;
        ensure!(port != 0, "The host port is unavailable.");
        fs::create_dir_all(directory)?;
        let session = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        let config = directory.join(format!("tunnel-{session}.yml"));
        fs::write(&config, b"{}\n")?;
        let log = directory.join(format!("tunnel-{session}.log"));
        let log_file = File::create(&log)?;
        let child = hidden(Command::new(helper).arg("tunnel"))
            .arg("--config")
            .arg(config)
            .args([
                "--no-autoupdate",
                "--url",
                local_origin,
                "--metrics",
                "127.0.0.1:0",
                "--loglevel",
                "info",
            ])
            .env_remove("TUNNEL_TOKEN")
            .env_remove("TUNNEL_TOKEN_FILE")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(log_file))
            .spawn()
            .context("Could not start internet hosting.")?;
        Ok(Self {
            child,
            log,
            origin: None,
            started: Instant::now(),
            registered_at: None,
        })
    }

    pub fn poll(&mut self) -> Result<Option<&str>> {
        if self.child.try_wait()?.is_some() {
            anyhow::bail!("{}", stopped_message(&read_log(&self.log).unwrap_or_default()));
        }
        if self.origin.is_none() || self.registered_at.is_none() {
            ensure!(
                self.started.elapsed() < Duration::from_secs(45),
                "Internet hosting timed out. Check your connection and try again."
            );
            let text = read_log(&self.log)?;
            self.origin = parse_origin(&text);
            if text.contains("Registered tunnel connection") {
                self.registered_at.get_or_insert_with(Instant::now);
            }
        }
        // Publishing the URL before the new DNS record propagates can poison
        // a guest resolver's negative cache for minutes. Keep setup in progress
        // until the route is registered and its DNS has had time to propagate.
        let ready = self
            .registered_at
            .is_some_and(|at| at.elapsed() >= Duration::from_secs(12));
        Ok(if ready { self.origin.as_deref() } else { None })
    }
}

fn read_log(path: &Path) -> Result<String> {
    let mut log = File::open(path)?;
    let len = log.metadata()?.len();
    log.seek(SeekFrom::Start(len.saturating_sub(128 * 1024)))?;
    let mut bytes = Vec::new();
    log.take(128 * 1024).read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn stopped_message(log: &str) -> &'static str {
    if log.lines().any(|line| line.trim() == "quick tunnel provisioning failed with status 429") {
        "The invite service is limiting new sessions. Please try hosting again later."
    } else {
        "Internet hosting stopped. Create a new invite and try again."
    }
}
impl Drop for Tunnel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parse_origin(log: &str) -> Option<String> {
    log.split_whitespace().find_map(|word| {
        let uri: tungstenite::http::Uri = word.parse().ok()?;
        let host = uri.host()?;
        (uri.scheme_str() == Some("https")
            && host.ends_with(".trycloudflare.com")
            && uri.authority().is_some_and(|a| !a.as_str().contains('@'))
            && uri.port().is_none()
            && uri.query().is_none()
            && (uri.path().is_empty() || uri.path() == "/"))
            .then(|| format!("https://{host}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_limited_provisioning_reports_retry_later_without_exposing_helper_logs() {
        let rate_limited = stopped_message("setup\nquick tunnel provisioning failed with status 429\n");
        assert!(rate_limited.contains("try hosting again later"));
        for log in ["", "quick tunnel provisioning failed with status 500", "port 429 registered"] {
            assert_ne!(stopped_message(log), rate_limited);
        }
    }
    #[test]
    fn helper_log_only_accepts_a_complete_expected_https_hostname() {
        assert_eq!(
            parse_origin("INF | https://blue-red-alice.trycloudflare.com |"),
            Some("https://blue-red-alice.trycloudflare.com".into())
        );
        for text in [
            "https://trycloudflare.com.evil.test",
            "https://trycloudflare.com@evil.test",
            "http://blue.trycloudflare.com",
            "https://blue.trycloudflare.com/path",
            "https://blue.trycloudflare.com/?x=y",
        ] {
            assert!(parse_origin(text).is_none());
        }
    }
}
