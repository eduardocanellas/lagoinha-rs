#![crate_name = "lagoinha"]
//! Lagoinha is a library that retrieve Addresses from the Brazilian Postal Code (CEP) using multiple APIs asynchronously, and returns the result from the first one to respond.
//! It uses async/.await and the Futures library for its asyncronous features, and can be used with most runtimes.
//!
//! # Services
//!
//! Currenlty the services used are : correios, viacep and cepla
//! It is expected to support adding a custom service to the pool in the future, and the ability to disable the default ones.
//!
//! While the default http library is Hyper, the CepLá service has an issue with its header implementation, and so the curl library was used. More information in the docs for this service.
//!
//! # Example
//! ```
//!extern crate lagoinha;
//!extern crate tokio;
//!
//!#[tokio::main]
//!async fn main() {    
//!    let addr = lagoinha::get_address("70150903").await;
//!    println!("{:#?}", addr);
//!}
//!```
//!

pub mod error;
pub mod services;
use error::Error;
use error::Source::LagoinhaLib;
use services::Address;

use futures::channel::mpsc;
use futures::{future::FutureExt, sink::SinkExt};

const SEND_ERROR: &str =
    "Failed awaiting channel send. This should not happen. Please contact the developer";

async fn viacep_requet(cep: &str, mut tx: mpsc::Sender<Result<services::Address, Error>>) {
    let addr = services::viacep::request(cep).await;
    match addr {
        Ok(addr) => {
            tx.send(Ok(addr.to_address()))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
        Err(err) => {
            tx.send(Err(err))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
    }
}

async fn cepla_requet(cep: &str, mut tx: mpsc::Sender<Result<services::Address, Error>>) {
    let addr = services::cepla::request(cep).await;
    match addr {
        Ok(addr) => {
            tx.send(Ok(addr.to_address()))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
        Err(err) => {
            tx.send(Err(err))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
    }
}

async fn correios_requet(cep: &str, mut tx: mpsc::Sender<Result<services::Address, Error>>) {
    let addr = services::correios::request(cep).await;
    match addr {
        Ok(addr) => {
            tx.send(Ok(addr.to_address()))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
        Err(err) => {
            tx.send(Err(err))
                .await
                .map_err(|e| println!("{} with error: {}", SEND_ERROR, e.to_string()))
                .ok();
        }
    }
}

pub async fn get_address(cep: &str) -> Result<Address, Error> {
    let (tx, mut rx) = mpsc::channel::<Result<services::Address, Error>>(1);

    futures::select! {
        () = viacep_requet(cep, tx.clone()).fuse() => "viacep",
        () = cepla_requet(cep, tx.clone()).fuse() => "cepla",
        () = correios_requet(cep, tx.clone()).fuse() => "correios",
        default => unreachable!()
    };

    let mut error_list: Vec<Error> = Vec::new();

    for _ in 0..2 {
        let read = rx.try_next();
        match read {
            Ok(read_address) => match read_address {
                Some(read_address) => match read_address {
                    Ok(addr) => return Ok(addr),
                    Err(e) => error_list.push(e),
                },
                None => error_list.push(Error {
                    kind: error::Kind::UnexpectedLibraryError,
                    source: LagoinhaLib,
                }),
            },
            Err(_) => {
                return Err(Error {
                    kind: error::Kind::UnexpectedLibraryError,
                    source: LagoinhaLib,
                })
            }
        };
    }

    Err(Error {
        source: error::Source::LagoinhaLib,
        kind: error::Kind::AllServicesRetunedErrors {
            e1: format!("{}", error_list[0]),
            e2: format!("{}", error_list[1]),
            e3: format!("{}", error_list[2]),
        },
    })
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_channels() {
        let addr = super::services::Address {
            cep: "70150903".to_string(),
            address: "SPP".to_string(),
            details: "".to_string(),
            neighborhood: "Zona Cívico-Administrativa".to_string(),
            city: "Brasília".to_string(),
            state: "DF".to_string(),
        };

        let recv_addr = super::get_address("70150903").await.unwrap();
        assert_eq!(addr.city, recv_addr.city);
        assert_eq!(addr.state, recv_addr.state);
        assert_eq!(addr.neighborhood, recv_addr.neighborhood);
        // the other fields, like cep can come with different formating
    }
}
