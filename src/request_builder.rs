use std::collections::HashMap;

use reqwest::{header::HeaderValue, Client, Method, Request, Response, Url};
use serde::Serialize;

use crate::error::JellyfinError;

pub struct JellyfinRequestBuilder<'a> {
    client: &'a Client,
    request: Request,
}

impl<'a> JellyfinRequestBuilder<'a> {
    pub fn new(client: &'a Client, method: Method, url: Url) -> Self {
        let mut request = Request::new(method, url);

        request.headers_mut().insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_static("Rust Jellyfin Library"),
        );

        request.headers_mut().insert(
            "X-Emby-Authorization",
            HeaderValue::from_str(&format!(
                "MediaBrowser Client=JellyRust,Device={},DeviceId={},Version={}",
                hostname::get().unwrap().to_str().unwrap(),
                "69",
                env!("CARGO_PKG_VERSION"),
            ))
            .expect("Invalid header values"),
        );

        Self { client, request }
    }

    pub fn json<T: Serialize>(mut self, json: &T) -> Result<Self, JellyfinError> {
        let body = serde_json::to_string(json)?;

        *self.request.body_mut() = Some(body.into());

        self.request.headers_mut().insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        Ok(self)
    }

    pub fn auth(mut self, token: &str) -> Self {
        self
            .request
            .headers_mut()
            .insert("X-Emby-Token", token.to_owned().parse().unwrap());

        self
    }

    pub fn query(mut self, params: HashMap<&str, String>) -> Self {
        {
            let mut query_pairs = self.request.url_mut().query_pairs_mut();

            for (key, value) in params {
                query_pairs.append_pair(key, &value);
            }
        }

        self
    }

    pub async fn send(self) -> Result<Response, JellyfinError> {
        tracing::info!("{} {}", self.request.method(), self.request.url());

        let response = self.client.execute(self.request).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            Err(JellyfinError::from(response.status()))
        } else {
            Ok(response)
        }
    }
}
