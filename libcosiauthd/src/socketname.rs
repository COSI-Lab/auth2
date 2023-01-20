use std::{net::SocketAddr, num::ParseIntError, str::FromStr};

/// SocketName represents a socket address as either a `std::net::SocketAddr` or a domain name + a
/// port.
///
/// For example:
/// `domain.com:1234`
/// `127.0.0.1:44`
#[derive(Debug, PartialEq, Eq)]
pub enum SocketName {
    Dns(String, u16),
    Addr(SocketAddr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SocketNameError {
    ParseIntError(ParseIntError),
    FormatError(),
}

impl std::fmt::Display for SocketNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(match self {
            SocketNameError::ParseIntError(err) => write!(f, "{}", err)?,
            SocketNameError::FormatError() => write!(
                f,
                "Doesn't match the either format. Expect something like 'auth.cosi.clarkson.edu:8765' or '128.153.145.3'."
            )?,
        })
    }
}

impl FromStr for SocketName {
    type Err = SocketNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match SocketAddr::from_str(s) {
            Ok(sa) => Ok(SocketName::Addr(sa)),
            Err(_) => {
                if s.contains(':') {
                    let mut comps = s.split(':');
                    let (l, r) = (comps.next().unwrap(), comps.next().unwrap());
                    Ok(SocketName::Dns(
                        l.into(),
                        u16::from_str(r).map_err(SocketNameError::ParseIntError)?,
                    ))
                } else {
                    Err(SocketNameError::FormatError())
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
