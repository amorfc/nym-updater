use reqwest::{header, Client, Method, StatusCode};
use serde::de::DeserializeOwned;

pub struct AppClient {
    base_url: String,
    client: Client,
}

impl AppClient {
    pub fn new(base_url: String) -> Self {
        AppClient {
            base_url,
            client: Client::new(),
        }
    }

    async fn send_request<T: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        body: Option<String>,
    ) -> Result<RestResponse<T>, String> {
        let request_builder = self.client.request(method, self.full_url(url));
        let request_builder = request_builder.header(header::USER_AGENT, "nym-updater/0.1.0");
        let request_builder = match body {
            Some(b) => request_builder.body(b),
            None => request_builder,
        };

        let req = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = self.client.execute(req).await.map_err(|e| {
            format!(
                "Failed to execute request to url: {} with error: {}",
                url, e
            )
        })?;

        let result = match response.status() {
            StatusCode::OK => {
                let result = response
                    .json::<T>()
                    .await
                    //If any error occurs, check result manually and look if any null value exists if it is make it optional on related model
                    .map_err(|e| format!("Failed to parse response body: {}", e))?;

                RestResponse::Success(result)
            }
            _ => {
                let res_text = response.text().await.map_err(|e| {
                    format!(
                        "Failed to parse unsuccess request response body error message text: {}",
                        e
                    )
                })?;
                RestResponse::Error { message: res_text }
            }
        };
        Ok(result)
    }

    fn full_url(&self, url: &str) -> String {
        format!("{}{}", self.base_url, url)
    }

    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<RestResponse<T>, String> {
        let res = self.send_request(Method::GET, url, None).await?;
        Ok(res)
    }
}

pub enum RestResponse<T> {
    Success(T),
    Error { message: String },
}
