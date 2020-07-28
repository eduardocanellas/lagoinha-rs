//! CepLá service: http://cep.la/
//!
//! This service has an out os [spec](https://tools.ietf.org/html/rfc2616#section-4.2) header implementation,
//! and does not comply with the [RFC2616](https://tools.ietf.org/html/rfc2616#section-4.2).
//! This causes an issue when using it with libraries, like Hyper, because they parse all headers to lower case.
//! To solve this issue, the Curl library was used.

extern crate curl;
extern crate serde;
extern crate serde_json;

use crate::error::Error;
use crate::error::Kind;
use crate::error::Source::Cepla;

use curl::easy::{Easy, List};

use serde::{Deserialize, Serialize};

/// request function runs the API call to cepla service
pub async fn request(cep: &str) -> Result<Address, Error> {
    let mut requester = Easy::new();
    let uri = format!("http://cep.la/{}", cep);
    requester.url(&uri).or(Err(Error {
        kind: Kind::UnexpectedLibraryError,
        source: Cepla,
    }))?;

    let mut list = List::new();
    list.append("Accept: application/json").or(Err(Error {
        kind: Kind::UnexpectedLibraryError,
        source: Cepla,
    }))?;

    requester.http_headers(list).or(Err(Error {
        kind: Kind::UnexpectedLibraryError,
        source: Cepla,
    }))?;
    let mut buf = Vec::new();
    {
        let mut transfer = requester.transfer();
        transfer
            .write_function(|new_data| {
                buf.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .or(Err(Error {
                kind: Kind::MissingBodyError,
                source: Cepla,
            }))?;
        transfer.perform().or(Err(Error {
            kind: Kind::MissingBodyError,
            source: Cepla,
        }))?;
    }
    match requester.response_code() {
        Ok(code) => match code {
            200..=299 => (),
            400..=499 => {
                return Err(Error {
                    kind: Kind::ClientError { code: code as u16 },
                    source: Cepla,
                });
            }
            500..=599 => {
                return Err(Error {
                    kind: Kind::ServerError { code: code as u16 },
                    source: Cepla,
                });
            }
            _ => {
                return Err(Error {
                    kind: Kind::UnknownServerError { code: code as u16 },
                    source: Cepla,
                });
            }
        },
        Err(_) => {
            return Err(Error {
                kind: Kind::UnexpectedLibraryError,
                source: Cepla,
            });
        }
    }

    let address = serde_json::from_slice::<Address>(&buf);
    match address {
        Ok(address) => return Ok(address),
        Err(e) => {
            let str_body = std::str::from_utf8(&buf);
            let str_body = match str_body {
                Ok(str_body) => str_body,
                Err(_) => "Failed to produce string body ", //+  e.to_string().as_str()},
            };
            return Err(Error {
                kind: Kind::BodyParsingError {
                    error: e.to_string(),
                    body: str_body.to_string(),
                },
                source: Cepla,
            });
        }
    };
}

/// Address struct used to deserialize the results from the cepla API
#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    #[serde(rename = "cep", default = "String::new")]
    pub cep: String,
    #[serde(rename = "uf", default = "String::new")]
    pub state: String,
    #[serde(rename = "cidade", default = "String::new")]
    pub city: String,
    #[serde(rename = "bairro", default = "String::new")]
    pub neighborhood: String,
    #[serde(rename = "logradouro", default = "String::new")]
    pub address: String,
    #[serde(rename = "aux", default = "String::new")]
    pub details: String,
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn valid_cepla() {
        let resaddr = super::request("70150903").await.unwrap();

        let addr = super::Address {
            cep: "70150903".to_string(),
            state: "DF".to_string(),
            city: "Brasília".to_string(),
            neighborhood: "Zona Cívico-Administrativa".to_string(),
            address: "SPP".to_string(),
            details: "Palácio da Alvorada (Residência Oficial do Presidente da República)"
                .to_string(),
        };

        assert_eq!(addr.address, resaddr.address);
        assert_eq!(addr.state, resaddr.state);
        assert_eq!(addr.neighborhood, resaddr.neighborhood);
        assert_eq!(addr.city, resaddr.city);
        assert_eq!(addr.cep, resaddr.cep);
        assert_eq!(addr.details, resaddr.details);
    }

    #[tokio::test]
    async fn valid_cepla_with_dash() {
        let resaddr = super::request("70150-903").await.unwrap();

        let addr = super::Address {
            cep: "70150903".to_string(),
            state: "DF".to_string(),
            city: "Brasília".to_string(),
            neighborhood: "Zona Cívico-Administrativa".to_string(),
            address: "SPP".to_string(),
            details: "Palácio da Alvorada (Residência Oficial do Presidente da República)"
                .to_string(),
        };

        assert_eq!(addr.address, resaddr.address);
        assert_eq!(addr.state, resaddr.state);
        assert_eq!(addr.neighborhood, resaddr.neighborhood);
        assert_eq!(addr.city, resaddr.city);
        assert_eq!(addr.cep, resaddr.cep);
        assert_eq!(addr.details, resaddr.details);
    }

    use crate::error::Kind;
    use crate::error::Source;
    #[tokio::test]
    async fn invalid_input_viacep() {
        let resaddr = super::request("123").await;
        assert!(resaddr.is_err());
        resaddr
            .map_err(|err| {
                assert_eq!(err.source, Source::Cepla);
                assert_eq!(
                    std::mem::discriminant(&err.kind),
                    std::mem::discriminant(&Kind::BodyParsingError {
                        error: "".to_owned(),
                        body: "".to_owned(),
                    })
                );
            })
            .ok();
    }
}
