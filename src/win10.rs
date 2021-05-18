mod bindings {
    windows::include_bindings!();
}

use thiserror::Error;

use self::bindings::Windows::Networking::{
    HostName,
    ServiceDiscovery::Dnssd::{DnssdRegistrationStatus, DnssdServiceInstance},
    Sockets::DatagramSocket,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    WindowsError(#[from] windows::Error),
    #[error("Cannot register service: Invalid service name")]
    InvalidServiceName,
    #[error("Cannot register service: Server error")]
    ServerError,
    #[error("Cannot register service: Security error")]
    SecurityError,
    #[error("Cannot register service: Unknown error")]
    UnkownError,
}

pub type Service = (DnssdServiceInstance, DatagramSocket);

pub async fn register(
    reg_name: &str,
    reg_type: &str,
    domain: Option<&str>,
    host: Option<&str>,
    port: u16,
    txt: &[&str],
) -> Result<Service, Error> {
    let domain = domain.unwrap_or("local");
    let instance_name = format!("{}.{}.{}.", reg_name, reg_type, domain);
    let host = host.map(HostName::CreateHostName).transpose()?;
    let instance = DnssdServiceInstance::Create(instance_name, host, port)?;

    let txt_map = instance.TextAttributes()?;
    for &entry in txt {
        let mut iter = entry.splitn(2, '=');
        let key = iter.next().unwrap();
        let val = iter.next().unwrap_or("");
        txt_map.Insert(key, val)?;
    }

    let socket = DatagramSocket::new()?;
    let result = instance
        .RegisterDatagramSocketAsync1(&socket)?
        .await?
        .Status()?;

    match result {
        DnssdRegistrationStatus::Success => (),
        DnssdRegistrationStatus::InvalidServiceName => return Err(Error::InvalidServiceName),
        DnssdRegistrationStatus::SecurityError => return Err(Error::SecurityError),
        DnssdRegistrationStatus::ServerError => return Err(Error::ServerError),
        _ => return Err(Error::UnkownError),
    }

    Ok((instance, socket))
}
