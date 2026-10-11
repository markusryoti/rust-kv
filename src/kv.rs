use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use tracing::{error, info, info_span, instrument};
use uuid::Uuid;

use crate::{
    command::{Command, Method},
    store::Store,
    thread_pool::ThreadPool,
};

const MAX_PAYLOAD: usize = 4096;
const BUF_SIZE: usize = 1024;

#[derive(Debug)]
pub struct KV {
    store: Arc<Store>,
    pool: ThreadPool,
}

#[derive(Debug)]
pub enum KVError {
    BindError,
    IOError,
    CommandError(String),
    SetError,
    EnqueueError,
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            KVError::BindError => "bind error",
            KVError::IOError => "io error",
            KVError::CommandError(e) => e,
            KVError::SetError => "set error",
            KVError::EnqueueError => "enqueue error",
        };
        f.write_str(err)
    }
}

impl KV {
    pub fn new(pool_size: usize) -> Self {
        let pool = ThreadPool::new(pool_size);

        KV {
            pool,
            store: Arc::new(Store::new()),
        }
    }

    pub fn listen(self) -> Result<(), KVError> {
        info!("Starting to listen");

        self.pool.start();

        let listener = TcpListener::bind("127.0.0.1:8888").map_err(|_| KVError::BindError)?;
        info!("Listener binded to a port");

        for stream in listener.incoming() {
            let stream = stream.map_err(|_| KVError::IOError)?;
            let store = Arc::clone(&self.store);

            let f = move || {
                let span = info_span!("client_request", "corr_id" = Uuid::new_v4().to_string());
                let _guard = span.enter();

                let _ = handle_client(stream, store).inspect_err(|e| {
                    error!("request failed: {}", e);
                });
            };

            self.pool
                .enqueue(Box::new(f))
                .map_err(|_| KVError::EnqueueError)?;
        }

        Ok(())
    }
}

impl Default for KV {
    fn default() -> Self {
        Self::new(1)
    }
}

#[instrument(skip(stream, store))]
fn handle_client(mut stream: TcpStream, store: Arc<Store>) -> Result<(), KVError> {
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

    let str_payload = std::str::from_utf8(&bytes).map_err(|_| KVError::IOError)?;
    let cmd = Command::from(str_payload).map_err(|e| KVError::CommandError(e.to_string()))?;

    info!(method = cmd.method.to_string(), "command parsed");

    let result = match cmd.method {
        Method::GET => store.get(&cmd.key).map_err(|_| KVError::IOError)?,
        Method::SET => store
            .set(cmd.key.clone(), cmd.value.unwrap_or("".into()))
            .map_err(|_| KVError::IOError)?,
    };

    info!("store operation done");

    let n_bytes = stream
        .write(result.as_bytes())
        .map_err(|_| KVError::IOError)?;

    info!(num_bytes = n_bytes, "result written to client socket");

    Ok(())
}
