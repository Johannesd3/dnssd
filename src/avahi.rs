use std::convert::Infallible;
use std::time::Duration;

use dbus::nonblock::Proxy;
use dbus::Path;
use log::warn;
use thiserror::Error;
use tokio::select;
use tokio::sync::oneshot;
use tokio::task::{spawn, spawn_blocking};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Dbus error: {0}")]
    DbusError(#[from] dbus::Error),
}

pub type Service = oneshot::Sender<Infallible>;

pub async fn register(
    reg_name: &str,
    reg_type: &str,
    domain: Option<&str>,
    host: Option<&str>,
    port: u16,
    txt: &[&str],
) -> Result<Service, Error> {
    let txt: Vec<&[u8]> = txt.iter().map(|x| x.as_bytes()).collect();
    let (resource, conn) = spawn_blocking(dbus_tokio::connection::new_system_sync)
        .await
        .unwrap()?;

    let (close_tx, close_rx) = oneshot::channel::<Infallible>();

    spawn(async move {
        select! {
            err = resource => {
                warn!("Lost connection to dbus: {}", err);
            },
            _ = close_rx => {

            }
        }
    });

    let mut proxy = Proxy::new(
        "org.freedesktop.Avahi",
        "/",
        Duration::from_secs(2),
        conn.clone(),
    );

    let (path,): (Path<'_>,) = proxy
        .method_call("org.freedesktop.Avahi.Server", "EntryGroupNew", ())
        .await?;
    proxy.path = path;

    let () = proxy
        .method_call(
            "org.freedesktop.Avahi.EntryGroup",
            "AddService",
            (
                -1_i32,
                0_i32,
                0_u32,
                reg_name,
                reg_type,
                domain.unwrap_or(""),
                host.unwrap_or(""),
                port,
                &txt,
            ),
        )
        .await?;

    let () = proxy
        .method_call("org.freedesktop.Avahi.EntryGroup", "Commit", ())
        .await?;

    Ok(close_tx)
}
