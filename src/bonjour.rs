use async_dnssd::{RegisterData, TxtRecord, TxtRecordError};
use thiserror::Error;

pub type Service = async_dnssd::Registration;
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid record {0:?}")]
    InvalidRecord(TxtRecordError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub async fn register(
    reg_name: &str,
    reg_type: &str,
    domain: Option<&str>,
    host: Option<&str>,
    port: u16,
    txt: &[&str],
) -> Result<Service, Error> {
    let mut record = TxtRecord::new();
    for &entry in txt {
        let mut iter = entry.splitn(2, '=');
        let key = iter.next().unwrap().as_bytes();
        let val = iter.next().map(str::as_bytes);

        record.set(key, val).map_err(Error::InvalidRecord)?;
    }
    let (r, _) = async_dnssd::register_extended(
        reg_type,
        port,
        RegisterData {
            domain,
            host,
            name: Some(reg_name),
            txt: record.data(),
            ..Default::default()
        },
    )?
    .await?;
    Ok(r)
}
