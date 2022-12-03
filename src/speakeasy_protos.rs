#[cfg(feature = "tokio02")]
mod tokio02 {
    mod embed_access_token_client;
    mod ingest_service_client;

    pub mod ingest {
        pub mod ingest_service_client {
            pub use crate::speakeasy_protos::tokio02::ingest_service_client::IngestServiceClient;
        }

        pub use speakeasy_protos_tokio_02::ingest::*;
    }

    pub mod embedaccesstoken {
        pub mod embed_access_token_service_client {
            pub use crate::speakeasy_protos::tokio02::embed_access_token_client::EmbedAccessTokenServiceClient;
        }

        pub use speakeasy_protos_tokio_02::embedaccesstoken::*;
    }
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
