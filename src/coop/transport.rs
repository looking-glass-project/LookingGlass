//! Bounded, authenticated WebSocket transport. No network work runs on the UI thread.
use super::invite::{authorized, Invite, PROTOCOL};
use anyhow::{bail, Context, Result};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tungstenite::{
    client::IntoClientRequest,
    handshake::server::{ErrorResponse, Request, Response},
    http::{header, StatusCode},
    protocol::WebSocketConfig,
    stream::MaybeTlsStream,
    Error, Message, WebSocket,
};

pub const MAX_INPUT: usize = 8 * 1024;
pub const MAX_STATE: usize = 8 * 1024 * 1024;
const QUEUE: usize = 4;
const IDLE: Duration = Duration::from_secs(15);
/// Session-level ceiling for connection setup: DNS may retry for 20 seconds,
/// followed by up to four TCP attempts and the TLS/WebSocket handshake. Keep
/// this separate from the shorter identity-check timeout after connection.
pub(super) const CONNECT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Waiting,
    Connecting,
    Connected,
    Disconnected(String),
    Stopped,
}

/// Reproducible application-level delay; TCP itself preserves message order.
#[derive(Clone, Copy, Default)]
pub struct Conditions {
    pub delay_ms: u16,
    pub jitter_ms: u16,
}

struct Shared {
    stop: AtomicBool,
    state: Mutex<State>,
    generation: AtomicU64,
    disconnect_generation: AtomicU64,
    incoming: Mutex<Receiver<(u64, Vec<u8>)>>,
}
impl Shared {
    fn set(&self, state: State) {
        *self.state.lock().unwrap() = state;
    }
    fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
}

pub struct Peer {
    outgoing: SyncSender<(u64, Vec<u8>)>,
    shared: Arc<Shared>,
    outgoing_limit: usize,
}
impl Peer {
    pub fn state(&self) -> State {
        self.shared.state.lock().unwrap().clone()
    }
    pub fn receive(&self) -> Option<Vec<u8>> {
        let incoming = self.shared.incoming.lock().unwrap();
        while let Ok((generation, data)) = incoming.try_recv() {
            if generation == self.generation() {
                return Some(data);
            }
        }
        None
    }
    pub fn generation(&self) -> u64 {
        self.shared.generation.load(Ordering::Acquire)
    }

    /// False means backpressure: keep/retry ordered input edges and replace a
    /// pending state snapshot with a newer one. Do not grow an unbounded queue.
    pub fn send(&self, data: Vec<u8>) -> Result<bool> {
        anyhow::ensure!(
            !data.is_empty() && data.len() <= self.outgoing_limit,
            "Co-op packet exceeds its allowed size"
        );
        if self.state() != State::Connected {
            return Ok(false);
        }
        match self.outgoing.try_send((self.generation(), data)) {
            Ok(()) => Ok(true),
            Err(TrySendError::Full(_)) => Ok(false),
            Err(TrySendError::Disconnected(_)) => bail!("Co-op connection has closed"),
        }
    }

    pub fn stop(&self) {
        self.shared.stop.store(true, Ordering::Relaxed);
    }

    /// Reject only the current guest, keeping a host's invite/listener usable.
    /// A generation tag prevents a late rejection from closing a new guest.
    pub fn disconnect(&self) {
        self.shared
            .disconnect_generation
            .store(self.generation(), Ordering::Release);
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn channels(
    state: State,
    outgoing_limit: usize,
) -> (
    Peer,
    Receiver<(u64, Vec<u8>)>,
    SyncSender<(u64, Vec<u8>)>,
    Arc<Shared>,
) {
    let (outgoing, outgoing_rx) = mpsc::sync_channel(QUEUE);
    let (incoming_tx, incoming) = mpsc::sync_channel(QUEUE);
    let shared = Arc::new(Shared {
        stop: AtomicBool::new(false),
        state: Mutex::new(state),
        generation: AtomicU64::new(0),
        disconnect_generation: AtomicU64::new(0),
        incoming: Mutex::new(incoming),
    });
    (
        Peer {
            outgoing,
            shared: shared.clone(),
            outgoing_limit,
        },
        outgoing_rx,
        incoming_tx,
        shared,
    )
}

fn config(limit: usize) -> WebSocketConfig {
    WebSocketConfig::default()
        .read_buffer_size(8192)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_STATE * 2)
        .max_message_size(Some(limit))
        .max_frame_size(Some(limit))
}

fn reject(status: StatusCode, message: &str) -> ErrorResponse {
    tungstenite::http::Response::builder()
        .status(status)
        .body(Some(message.to_owned()))
        .unwrap()
}

/// Stop an unauthenticated peer from keeping the handshake alive indefinitely
/// by sending a byte before each socket timeout, or by sending enormous headers.
struct HandshakeStream {
    tcp: TcpStream,
    deadline: Option<Instant>,
    bytes: usize,
}
impl Read for HandshakeStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Some(deadline) = self.deadline {
            if Instant::now() >= deadline || self.bytes >= 16384 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Handshake limit reached",
                ));
            }
            let limit = buffer.len().min(16384 - self.bytes);
            let read = self.tcp.read(&mut buffer[..limit])?;
            self.bytes += read;
            Ok(read)
        } else {
            self.tcp.read(buffer)
        }
    }
}
impl Write for HandshakeStream {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.tcp.write(data)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.tcp.flush()
    }
}

fn handshake(stream: TcpStream, invite: &Invite) -> Result<WebSocket<HandshakeStream>> {
    // Windows accepts inherit the listener's nonblocking mode. The bounded
    // upgrade below is synchronous: otherwise a perfectly valid guest whose
    // first HTTP bytes arrive later is mistaken for a failed handshake.
    stream.set_nonblocking(false)?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let expected = invite.authorization();
    let callback = |request: &Request, mut response: Response| {
        if request.uri().path() != "/coop" || request.uri().query().is_some() {
            return Err(reject(
                StatusCode::NOT_FOUND,
                "In Looking Glass, choose Co-op > Join and paste the complete invite link.",
            ));
        }
        if !request
            .headers()
            .get(header::AUTHORIZATION)
            .is_some_and(|h| authorized(&expected, h.as_bytes()))
        {
            return Err(reject(
                StatusCode::UNAUTHORIZED,
                "This invite is incorrect or expired.",
            ));
        }
        if request
            .headers()
            .get(header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|h| h.to_str().ok())
            != Some(PROTOCOL)
        {
            return Err(reject(
                StatusCode::BAD_REQUEST,
                "The players need compatible co-op builds.",
            ));
        }
        response
            .headers_mut()
            .insert(header::SEC_WEBSOCKET_PROTOCOL, PROTOCOL.parse().unwrap());
        Ok(response)
    };
    let stream = HandshakeStream {
        tcp: stream,
        deadline: Some(Instant::now() + Duration::from_secs(2)),
        bytes: 0,
    };
    let mut socket = tungstenite::accept_hdr_with_config(stream, callback, Some(config(MAX_INPUT)))
        .map_err(|_| anyhow::anyhow!("Invite authentication or handshake failed"))?;
    socket.get_mut().deadline = None;
    socket.get_ref().tcp.set_nonblocking(true)?;
    Ok(socket)
}

/// Bind only loopback. A separately owned tunnel exposes exactly this listener,
/// never another local application, a directory listing or a remote shell.
pub fn host(conditions: Conditions) -> Result<(Peer, Invite)> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(true)?;
    let invite = Invite::new(&format!("http://{}", listener.local_addr()?))?;
    let worker_invite = invite.clone();
    let (peer, outgoing, incoming, shared) = channels(State::Waiting, MAX_STATE);
    thread::Builder::new()
        .name("coop-host".into())
        .spawn(move || {
            while !shared.stopped() {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if let Ok(mut socket) = handshake(stream, &worker_invite) {
                            if shared.stopped() {
                                break;
                            }
                            // Reconnecting guests get a fresh state from the simulation.
                            while outgoing.try_recv().is_ok() {}
                            while shared.incoming.lock().unwrap().try_recv().is_ok() {}
                            shared.generation.fetch_add(1, Ordering::AcqRel);
                            shared.set(State::Connected);
                            let result = exchange(
                                &mut socket,
                                &outgoing,
                                &incoming,
                                &shared,
                                conditions,
                                Some(&listener),
                            );
                            if !shared.stopped() {
                                shared.set(State::Disconnected(result.err().map_or_else(
                                    || "Player left the session.".into(),
                                    |e| e.to_string(),
                                )));
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => {
                        shared.set(State::Disconnected(
                            "Unable to accept a player connection.".into(),
                        ));
                        break;
                    }
                }
            }
            if shared.stopped() {
                shared.set(State::Stopped);
            }
        })?;
    Ok((peer, invite))
}

// A newly registered relay can briefly fail resolution on a guest's resolver.
// Keep this on the connection worker, retry within a finite budget, and honour
// cancellation between lookups and while waiting. An OS lookup already in
// progress may still take its system timeout; never connect after cancellation.
fn resolve_with_retry(
    shared: &Shared,
    budget: Duration,
    interval: Duration,
    mut resolve: impl FnMut() -> std::io::Result<Vec<SocketAddr>>,
) -> Result<Vec<SocketAddr>> {
    let deadline = Instant::now() + budget;
    loop {
        anyhow::ensure!(!shared.stopped(), "Connection cancelled");
        let result = resolve();
        anyhow::ensure!(!shared.stopped(), "Connection cancelled");
        if let Ok(addresses) = result {
            if !addresses.is_empty() { return Ok(addresses); }
        }
        let retry_at = (Instant::now() + interval).min(deadline);
        loop {
            anyhow::ensure!(!shared.stopped(), "Connection cancelled");
            let now = Instant::now();
            anyhow::ensure!(now < deadline,
                "Cannot resolve the invite's internet address. Check your connection and try joining again.");
            if now >= retry_at { break; }
            thread::sleep((retry_at - now).min(Duration::from_millis(50)));
        }
    }
}

fn connect(invite: &Invite, shared: &Shared) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let url = invite.websocket_url();
    let uri: tungstenite::http::Uri = url.parse()?;
    let host = uri
        .host()
        .context("Invite host is missing")?
        .trim_matches(['[', ']']);
    let port = uri
        .port_u16()
        .unwrap_or(if uri.scheme_str() == Some("wss") {
            443
        } else {
            80
        });
    let mut stream = None;
    let addresses = resolve_with_retry(shared, Duration::from_secs(20), Duration::from_secs(1),
        || (host, port).to_socket_addrs().map(|addresses| addresses.take(4).collect()))?;
    for address in addresses {
        if shared.stopped() {
            bail!("Connection cancelled");
        }
        if let Ok(tcp) = TcpStream::connect_timeout(&address, Duration::from_secs(4)) {
            stream = Some(tcp);
            break;
        }
    }
    let stream = stream.context("Cannot reach the host. Check that the host is still open.")?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(8)))?;
    stream.set_write_timeout(Some(Duration::from_secs(8)))?;
    let mut request = url.into_client_request()?;
    request
        .headers_mut()
        .insert(header::AUTHORIZATION, invite.authorization().parse()?);
    request
        .headers_mut()
        .insert(header::SEC_WEBSOCKET_PROTOCOL, PROTOCOL.parse()?);
    let (socket, response) =
        tungstenite::client_tls_with_config(request, stream, Some(config(MAX_STATE)), None)
            .map_err(|error| {
                // Never include request headers or response bodies in player logs.
                let reason = match error {
                    tungstenite::HandshakeError::Failure(Error::Http(response)) => {
                        format!("The host returned HTTP {}.", response.status().as_u16())
                    }
                    tungstenite::HandshakeError::Failure(Error::Tls(_)) => {
                        "The secure connection could not be verified.".to_owned()
                    }
                    tungstenite::HandshakeError::Failure(Error::Io(e)) => {
                        format!("The connection failed ({:?}).", e.kind())
                    }
                    _ => "The host did not complete a compatible co-op handshake.".to_owned(),
                };
                anyhow::anyhow!("{reason} Check the invite and that the host is still open.")
            })?;
    anyhow::ensure!(
        response
            .headers()
            .get(header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|h| h.to_str().ok())
            == Some(PROTOCOL),
        "The invite does not point to a compatible Looking Glass host."
    );
    match socket.get_ref() {
        MaybeTlsStream::Plain(tcp) => tcp.set_nonblocking(true)?,
        MaybeTlsStream::NativeTls(tls) => tls.get_ref().set_nonblocking(true)?,
        _ => bail!("Unsupported TLS connection"),
    }
    Ok(socket)
}

pub fn join(invite: Invite, conditions: Conditions) -> Result<Peer> {
    let (peer, outgoing, incoming, shared) = channels(State::Connecting, MAX_INPUT);
    thread::Builder::new()
        .name("coop-guest".into())
        .spawn(move || {
            let result = (|| {
                let mut socket = connect(&invite, &shared)?;
                if shared.stopped() {
                    return Ok(());
                }
                shared.generation.store(1, Ordering::Release);
                shared.set(State::Connected);
                exchange(&mut socket, &outgoing, &incoming, &shared, conditions, None)
            })();
            if shared.stopped() {
                shared.set(State::Stopped);
            } else {
                shared.set(State::Disconnected(result.err().map_or_else(
                    || "The host ended the session.".into(),
                    |e: anyhow::Error| e.to_string(),
                )));
            }
        })?;
    Ok(peer)
}

fn would_block(error: &Error) -> bool {
    matches!(error, Error::Io(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut))
}

fn exchange<S: Read + Write>(
    socket: &mut WebSocket<S>,
    outgoing: &Receiver<(u64, Vec<u8>)>,
    incoming: &SyncSender<(u64, Vec<u8>)>,
    shared: &Shared,
    conditions: Conditions,
    listener: Option<&TcpListener>,
) -> Result<()> {
    let mut received_at = Instant::now();
    let mut ping_at = Instant::now();
    let mut pending = VecDeque::<(Instant, Vec<u8>)>::new();
    let mut random = 0x41872_u32;
    let generation = shared.generation.load(Ordering::Acquire);
    let mut waiting_input = None;
    let mut blocked_since = None;
    while !shared.stopped() {
        if shared.disconnect_generation.load(Ordering::Acquire) == generation {
            break;
        }
        // A third connection gets an immediate failure instead of waiting forever.
        if let Some(listener) = listener {
            if let Ok((mut extra, _)) = listener.accept() {
                let _ = extra.set_write_timeout(Some(Duration::from_millis(100)));
                let _ = extra.write_all(
                    b"HTTP/1.1 409 Conflict\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
            }
        }
        if pending.len() < QUEUE {
            match outgoing.try_recv() {
                Ok((packet_generation, data)) => {
                    if packet_generation != generation {
                        continue;
                    }
                    random = random.wrapping_mul(1664525).wrapping_add(1013904223);
                    let jitter = random % (u32::from(conditions.jitter_ms) + 1);
                    let due = Instant::now()
                        + Duration::from_millis(u64::from(conditions.delay_ms) + u64::from(jitter));
                    let due = pending.back().map_or(due, |(last, _)| due.max(*last));
                    pending.push_back((due, data));
                }
                Err(TryRecvError::Disconnected) => return Ok(()),
                Err(TryRecvError::Empty) => (),
            }
        }
        if pending
            .front()
            .is_some_and(|(due, _)| *due <= Instant::now())
        {
            let (_, data) = pending.pop_front().unwrap();
            if let Err(e) = socket.write(Message::Binary(data.into())) {
                if !would_block(&e) {
                    bail!("The connection cannot keep up with game updates.");
                }
            }
        }
        if ping_at.elapsed() >= Duration::from_secs(3) {
            if let Err(e) = socket.write(Message::Ping(Vec::new().into())) {
                if !would_block(&e) {
                    bail!("Connection lost while checking the other player.");
                }
            }
            ping_at = Instant::now();
        }
        if let Err(e) = socket.flush() {
            if !would_block(&e) {
                bail!("The other player disconnected.");
            }
        }
        // TCP may deliver a normal internet burst between render frames. Keep
        // at most one additional message and stop reading until the consumer
        // catches up. Checkpoint writes can briefly block the render thread;
        // use the same bounded grace as a quiet connection instead of dropping
        // a healthy session after two seconds. Queue capacity is unchanged.
        if let Some(data) = waiting_input.take() {
            match incoming.try_send((generation, data)) {
                Ok(()) => blocked_since = None,
                Err(TrySendError::Full((_, data))) => waiting_input = Some(data),
                Err(TrySendError::Disconnected(_)) => return Ok(()),
            }
        }
        anyhow::ensure!(
            blocked_since.is_none_or(|at: Instant| at.elapsed() < IDLE),
            "Game updates could not be consumed fast enough. Rejoin the host."
        );
        // Bound processing as well as memory so flooding cannot starve outgoing traffic.
        for _ in 0..16 {
            if waiting_input.is_some() {
                break;
            }
            match socket.read() {
                Ok(message) => {
                    received_at = Instant::now();
                    match message {
                        Message::Binary(bytes) => {
                            anyhow::ensure!(!bytes.is_empty(), "Received an empty game packet.");
                            match incoming.try_send((generation, bytes.to_vec())) {
                                Ok(()) => (),
                                Err(TrySendError::Full((_, data))) => {
                                    waiting_input = Some(data);
                                    blocked_since.get_or_insert_with(Instant::now);
                                    break;
                                }
                                Err(TrySendError::Disconnected(_)) => return Ok(()),
                            }
                        }
                        Message::Ping(_) | Message::Pong(_) => (),
                        Message::Close(_) => return Ok(()),
                        _ => bail!("The other player sent an unsupported packet."),
                    }
                }
                Err(e) if would_block(&e) => break,
                Err(_) => bail!("The other player disconnected or sent an invalid packet."),
            }
        }
        anyhow::ensure!(
            received_at.elapsed() < IDLE,
            "The other player stopped responding."
        );
        thread::sleep(Duration::from_millis(2));
    }
    let _ = socket.close(None);
    let _ = socket.flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guest_waits_for_connection_setup_but_keeps_handshake_and_cancel_bounds() {
        use crate::coop::{protocol::Identity,session::{Guest,Phase}};
        for connects in [false,true] {
            let (peer,_outgoing,_incoming,shared)=channels(State::Connecting,MAX_INPUT);
            let start=Instant::now();
            let mut guest=Guest::new(peer,Identity::current("b".repeat(64)),start).unwrap();
            // A DNS retry can legitimately consume more than the old 15-second
            // identity-check allowance before the socket is even connected.
            let resolved=start+Duration::from_secs(21);
            guest.poll(resolved).unwrap();
            assert_eq!(guest.phase(),Phase::Connecting);assert!(!shared.stopped());
            let expired=if connects {
                shared.set(State::Connected);
                guest.poll(resolved).unwrap();
                assert_eq!(guest.phase(),Phase::Checking);
                guest.poll(resolved+Duration::from_secs(14)).unwrap();
                resolved+Duration::from_secs(15)
            } else {
                guest.poll(start+CONNECT_TIMEOUT-Duration::from_millis(1)).unwrap();
                start+CONNECT_TIMEOUT
            };
            assert!(guest.poll(expired).is_err());
            assert_eq!(guest.phase(),Phase::Ended);assert!(shared.stopped());
        }
        let (peer,_outgoing,_incoming,shared)=channels(State::Connecting,MAX_INPUT);
        let guest=Guest::new(peer,Identity::current("b".repeat(64)),Instant::now()).unwrap();
        drop(guest);assert!(shared.stopped(),"Cancelling setup left the connection worker live");
    }
    #[test]
    fn transient_and_empty_dns_results_recover_to_a_reachable_listener() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let expected = listener.local_addr().unwrap();
        let (_peer, _, _, shared) = channels(State::Connecting, MAX_INPUT);
        let mut attempts = 0;
        let addresses = resolve_with_retry(&shared, Duration::from_secs(2), Duration::from_millis(1), || {
            attempts += 1;
            match attempts {
                1 => Err(std::io::ErrorKind::NotFound.into()),
                2 => Ok(Vec::new()),
                _ => ("localhost", expected.port()).to_socket_addrs()
                    .map(|all| all.filter(|a| a.is_ipv4()).collect()),
            }
        }).unwrap();
        assert_eq!(attempts, 3);
        let client = TcpStream::connect_timeout(&addresses[0], Duration::from_secs(1)).unwrap();
        let (server, _) = listener.accept().unwrap();
        assert_eq!(client.peer_addr().unwrap(), server.local_addr().unwrap());
    }
    #[test]
    fn persistent_dns_failure_exhausts_the_budget() {
        let (_peer, _, _, shared) = channels(State::Connecting, MAX_INPUT);
        let start = Instant::now();
        let result = resolve_with_retry(&shared, Duration::from_millis(20), Duration::from_millis(1),
            || Err(std::io::ErrorKind::NotFound.into()));
        assert!(result.unwrap_err().to_string().starts_with("Cannot resolve the invite's internet address."));
        assert!(start.elapsed() >= Duration::from_millis(20));
    }
    #[test]
    fn cancelling_dns_wait_or_inflight_lookup_never_returns_an_address() {
        let (peer, _, _, shared) = channels(State::Connecting, MAX_INPUT);
        let (attempted, ready) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut attempts = 0;
            let result = resolve_with_retry(&shared, Duration::from_secs(20), Duration::from_secs(10), || {
                attempts += 1;
                attempted.send(()).unwrap();
                Err(std::io::ErrorKind::NotFound.into())
            });
            (result, attempts)
        });
        ready.recv_timeout(Duration::from_secs(2)).unwrap();
        peer.stop();
        let (result, attempts) = worker.join().unwrap();
        assert_eq!(attempts, 1);
        assert_eq!(result.unwrap_err().to_string(), "Connection cancelled");

        let (peer, _, _, shared) = channels(State::Connecting, MAX_INPUT);
        let result = resolve_with_retry(&shared, Duration::from_secs(20), Duration::from_secs(1), || {
            peer.stop();
            Ok(vec!["127.0.0.1:1234".parse().unwrap()])
        });
        assert_eq!(result.unwrap_err().to_string(), "Connection cancelled");
    }
    #[test]
    fn accepted_nonblocking_socket_waits_for_delayed_upgrade_bytes() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let invite = Invite::new(&format!("http://{address}")).unwrap();
        let server_invite = invite.clone();
        let (accepted, ready) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            // Reproduce Windows inheritance on every test platform.
            stream.set_nonblocking(true).unwrap();
            accepted.send(()).unwrap();
            handshake(stream, &server_invite).map(|_| ())
        });
        let stream = TcpStream::connect(address).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        ready.recv_timeout(Duration::from_secs(3)).unwrap();
        thread::sleep(Duration::from_millis(100));
        let mut request = invite.websocket_url().into_client_request().unwrap();
        request.headers_mut().insert(header::AUTHORIZATION, invite.authorization().parse().unwrap());
        request.headers_mut().insert(header::SEC_WEBSOCKET_PROTOCOL, PROTOCOL.parse().unwrap());
        let result = tungstenite::client(request, stream);
        server.join().unwrap().unwrap();
        assert_eq!(result.unwrap().1.status(), StatusCode::SWITCHING_PROTOCOLS);
    }
    #[track_caller]
    fn wait_for(mut done: impl FnMut() -> bool) {
        let started = Instant::now();
        while !done() {
            assert!(started.elapsed() < Duration::from_secs(5));
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[track_caller]
    fn wait_connected(host: &Peer, guest: &Peer) {
        wait_for(|| {
            let h = host.state();
            let g = guest.state();
            assert!(!matches!(h, State::Disconnected(_) | State::Stopped), "Host: {h:?}, guest: {g:?}");
            assert!(!matches!(g, State::Disconnected(_) | State::Stopped), "Host: {h:?}, guest: {g:?}");
            h == State::Connected && g == State::Connected
        });
    }
    #[test]
    fn separate_endpoints_exchange_delayed_packets_and_reconnect_to_same_host() {
        let delay = Conditions {
            delay_ms: 40,
            jitter_ms: 30,
        };
        let (host, invite) = host(delay).unwrap();
        let guest = join(invite.clone(), delay).unwrap();
        wait_connected(&host, &guest);
        let started = Instant::now();
        assert!(guest.send(b"input".to_vec()).unwrap());
        wait_for(|| {
            host.receive()
                .map(|p| {
                    assert_eq!(p, b"input");
                    true
                })
                .unwrap_or(false)
        });
        assert!(started.elapsed() >= Duration::from_millis(40));
        assert!(host.send(b"state".to_vec()).unwrap());
        wait_for(|| {
            guest
                .receive()
                .map(|p| {
                    assert_eq!(p, b"state");
                    true
                })
                .unwrap_or(false)
        });
        drop(guest);
        wait_for(|| matches!(host.state(), State::Disconnected(_)));
        let guest2 = join(invite, Conditions::default()).unwrap();
        wait_for(|| guest2.state() == State::Connected && host.state() == State::Connected);
        assert!(host.send(b"fresh state".to_vec()).unwrap());
        wait_for(|| guest2.receive().is_some());
    }
    #[test]
    fn invalid_invite_cannot_take_the_slot_and_third_player_cannot_join() {
        let (host, invite) = host(Conditions::default()).unwrap();
        let wrong = join(Invite::new(invite.origin()).unwrap(), Conditions::default()).unwrap();
        wait_for(|| matches!(wrong.state(), State::Disconnected(_)));
        assert_eq!(host.state(), State::Waiting);
        let guest = join(invite.clone(), Conditions::default()).unwrap();
        wait_for(|| guest.state() == State::Connected);
        let third = join(invite, Conditions::default()).unwrap();
        wait_for(|| matches!(third.state(), State::Disconnected(_)));
        assert_eq!(guest.state(), State::Connected);
        assert!(!guest.send(vec![0; MAX_INPUT + 1]).is_ok());
    }
    #[test]
    fn ignored_updates_and_slow_readers_have_bounded_backpressure() {
        let (host, invite) = host(Conditions::default()).unwrap();
        let guest = join(invite, Conditions::default()).unwrap();
        wait_connected(&host, &guest);
        let started = Instant::now();
        loop {
            if matches!(guest.state(), State::Disconnected(_)) {
                break;
            }
            let _ = host.send(vec![1; 1024]);
            assert!(started.elapsed() < IDLE + Duration::from_secs(3));
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn a_checkpoint_length_stall_preserves_ordered_packets_and_the_connection() {
        let (host, invite) = host(Conditions::default()).unwrap();
        let guest = join(invite, Conditions::default()).unwrap();
        wait_connected(&host, &guest);
        for n in 0..8_u8 {
            wait_for(|| host.send(vec![n]).unwrap());
        }
        // Exceeds the former two-second queue deadline. A slow checkpoint
        // write must not discard the peer or any of its ordered packets.
        thread::sleep(Duration::from_secs(3));
        assert_eq!(guest.state(), State::Connected);
        for n in 0..8_u8 {
            wait_for(|| {
                guest
                    .receive()
                    .map(|packet| {
                        assert_eq!(packet, vec![n]);
                        true
                    })
                    .unwrap_or(false)
            });
        }
        assert!(host.send(b"after burst".to_vec()).unwrap());
        wait_for(|| {
            guest
                .receive()
                .map(|packet| {
                    assert_eq!(packet, b"after burst");
                    true
                })
                .unwrap_or(false)
        });
    }

    #[test]
    fn reconnect_discards_queued_input_from_the_previous_connection() {
        let (peer, _, incoming, shared) = channels(State::Connected, MAX_INPUT);
        shared.generation.store(1, Ordering::Release);
        incoming.send((1, b"old input".to_vec())).unwrap();
        shared.generation.store(2, Ordering::Release);
        incoming.send((2, b"new input".to_vec())).unwrap();
        assert_eq!(peer.receive(), Some(b"new input".to_vec()));
        assert!(peer.receive().is_none());
    }
}
