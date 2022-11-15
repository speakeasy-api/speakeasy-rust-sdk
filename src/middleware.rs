//! Contains the middleware for different web frameworks

#[cfg(feature = "actix3")]
pub mod actix3;

#[cfg(feature = "actix4")]
pub mod actix4;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "axum")]
pub mod host_extract;
