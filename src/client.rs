//! The client: a `sloptopop serve --stdio` child process and the JSON-RPC
//! session with it.

use std::fmt;
use std::io::{self, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};

use serde_json::Value;

use crate::error::Error;
use crate::handshake::{ServerInfo, initialize_params, server_info_from};
use crate::locate;
use crate::options::Options;
use crate::rpc::{call, notify};

/// A running `sloptopop serve --stdio` process with an initialized session.
///
/// Requests are synchronous: each call writes one request and blocks until
/// its reply arrives. The server's stderr is passed through to this
/// process's stderr.
///
/// End the session with [`Client::close`]. Dropping a client that was not
/// closed does the same on a best-effort basis, ignoring errors.
pub struct Client {
    child: Child,
    stdin: Box<dyn Write + Send>,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    info: ServerInfo,
    closed: bool,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("id", &self.child.id())
            .field("info", &self.info)
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Starts `sloptopop serve --stdio` and initializes the session.
    ///
    /// # Errors
    ///
    /// [`Error::BinaryNotFound`] or [`Error::Spawn`] if the binary can't be
    /// found or started, [`Error::Rpc`] if the server refuses `initialize`
    /// (e.g. [`code::UNSUPPORTED_PROTOCOL_VERSION`](crate::code::UNSUPPORTED_PROTOCOL_VERSION)),
    /// or another transport error. The process is killed on failure.
    pub fn start(options: Options) -> Result<Self, Error> {
        let path = locate::find(options.binary).ok_or(Error::BinaryNotFound)?;
        let mut child = Command::new(&path)
            .args(["serve", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| Error::Spawn { path, source })?;
        // Both are `Some`: they were requested as pipes just above.
        let pipes = child.stdin.take().zip(child.stdout.take());
        pipes.ok_or(Error::ProcessDied).and_then(|(stdin, stdout)| {
            let mut client = Self {
                child,
                stdin: Box::new(BufWriter::new(stdin)),
                stdout: BufReader::new(stdout),
                next_id: 1,
                info: ServerInfo::default(),
                closed: false,
            };
            match client.initialize() {
                Ok(info) => {
                    client.info = info;
                    Ok(client)
                }
                Err(e) => {
                    client.abort();
                    Err(e)
                }
            }
        })
    }

    /// What the server reported in reply to `initialize`.
    #[must_use]
    pub const fn server_info(&self) -> &ServerInfo {
        &self.info
    }

    /// The operating system's id for the server process.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    /// Sends a request and waits for its reply, for methods this crate does
    /// not wrap yet. A `null` `params` is omitted from the request.
    ///
    /// # Errors
    ///
    /// [`Error::Rpc`] if the server answers with an error; the session stays
    /// usable. Any other error is a transport problem: the process is killed
    /// and every later request fails.
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        let id = self.next_id;
        self.next_id += 1;
        let reply = call(&mut *self.stdin, &mut self.stdout, id, method, params);
        if reply.as_ref().is_err_and(Error::is_transport) {
            self.abort();
        }
        reply
    }

    /// Ends the session: sends `shutdown`, then `exit`, closes the server's
    /// stdin and waits for the process. Returns its exit status, which is
    /// success after a clean shutdown.
    ///
    /// # Errors
    ///
    /// The error of `shutdown` or `exit` if either fails (the process is
    /// still waited for), or [`Error::Io`] if waiting fails.
    pub fn close(mut self) -> Result<ExitStatus, Error> {
        self.finish()
    }

    fn initialize(&mut self) -> Result<ServerInfo, Error> {
        let result = self.request("initialize", initialize_params())?;
        server_info_from(result)
    }

    fn finish(&mut self) -> Result<ExitStatus, Error> {
        let goodbye = self
            .request("shutdown", Value::Null)
            .and_then(|_| notify(&mut *self.stdin, "exit"));
        self.stdin = Box::new(io::sink());
        self.closed = true;
        goodbye.and(self.child.wait().map_err(Error::Io))
    }

    /// Kills the process after a transport error; the stream is unusable.
    fn abort(&mut self) {
        self.closed = true;
        self.stdin = Box::new(io::sink());
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if !self.closed {
            drop(self.finish());
        }
    }
}
