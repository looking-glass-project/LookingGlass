//! Cancellable Host setup. Keep this guard alongside the host session for the
//! lifetime of its invite; dropping it stops only its own internet helper.
use super::{
    invite::Invite,
    transport::{self, Conditions, Peer},
    tunnel::{self, Tunnel},
};
use anyhow::{ensure, Result};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    PreparingHelper,
    OpeningTunnel,
    CheckingConnection,
    Ready,
    Failed(String),
    Stopped,
}
struct Shared {
    cancel: AtomicBool,
    status: Mutex<Status>,
}
impl Shared {
    fn set(&self, status: Status) {
        *self.status.lock().unwrap() = status;
    }
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
}

pub struct Hosting {
    shared: Arc<Shared>,
    result: Receiver<(Peer, Invite)>,
}
impl Hosting {
    pub fn start(
        profile: PathBuf,
        helper_cache: PathBuf,
        internet: bool,
        conditions: Conditions,
    ) -> Result<Self> {
        let shared = Arc::new(Shared {
            cancel: AtomicBool::new(false),
            status: Mutex::new(Status::OpeningTunnel),
        });
        let (sender, result) = mpsc::sync_channel(1);
        let worker = shared.clone();
        thread::Builder::new()
            .name("coop-invite".into())
            .spawn(move || {
                let result = (|| -> Result<()> {
                    ensure!(!worker.cancelled(), "Hosting cancelled.");
                    let (peer, local) = transport::host(conditions)?;
                    let mut tunnel = None;
                    let invite = if internet {
                        worker.set(Status::PreparingHelper);
                        let helper = tunnel::ensure_helper(&helper_cache, &worker.cancel)?;
                        ensure!(!worker.cancelled(), "Hosting cancelled.");
                        worker.set(Status::OpeningTunnel);
                        let mut opening =
                            Tunnel::start(&helper, local.origin(), &profile.join("tunnel"))?;
                        let origin = loop {
                            ensure!(!worker.cancelled(), "Hosting cancelled.");
                            if let Some(origin) = opening.poll()? {
                                break origin.to_owned();
                            }
                            thread::sleep(Duration::from_millis(50));
                        };
                        let public = local.with_origin(&origin)?;
                        worker.set(Status::CheckingConnection);
                        check_public_route(&worker, &public, Duration::from_secs(60), || {
                            opening.poll().map(|_| ())
                        })?;
                        tunnel = Some(opening);
                        public
                    } else {
                        local
                    };
                    ensure!(!worker.cancelled(), "Hosting cancelled.");
                    sender
                        .send((peer, invite))
                        .map_err(|_| anyhow::anyhow!("Hosting cancelled."))?;
                    worker.set(Status::Ready);
                    while !worker.cancelled() {
                        if let Some(tunnel) = &mut tunnel {
                            tunnel.poll()?;
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                    Ok(()) // owned Tunnel drops here, including all error exits
                })();
                match result {
                    Err(error) if !worker.cancelled() => {
                        worker.set(Status::Failed(error.to_string()))
                    }
                    _ => worker.set(Status::Stopped),
                }
            })?;
        Ok(Self { shared, result })
    }
    pub fn status(&self) -> Status {
        self.shared.status.lock().unwrap().clone()
    }
    /// The caller moves the ready listener into Host and retains this guard.
    pub fn take_ready(&self) -> Result<Option<(Peer, Invite)>> {
        ensure!(!self.shared.cancelled(), "Hosting cancelled.");
        match self.result.try_recv() {
            Ok(value) => Ok(Some(value)),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => match self.status() {
                Status::Failed(reason) => anyhow::bail!("{reason}"),
                Status::Stopped => anyhow::bail!("Hosting stopped."),
                _ => Ok(None),
            },
        }
    }
    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
    }
}

fn check_public_route(
    shared: &Shared,
    invite: &Invite,
    budget: Duration,
    mut check_tunnel: impl FnMut() -> Result<()>,
) -> Result<()> {
    let started = Instant::now();
    let mut probe = None;
    let mut retry_at = started;
    let mut reason = "The connection check timed out.".to_owned();
    loop {
        ensure!(!shared.cancelled(), "Hosting cancelled.");
        ensure!(
            started.elapsed() < budget,
            "Your invite is not reachable yet. {reason} Try hosting again later."
        );
        check_tunnel()?;
        if probe.is_none() && Instant::now() >= retry_at {
            probe = Some(transport::Readiness::start(invite.clone())?);
        }
        if let Some(check) = &probe {
            match check.state() {
                transport::State::Connected => return Ok(()),
                transport::State::Disconnected(error) => {
                    reason = error;
                    probe = None;
                    retry_at = Instant::now() + Duration::from_secs(1);
                }
                transport::State::Stopped => anyhow::bail!("Invite connection check stopped."),
                _ => (),
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}
impl Drop for Hosting {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};
    use std::time::Instant;

    fn checking() -> Arc<Shared> {
        Arc::new(Shared {
            cancel: AtomicBool::new(false),
            status: Mutex::new(Status::CheckingConnection),
        })
    }

    fn endpoint() -> (TcpListener, Invite) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let invite = Invite::new(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
        (listener, invite)
    }

    // A bounded real endpoint lets these tests hold setup between its request
    // and response, without external DNS or a tunnel provider in unit tests.
    fn request(listener: &TcpListener) -> (TcpStream, String) {
        let started = Instant::now();
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(started.elapsed() < Duration::from_secs(4));
                    thread::sleep(Duration::from_millis(2));
                }
                Err(e) => panic!("{e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            socket.read_exact(&mut byte).unwrap();
            header.push(byte[0]);
            assert!(header.len() < transport::MAX_INPUT);
        }
        let header = String::from_utf8(header).unwrap();
        let challenge = header.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case("x-looking-glass-probe").then(|| value.trim().to_owned())
        }).unwrap();
        (socket, challenge)
    }

    #[test]
    fn public_route_retries_a_transient_failure_and_waits_for_the_actual_reply() {
        let (listener, invite) = endpoint();
        let shared = checking();
        let (finished, result) = mpsc::channel();
        let worker = thread::spawn(move || {
            finished.send(check_public_route(&shared, &invite, Duration::from_secs(5), || Ok(()))).unwrap();
        });
        let (mut first, _) = request(&listener);
        let failed_at = Instant::now();
        first.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(first);
        let (mut second, challenge) = request(&listener);
        assert!(failed_at.elapsed() >= Duration::from_secs(1));
        assert!(matches!(result.try_recv(), Err(TryRecvError::Empty)), "Setup finished before the endpoint answered");
        second.write_all(format!("HTTP/1.1 409 Conflict\r\nx-looking-glass-ready: {challenge}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes()).unwrap();
        result.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
        worker.join().unwrap();
        assert_eq!(second.read(&mut [0; 1]).unwrap(), 0, "Readiness retained its completed socket");
    }

    #[test]
    fn public_route_cancel_deadline_and_helper_failure_close_an_unanswered_probe() {
        for mode in ["cancel", "deadline", "helper"] {
            let (listener, invite) = endpoint();
            let shared = checking();
            let worker_shared = shared.clone();
            let helper_failed = Arc::new(AtomicBool::new(false));
            let worker_failed = helper_failed.clone();
            let budget = if mode == "deadline" { Duration::from_millis(600) } else { Duration::from_secs(5) };
            let (finished, result) = mpsc::channel();
            let worker = thread::spawn(move || {
                finished.send(check_public_route(&worker_shared, &invite, budget, || {
                    ensure!(!worker_failed.load(Ordering::Acquire), "Owned helper exited");
                    Ok(())
                })).unwrap();
            });
            let (mut socket, _) = request(&listener);
            assert!(matches!(result.try_recv(), Err(TryRecvError::Empty)));
            let stopped_at = Instant::now();
            match mode {
                "cancel" => shared.cancel.store(true, Ordering::Release),
                "helper" => helper_failed.store(true, Ordering::Release),
                _ => (),
            }
            let reason = result.recv_timeout(Duration::from_secs(2)).unwrap().unwrap_err().to_string();
            match mode {
                "cancel" => assert_eq!(reason, "Hosting cancelled."),
                "helper" => assert_eq!(reason, "Owned helper exited"),
                _ => assert!(reason.starts_with("Your invite is not reachable yet.")),
            }
            if mode != "deadline" { assert!(stopped_at.elapsed() < Duration::from_millis(750)); }
            socket.set_read_timeout(Some(Duration::from_millis(750))).unwrap();
            let closed = socket.read(&mut [0; 1]);
            assert!(matches!(&closed, Ok(0)) || matches!(&closed, Err(e)
                if matches!(e.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted)),
                "Unfinished readiness socket was not closed: {mode}: {closed:?}");
            worker.join().unwrap();
        }
    }
    #[test]
    fn dropping_or_cancelling_setup_releases_its_listener() {
        let hosting =
            Hosting::start(PathBuf::new(), PathBuf::new(), false, Conditions::default()).unwrap();
        let started = Instant::now();
        let (peer, invite) = loop {
            if let Some(ready) = hosting.take_ready().unwrap() {
                break ready;
            }
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        };
        let address = invite.origin().strip_prefix("http://").unwrap().to_owned();
        drop(peer);
        hosting.cancel();
        assert!(hosting.take_ready().is_err());
        while hosting.status() != Status::Stopped {
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
        // Listener worker may take a scheduling quantum to notice Peer::drop.
        loop {
            if let Ok(listener) = std::net::TcpListener::bind(&address) {
                drop(listener);
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
    }
}
