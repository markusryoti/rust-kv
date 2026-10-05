use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::command::{Command, Method};

const MAX_PAYLOAD: usize = 4096;
const BUF_SIZE: usize = 1024;

pub struct KV {
    store: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Debug)]
pub enum KVError {
    BindError,
    IOError,
    CommandError(String),
    MutexErr,
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = match self {
            KVError::BindError => "bind error",
            KVError::IOError => "io error",
            KVError::MutexErr => "mutex error",
            KVError::CommandError(e) => e,
        };
        f.write_str(err)
    }
}

impl Error for KVError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl KV {
    pub fn new() -> Self {
        KV {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn listen(self) -> Result<(), KVError> {
        let listener = TcpListener::bind("127.0.0.1:8888").map_err(|_| KVError::BindError)?;

        for stream in listener.incoming() {
            let stream = stream.map_err(|_| KVError::IOError)?;
            self.handle_client(stream).map_err(|_| KVError::IOError)?
        }

        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), KVError> {
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

        let s = std::str::from_utf8(&bytes).map_err(|_| KVError::IOError)?;
        let c = Command::from(s).map_err(|e| KVError::CommandError(e.to_string()))?;

        let result = if let Ok(store) = &mut self.store.lock() {
            match c.method {
                Method::GET => store.get(&c.key).cloned(),
                Method::SET => {
                    let value = c.value.unwrap();
                    store.insert(c.key, value.clone());
                    Some(value)
                }
            }
        } else {
            return Err(KVError::MutexErr);
        };

        let result = result.unwrap_or("".into());

        let _ = stream
            .write(result.as_bytes())
            .map_err(|_| KVError::IOError)?;

        Ok(())
    }
}

impl Default for KV {
    fn default() -> Self {
        Self::new()
    }
}
