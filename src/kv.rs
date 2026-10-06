use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, info_span, instrument};
use uuid::Uuid;

use crate::{
    command::{Command, Method},
    store::Store,
};

const MAX_PAYLOAD: usize = 4096;
const BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub struct KV {
    store: Store,
}

#[derive(Debug)]
pub enum KVError {
    BindError,
    IOError,
    CommandError(String),
    MutexErr,
    SetError,
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            KVError::BindError => "bind error",
            KVError::IOError => "io error",
            KVError::MutexErr => "mutex error",
            KVError::CommandError(e) => e,
            KVError::SetError => "set error",
        };
        f.write_str(err)
    }
}

impl KV {
    pub fn new() -> Self {
        KV {
            store: Store::new(),
        }
    }

    pub fn listen(self) -> Result<(), KVError> {
        info!("Starting to listen");

        let listener = TcpListener::bind("127.0.0.1:8888").map_err(|_| KVError::BindError)?;

        info!("Listener binded to a port");

        for stream in listener.incoming() {
            let stream = stream.map_err(|_| KVError::IOError)?;

            let span = info_span!("client_request", "corr_id" = Uuid::new_v4().to_string());
            let _guard = span.enter();

            let _ = self.handle_client(stream).inspect_err(|e| {
                error!("request failed: {}", e);
            });
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    fn handle_client(&self, mut stream: TcpStream) -> Result<(), KVError> {
        info!("request handling start");

        let mut bytes: Vec<u8> = vec![];

        let mut read_bytes = 0;
        while read_bytes < MAX_PAYLOAD {
            let mut buf = vec![0; BUF_SIZE];
            let n_bytes = stream.read(&mut buf).map_err(|_| KVError::IOError)?;
            if n_bytes == 0 {
                break;
            }

            bytes.append(&mut buf[..n_bytes].to_vec());
            read_bytes += n_bytes;

            if n_bytes < BUF_SIZE {
                break;
            }
        }

        info!(num_bytes = read_bytes, "request read");

        let s = std::str::from_utf8(&bytes).map_err(|_| KVError::IOError)?;
        let c = Command::from(s).map_err(|e| KVError::CommandError(e.to_string()))?;

        info!(method = c.method.to_string(), "command parsed");

        let result = match c.method {
            Method::GET => self.store.get(&c.key).map_err(|_| KVError::IOError)?,
            Method::SET => self
                .store
                .set(c.key.clone(), c.value.unwrap_or("".into()))
                .map_err(|_| KVError::IOError)?,
        };

        info!("store operation done");

        let n_bytes = stream
            .write(result.as_bytes())
            .map_err(|_| KVError::IOError)?;

        info!(num_bytes = n_bytes, "result written to client socket");

        Ok(())
    }
}

impl Default for KV {
    fn default() -> Self {
        Self::new()
    }
}
