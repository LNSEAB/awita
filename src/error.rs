use windows::core::HRESULT;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error::Api: {}", .0)]
    Api(windows::core::Error),
    #[error("closed")]
    Closed,
}

impl From<HRESULT> for Error {
    fn from(src: HRESULT) -> Self {
        Self::Api(src.into())
    }
}

impl From<windows::core::Error> for Error {
    fn from(src: windows::core::Error) -> Self {
        Self::Api(src)
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
        Self::Closed
    }
}

impl From<async_broadcast::RecvError> for Error {
    fn from(_: async_broadcast::RecvError) -> Self {
        Self::Closed
    }
}
