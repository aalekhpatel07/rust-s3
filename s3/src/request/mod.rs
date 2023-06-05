#[cfg(feature = "with-async-std")]
mod async_std_backend;
#[cfg(feature = "with-async-std")]
pub use async_std_backend::*;

#[cfg(feature = "with-tokio")]
mod tokio_backend;
#[cfg(feature = "with-tokio")]
pub use tokio_backend::*;


#[cfg(feature = "sync")]
mod blocking;
#[cfg(feature = "sync")]
pub use blocking::*;

mod request_trait;
pub use request_trait::*;

#[cfg(any(feature = "with-tokio", feature = "with-async-std"))]
mod async_common;

#[cfg(any(feature = "with-tokio", feature = "with-async-std"))]
pub use async_common::*;


#[cfg(feature = "with-tokio")]
pub(crate) use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[cfg(feature = "with-async-std")]
pub(crate) use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite};