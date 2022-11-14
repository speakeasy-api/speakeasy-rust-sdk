#[cfg(feature = "tokio02")]
mod tokio02 {
    mod ingest_service_client;

    pub mod ingest {
        pub mod ingest_service_client {
            pub use crate::speakeasy_protos::tokio02::ingest_service_client::IngestServiceClient;
        }

        pub use speakeasy_protos_tokio_02::ingest::*;
    }

    pub use speakeasy_protos_tokio_02::embedaccesstoken;
}

#[cfg(feature = "tokio02")]
pub use self::tokio02::*;

#[cfg(feature = "tokio")]
mod tokio {
    pub mod ingest {
        pub use speakeasy_protos_tokio_latest::ingest::*;
    }
    pub use speakeasy_protos_tokio_latest::embedaccesstoken;
}

#[cfg(feature = "tokio")]
pub use self::tokio::*;
