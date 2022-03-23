pub mod error;
pub mod model;
pub mod request_builder;

use error::JellyfinError;
use model::{Audio, ImageType, Item, ItemType, ItemsResponse, MusicAlbum, UserInfo};
use request_builder::JellyfinRequestBuilder;
use reqwest::{header::HeaderValue, Client, Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::model::AuthResponse;

#[derive(Clone)]
pub struct JellyfinApi {
    http: Client,
    server_url: Arc<Url>,
    auth_info: Option<Arc<AuthInfo>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthInfo {
    pub token: String,
    pub user_id: String,
}

impl JellyfinApi {
    pub fn new(server_url: &str) -> Result<Self, JellyfinError> {
        let http = Client::new();
        let url = Url::parse(server_url)?;

        Ok(Self {
            http,
            server_url: Arc::new(url),
            auth_info: None,
        })
    }

    pub async fn new_with_password(
        server_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, JellyfinError> {
        let mut api = Self::new(server_url)?;

        api.authenticate_with_password(username, password).await?;

        Ok(api)
    }

    pub async fn new_with_token(server_url: &str, token: String) -> Result<Self, JellyfinError> {
        let mut api = Self::new(server_url)?;

        api.authenticate_with_token(token).await?;

        Ok(api)
    }
    
    pub fn new_with_auth_info(server_url: &str, auth_info: AuthInfo) -> Result<Self, JellyfinError> {
        let mut api = Self::new(server_url)?;
        api.auth_info = Some(Arc::new(auth_info));
        Ok(api)
    }

    pub async fn authenticate_with_token(&mut self, token: String) -> Result<(), JellyfinError> {
        let user_info: UserInfo = self
            .get("/Users/Me")
            .auth(&token)
            .send()
            .await?
            .json()
            .await?;

        tracing::debug!("Authenticated as: {:?}", user_info);

        self.auth_info = Some(Arc::new(AuthInfo {
            token,
            user_id: user_info.id,
        }));

        Ok(())
    }

    pub async fn authenticate_with_password(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), JellyfinError> {
        let x_emby_authorization = format!(
            "MediaBrowser Client=jellyfin-rust,Device={},DeviceId={},Version={}",
            hostname::get().unwrap().to_str().unwrap(),
            Uuid::new_v4(),
            env!("CARGO_PKG_VERSION"),
        );
        tracing::debug!("X-Emby-Authorization: {}", x_emby_authorization);

        let auth_response: AuthResponse = self
            .post("/Users/AuthenticateByName")
            .json(&json!({
                "Username": username,
                "Pw": password,
            }))?
            .header(
                "X-Emby-Authorization",
                HeaderValue::from_str(&x_emby_authorization).unwrap(),
            )
            .send()
            .await?
            .json()
            .await?;
        tracing::debug!("Successfully authenticated: {:?}", auth_response);

        self.auth_info = Some(Arc::new(AuthInfo {
            token: auth_response.access_token,
            user_id: auth_response.user.id,
        }));

        Ok(())
    }

    pub fn get_auto_info(&self) -> Option<&AuthInfo> {
        match &self.auth_info {
            Some(auth_info) => Some(&*auth_info),
            None => None,
        }
    }

    pub async fn ping(&self) -> Result<(), JellyfinError> {
        if self.get("/System/Ping").send().await?.text().await? == "\"Jellyfin Server\"" {
            Ok(())
        } else {
            Err(JellyfinError::ServerError)
        }
    }

    fn make_request(&self, method: Method, path: &str) -> JellyfinRequestBuilder<'_> {
        let builder = JellyfinRequestBuilder::new(
            &self.http,
            method,
            self.server_url.join(path).expect("Invalid path"),
        );

        if let Some(auth_info) = &self.auth_info {
            builder.auth(&auth_info.token)
        } else {
            builder
        }
    }

    fn get(&self, path: &str) -> JellyfinRequestBuilder<'_> {
        self.make_request(Method::GET, path)
    }

    fn post(&self, path: &str) -> JellyfinRequestBuilder<'_> {
        self.make_request(Method::POST, path)
    }

    pub async fn get_items(
        &self,
        parent_id: Option<&str>,
        item_types: Vec<ItemType>,
        recursive: bool,
        limit: u32,
        extra_params: Vec<(&str, String)>,
    ) -> Result<ItemsResponse<ItemType>, JellyfinError> {
        let mut params = HashMap::new();

        if let Some(parent_id) = parent_id {
            params.insert("ParentId", parent_id.to_owned());
        }

        if !item_types.is_empty() {
            let item_types: Vec<String> = item_types
                .into_iter()
                .map(|item| item.as_str().to_owned())
                .collect();

            params.insert("IncludeItemTypes", item_types.join(","));
        }

        if recursive {
            params.insert("Recursive", "True".to_owned());
        }

        params.insert("Limit", limit.to_string());

        for (k, v) in extra_params {
            params.insert(k, v);
        }
        tracing::trace!("request params: {:?}", params);

        Ok(self
            .get(&format!(
                "/Users/{}/Items",
                self.auth_info
                    .as_ref()
                    .ok_or_else(|| JellyfinError::AuthorizationError)?
                    .user_id
            ))
            .query(params)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_artists(
        &self,
        library_id: Option<&str>,
    ) -> Result<ItemsResponse<Item>, JellyfinError> {
        let mut params = HashMap::new();

        if let Some(parent_id) = library_id {
            params.insert("ParentId", parent_id.to_owned());
        }

        Ok(self
            .get("/Artists")
            .query(params)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_albums(
        &self,
        artist_ids: Vec<&str>,
    ) -> Result<Vec<MusicAlbum>, JellyfinError> {
        let response = self
            .get_items(
                None,
                vec![ItemType::MusicAlbum(MusicAlbum::default())],
                true,
                200,
                vec![("ArtistIds", artist_ids.join(","))],
            )
            .await?;

        response
            .items
            .into_iter()
            .map(|item| match item {
                ItemType::MusicAlbum(album) => Ok(album),
                _ => Err(JellyfinError::ServerError),
            })
            .collect()
    }

    pub async fn get_playlist_items(&self, playlist_id: &str) -> Result<Vec<Audio>, JellyfinError> {
        let user_id = self
            .auth_info
            .as_ref()
            .ok_or_else(|| JellyfinError::AuthorizationError)?
            .user_id
            .clone();

        let mut params = HashMap::new();
        params.insert("UserId", user_id);

        let response: ItemsResponse<Audio> = self
            .get(&format!("/Playlists/{}/Items", playlist_id))
            .query(params)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.items)
    }

    /// Size in (width, height)
    pub async fn get_item_image(
        &self,
        item_id: &str,
        image_type: ImageType,
        max_size: (u32, u32),
    ) -> Result<Option<Vec<u8>>, JellyfinError> {
        let mut params = HashMap::new();

        params.insert("maxWidth", max_size.0.to_string());
        params.insert("maxHeight", max_size.1.to_string());

        match self
            .get(&format!(
                "/Items/{}/Images/{}",
                item_id,
                image_type.as_str()
            ))
            .query(params)
            .send()
            .await
        {
            Ok(response) => Ok(Some(response.bytes().await?.to_vec())),
            Err(JellyfinError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn get_views(&self) -> Result<ItemsResponse<Item>, JellyfinError> {
        Ok(self
            .get(&format!(
                "/Users/{}/Views",
                self.auth_info
                    .as_ref()
                    .ok_or_else(|| JellyfinError::AuthorizationError)?
                    .user_id
            ))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_audio_stream(
        &self,
        item_id: &str,
    ) -> Result<reqwest::Response, JellyfinError> {
        let mut params = HashMap::new();

        params.insert("Container", "flac,mp3,ogg,wav".to_string());
        params.insert(
            "UserId",
            self.auth_info
                .as_ref()
                .ok_or_else(|| JellyfinError::AuthorizationError)?
                .user_id
                .clone(),
        );

        self.get(&format!("/Audio/{}/universal", item_id))
            .query(params)
            .send()
            .await
    }
}
