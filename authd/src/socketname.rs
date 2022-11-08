use std::{net::SocketAddr, str::FromStr};

/// SocketName represents either addr 
/// `"domain.com:port"`
/// or 
/// `"ipaddress:port"`
#[derive(Debug, PartialEq, Eq)]
pub enum SocketName {
    Dns(String, u16),
    Addr(SocketAddr),
}

impl FromStr for SocketName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match SocketAddr::from_str(s) {
            Ok(sa) => Ok(SocketName::Addr(sa)),
            Err(_) => {
                if s.contains(':') {
                    let mut comps = s.split(':');
                    let (l, r) = (comps.next().unwrap(), comps.next().unwrap());
                    Ok(SocketName::Dns(
                        l.into(),
                        u16::from_str(r).map_err(|e| anyhow::anyhow!(e.to_string()))?,
                    ))
                } else {
                    Err(anyhow::anyhow!("not a socket addr & missing port for dns, example: auth.cosi.clarkson.edu:8765"))
                }
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for SocketName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        SocketName::from_str(&s).map_err(|e| D::Error::custom(e.to_string()))
    }
}

impl std::net::ToSocketAddrs for SocketName {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(match self {
            SocketName::Dns(host, port) => (host.as_str(), *port)
                .to_socket_addrs()?
                .into_iter()
                .collect::<Vec<_>>()
                .into_iter(),
            SocketName::Addr(sa) => vec![*sa].into_iter(),
        })
    }
}

