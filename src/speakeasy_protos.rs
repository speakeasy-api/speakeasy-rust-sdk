#[cfg(feature = "tokio02")]
mod tokio02 {
    pub use speakeasy_protos_tokio_02::embedaccesstoken;
    pub use speakeasy_protos_tokio_02::ingest;
}

#[cfg(feature = "tokio02")]
pub use self::tokio02::*;

#[cfg(feature = "tokio")]
mod tokio {
    pub use speakeasy_protos_tokio_latest::embedaccesstoken;
    pub use speakeasy_protos_tokio_latest::ingest;
}

#[cfg(feature = "tokio")]
pub use self::tokio::*;
