//! Real two-process transport diagnostics. No original game data is required.
//! This verifies connectivity/isolation, not shared gameplay or campaign support.
use super::{
    hosting::{Hosting, Status as HostingStatus},
    invite::Invite,
    transport::{self, Conditions, Peer, State},
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const ROUNDS: u32 = 16;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    sequence: u32,
    guest_pid: u32,
    host_pid: Option<u32>,
}

struct Options {
    host: bool,
    internet: bool,
    profile: PathBuf,
    invite: PathBuf,
    seconds: u64,
    conditions: Conditions,
}

pub fn cli(args: &[String]) -> Option<Result<()>> {
    match args.first().map(String::as_str) {
        Some("--coop-probe-host" | "--coop-probe-join") => Some(run(args)),
        _ => None,
    }
}

fn parse(args: &[String]) -> Result<Options> {
    let host = args[0] == "--coop-probe-host";
    let mut o = Options {
        host,
        internet: false,
        profile: PathBuf::new(),
        invite: PathBuf::new(),
        seconds: 90,
        conditions: Conditions::default(),
    };
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--internet" => o.internet = true,
            "--profile" => o.profile = iter.next().context("--profile requires a folder")?.into(),
            "--invite-file" => {
                o.invite = iter.next().context("--invite-file requires a file")?.into()
            }
            "--seconds" => {
                o.seconds = iter
                    .next()
                    .context("--seconds requires a duration")?
                    .parse()?
            }
            "--delay-ms" => {
                o.conditions.delay_ms = iter
                    .next()
                    .context("--delay-ms requires a number")?
                    .parse()?
            }
            "--jitter-ms" => {
                o.conditions.jitter_ms = iter
                    .next()
                    .context("--jitter-ms requires a number")?
                    .parse()?
            }
            _ => anyhow::bail!("Unknown co-op probe argument: {arg}"),
        }
    }
    ensure!(
        !o.profile.as_os_str().is_empty() && !o.invite.as_os_str().is_empty(),
        "The probe requires --profile and --invite-file."
    );
    ensure!(
        (5..=300).contains(&o.seconds),
        "Probe duration must be between 5 and 300 seconds."
    );
    ensure!(
        o.conditions.delay_ms <= 2000 && o.conditions.jitter_ms <= 2000,
        "Probe delay is limited to two seconds."
    );
    Ok(o)
}

fn run(args: &[String]) -> Result<()> {
    let o = parse(args)?;
    fs::create_dir_all(&o.profile)?;
    // Use the same preference resolver as the game, in separate OS processes.
    std::env::set_var("LOOKING_GLASS_SETTINGS_DIR", &o.profile);
    let identity = serde_json::json!({"role": if o.host { "host" } else { "guest" }, "pid": std::process::id()});
    fs::write(
        crate::preferences::path("coop-probe-settings.json"),
        serde_json::to_vec(&identity)?,
    )?;
    fs::create_dir_all(o.profile.join("saves"))?;
    fs::write(
        o.profile.join("saves/coop-probe-profile.json"),
        serde_json::to_vec(&identity)?,
    )?;
    if o.host {
        host(&o)
    } else {
        guest(&o)
    }
}

fn host(o: &Options) -> Result<()> {
    println!("Preparing authenticated hosting...");
    let hosting = Hosting::start(
        o.profile.clone(),
        PathBuf::from("private/coop/helpers"),
        o.internet,
        o.conditions,
    )?;
    let started = Instant::now();
    let (peer, invite) = loop {
        ensure!(
            started.elapsed() < Duration::from_secs(150),
            "Host setup timed out."
        );
        if let Some(ready) = hosting.take_ready()? {
            break ready;
        }
        thread::sleep(Duration::from_millis(50));
    };
    // A private file passes the secret between the two test processes. It is
    // deliberately never printed, embedded in a command line or sent to CI logs.
    if let Some(parent) = o.invite.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&o.invite, invite.link())?;
    println!("Invite ready; waiting for the independent guest process.");
    let started = Instant::now();
    let mut received = 0;
    let mut guest_pid = None;
    let mut pending = None;
    loop {
        ensure!(
            started.elapsed() < Duration::from_secs(o.seconds),
            "Host probe timed out after {received}/{ROUNDS} exchanges."
        );
        if let HostingStatus::Failed(reason) = hosting.status() {
            anyhow::bail!("{reason}");
        }
        if let Some(data) = peer.receive() {
            let mut probe: Probe = serde_json::from_slice(&data)?;
            ensure!(
                probe.host_pid.is_none() && probe.guest_pid != std::process::id(),
                "The guest must be a separate process."
            );
            ensure!(
                probe.sequence == received,
                "Probe messages arrived out of order."
            );
            ensure!(
                guest_pid.is_none() || guest_pid == Some(probe.guest_pid),
                "The guest identity changed during the probe."
            );
            guest_pid = Some(probe.guest_pid);
            received += 1;
            probe.host_pid = Some(std::process::id());
            ensure!(
                pending.is_none(),
                "Guest sent another request before its reply."
            );
            pending = Some(serde_json::to_vec(&probe)?);
        }
        if let Some(data) = &pending {
            if peer.send(data.clone())? {
                pending = None;
            }
        }
        if matches!(peer.state(), State::Disconnected(_)) && received == ROUNDS {
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    fs::write(
        o.profile.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "passed": true, "role": "host", "pid": std::process::id(), "peer_pid": guest_pid,
            "exchanges": received, "internet": o.internet,
        }))?,
    )?;
    println!(
        "PASS host: {ROUNDS} ordered exchanges with a separate process; guest disconnect observed."
    );
    hosting.cancel();
    let stopping = Instant::now();
    while hosting.status() != HostingStatus::Stopped {
        ensure!(
            stopping.elapsed() < Duration::from_secs(5),
            "Internet hosting did not stop cleanly."
        );
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn guest(o: &Options) -> Result<()> {
    let started = Instant::now();
    let invite =
        Invite::parse(&fs::read_to_string(&o.invite).context("Host invite is not ready")?)?;
    let mut last_error = "The host has not responded.".to_owned();
    let peer = loop {
        ensure!(
            started.elapsed() < Duration::from_secs(o.seconds),
            "Guest could not connect before the probe deadline: {last_error}"
        );
        let peer = transport::join(invite.clone(), o.conditions)?;
        while peer.state() == State::Connecting {
            thread::sleep(Duration::from_millis(10));
        }
        if peer.state() == State::Connected {
            break peer;
        }
        if let State::Disconnected(reason) = peer.state() {
            if last_error != reason {
                println!("Connection attempt: {reason}");
            }
            last_error = reason;
        }
        // A new public tunnel's route/DNS may need a moment after its URL appears.
        drop(peer);
        thread::sleep(Duration::from_millis(300));
    };
    println!("Guest connected; exchanging authenticated packets.");
    let mut host_pid = None;
    for sequence in 0..ROUNDS {
        let probe = Probe {
            sequence,
            guest_pid: std::process::id(),
            host_pid: None,
        };
        let data = serde_json::to_vec(&probe)?;
        loop {
            check_deadline(&peer, started, o.seconds)?;
            if peer.send(data.clone())? {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        loop {
            check_deadline(&peer, started, o.seconds)?;
            if let Some(data) = peer.receive() {
                let reply: Probe = serde_json::from_slice(&data)?;
                ensure!(
                    reply.sequence == sequence && reply.guest_pid == std::process::id(),
                    "The host returned the wrong probe reply."
                );
                let pid = reply.host_pid.context("The reply has no host identity")?;
                ensure!(
                    pid != std::process::id() && (host_pid.is_none() || host_pid == Some(pid)),
                    "The host is not a stable, separate process."
                );
                host_pid = Some(pid);
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
    }
    // Stop and wait for the worker to deliver a WebSocket close before process
    // exit. This also proves shutdown is bounded and releases the host's slot.
    peer.stop();
    let stopping = Instant::now();
    while peer.state() != State::Stopped {
        ensure!(
            stopping.elapsed() < Duration::from_secs(3),
            "Guest transport did not stop cleanly."
        );
        thread::sleep(Duration::from_millis(5));
    }
    fs::write(
        o.profile.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "passed": true, "role": "guest", "pid": std::process::id(), "peer_pid": host_pid,
            "exchanges": ROUNDS, "internet": invite.origin().starts_with("https://"),
        }))?,
    )?;
    println!("PASS guest: {ROUNDS} ordered exchanges and clean shutdown.");
    Ok(())
}

fn check_deadline(peer: &Peer, start: Instant, seconds: u64) -> Result<()> {
    ensure!(
        start.elapsed() < Duration::from_secs(seconds),
        "Guest probe timed out."
    );
    if let State::Disconnected(message) = peer.state() {
        anyhow::bail!("{message}");
    }
    Ok(())
}
