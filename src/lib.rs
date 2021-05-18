use std::marker::PhantomData;

use thiserror::Error;

#[cfg(windows)]
mod win10;
#[cfg(windows)]
use win10 as imp;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod bonjour;
#[cfg(any(target_os = "macos", target_os = "ios"))]
use bonjour as imp;

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios")))]
mod avahi;
#[cfg(all(unix, not(target_os = "macos"), not(target_os = "ios")))]
use avahi as imp;

// The PhantomData should guarantee the same trait bounds on all platforms.
#[derive(Debug, Error)]
#[error("{0}")]
pub struct Error(imp::Error, PhantomData<std::io::Error>);

pub struct Service(imp::Service, PhantomData<&'static mut std::cell::Cell<()>>);

pub async fn register(
    reg_name: &str,
    reg_type: &str,
    domain: Option<&str>,
    host: Option<&str>,
    port: u16,
    txt: &[&str],
) -> Result<Service, Error> {
    imp::register(reg_name, reg_type, domain, host, port, txt)
        .await
        .map(|s| Service(s, PhantomData))
        .map_err(|e| Error(e, PhantomData))
}
