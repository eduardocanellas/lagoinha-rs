//! CepLá service: http://cep.la/
//! 
//! This service has an out os [spec](https://tools.ietf.org/html/rfc2616#section-4.2) header implementation,
//! and does not comply with the [RFC2616](https://tools.ietf.org/html/rfc2616#section-4.2).
//! This causes an issue when using it with libraries, like Hyper, because they parse all headers to lower case.  
//! To solve this issue, the Curl library was used.
 
extern crate curl;
extern crate serde_json;
extern crate serde;

use curl::easy::{Easy, List};

use serde::{Serialize, Deserialize};

/// request function runs the API call to cepla service
pub async fn request(cep : &str) -> Result<Address, hyper::Error>{
    let mut requester = Easy::new();
    let uri = format!("http://cep.la/{}",cep);
    requester.url(&uri).unwrap();
    let mut list = List::new();
    list.append("Accept: application/json").unwrap();
    requester.http_headers(list).unwrap();
    let mut buf = Vec::new();
    {
        let mut transfer = requester.transfer();
        transfer.write_function(|new_data| {
            buf.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let address = serde_json::from_slice::<Address>(&buf).unwrap();
    return Ok(address)
}

/// Address struct used to deserialize the results from the cepla API
#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    #[serde(rename = "cep")]
    pub cep: String,
    #[serde(rename = "uf")]
    pub state: String,
    #[serde(rename = "cidade")]
    pub city: String,
    #[serde(rename = "bairro")]
    pub neighborhood: String,
    #[serde(rename = "logradouro")]
    pub address: String,
    #[serde(rename = "aux")]
    pub details: String,
}

#[cfg(test)]
mod tests {
    // use viacep;
    #[tokio::test]
    async fn valid_cepla() {
        let resaddr = super::request("70150903").await.unwrap();
        
        let addr = super::Address {
            cep: "70150903".to_string(),
            state: "DF".to_string(),
            city: "Brasília".to_string(),
            neighborhood: "Zona Cívico-Administrativa".to_string(),
            address: "SPP".to_string(),
            details: "Palácio da Alvorada (Residência Oficial do Presidente da República)".to_string(),
        };

        assert_eq!(addr.address, resaddr.address);
        assert_eq!(addr.state, resaddr.state);
        assert_eq!(addr.neighborhood, resaddr.neighborhood);
        assert_eq!(addr.city, resaddr.city);
        assert_eq!(addr.cep, resaddr.cep);
        assert_eq!(addr.details, resaddr.details);
    }
}
