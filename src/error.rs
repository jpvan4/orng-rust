use thiserror::Error;
use std::sync::mpsc;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RandomX error: {0}")]
    RandomX(#[from] randomx_rs::RandomXError),

    #[error("Channel error: {0}")]
    Channel(String),

    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Thread error: {0}")]
    Thread(String),

    #[error("Stratum error: {0}")]
    Stratum(String),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl<T> From<mpsc::SendError<T>> for Error {
    fn from(err: mpsc::SendError<T>) -> Self {
        Error::Channel(format!("Failed to send data: {}", err))
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Self {
        Error::Channel(format!("Failed to receive data: {}", err))
    }
}

impl From<mpsc::TryRecvError> for Error {
    fn from(err: mpsc::TryRecvError) -> Self {
        match err {
            mpsc::TryRecvError::Empty => Error::Channel("Channel is empty".to_string()),
            mpsc::TryRecvError::Disconnected => {
                Error::Channel("Channel is disconnected".to_string())
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "fail");
        let our_err: Error = io_err.into();
        match our_err {
            Error::Io(_) => (),
            _ => panic!("Expected IO error"),
        }
    }

    #[test]
    fn test_channel_send_error_conversion() {
        let (tx, rx) = mpsc::channel::<i32>();
        drop(rx);
        let send_res = tx.send(1);
        assert!(send_res.is_err());
        let our_err: Error = send_res.unwrap_err().into();
        matches!(our_err, Error::Channel(_));
    }

    #[test]
    fn test_channel_recv_error_conversion() {
        let (tx, rx) = mpsc::channel::<i32>();
        drop(tx);
        let recv_res = rx.recv();
        assert!(recv_res.is_err());
        let our_err: Error = recv_res.unwrap_err().into();
        matches!(our_err, Error::Channel(_));
    }

    #[test]
    fn test_try_recv_error_conversion() {
        let (_tx, rx) = mpsc::channel::<i32>();
        let res = rx.try_recv();
        assert!(res.is_err());
        let our_err: Error = res.unwrap_err().into();
        matches!(our_err, Error::Channel(_));
    }

    #[test]
    fn test_json_error_conversion() {
        let bad = "{invalid";
        let res: serde_json::Result<serde_json::Value> = serde_json::from_str(bad);
        assert!(res.is_err());
        let our_err: Error = res.unwrap_err().into();
        matches!(our_err, Error::Json(_));
    }

    #[test]
    fn test_hex_decode_error_conversion() {
        let res = hex::decode("bad");
        assert!(res.is_err());
        let our_err: Error = res.unwrap_err().into();
        matches!(our_err, Error::HexDecode(_));
    }
}
