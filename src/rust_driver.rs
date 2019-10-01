use crate::traits::Driver;
use failure::{format_err, Error};
use rayon::prelude::*;
use reqwest::{header, Client, Url};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RustDriver {
    base_url: Url,
    client: Client,
}

impl Driver for RustDriver {
    fn get_info(self, data: &str) -> String {
        let mut deps = vec![];
        let mut in_deps = false;
        for line in data.split('\n') {
            if line.trim() == "[dependencies]" {
                in_deps = true;
                continue;
            }
            if line.trim().starts_with('[') {
                break;
            }
            if !in_deps {
                continue;
            }
            if let Some(split_index) = line.find('=') {
                deps.push(line.split_at(split_index).0.trim());
            }
        }

        let descriptions: Vec<String> = deps
            .par_iter()
            .map(|name| get_crate_description(&self.client, &self.base_url, name))
            .collect();

        // ...

        unimplemented!()
    }
}

impl RustDriver {
    pub fn new(user_agent: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(user_agent).unwrap(),
        );
        Self {
            base_url: Url::parse("https://crates.io/api/v1/").unwrap(),
            client: Client::builder().default_headers(headers).build().unwrap(),
        }
    }
}

fn get_crate_info(client: &Client, base_url: &Url, name: &str) -> Result<Value, Error> {
    let mut resp = client.get(&format!("{}{}", base_url, name)).send()?;
    if !resp.status().is_success() {
        return Err(format_err!(
            "Bad status {} from crates.io API",
            resp.status()
        ));
    }
    let json: Value = resp.json()?;
    Ok(json)
}

fn get_crate_description(client: &Client, base_url: &Url, name: &str) -> String {
    let json: Value = match get_crate_info(client, base_url, name) {
        Ok(j) => j,
        Err(_) => return String::from("-- unknown --"),
    };
    let description = match json["crate"]["description"].as_str() {
        Some(d) => d,
        None => "-- unknown --",
    };
    description.to_owned()
}
