#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", .0)]
    Api(windows::Error),
}

impl From<windows::HRESULT> for Error {
    fn from(src: windows::HRESULT) -> Self {
        Self::Api(src.into())
    }
}

impl From<windows::Error> for Error {
    fn from(src: windows::Error) -> Self {
        Self::Api(src)
    }
}
