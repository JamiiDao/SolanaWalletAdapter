use wallet_adapter::{
    wasm_bindgen_futures::JsFuture,
    web_sys::Window,
    web_sys::{wasm_bindgen::JsCast, Headers, Request, RequestInit, Response},
    WalletError, WalletResult,
};

use crate::ClusterNetState;

// NOTE: You can use Reqwest crate instead to fetch the blockhash but
// this code shows how to use the browser `fetch` api
#[derive(Debug)]
pub struct FetchReq {
    headers: Headers,
    options: RequestInit,
}

impl FetchReq {
    pub fn new(method: &str) -> WalletResult<Self> {
        let options = RequestInit::new();
        options.set_method(method);

        Ok(Self {
            headers: Headers::new()?,
            options,
        })
    }

    pub fn new_for_rpc() -> WalletResult<Self> {
        Self::new("POST")?
            .add_header("content-type", "application/json")?
            .add_header("Accept", "application/json")
    }

    pub async fn ping(active_cluster_endpoint: &str, window: &Window) -> ClusterNetState {
        let body = jzon::object! {
            jsonrpc: "2.0",
            id: 1,
            method: "getVersion",
        }
        .to_string();

        let req = if let Ok(req) = Self::new_for_rpc() {
            req
        } else {
            return ClusterNetState::Failure;
        };

        req.set_body(&body)
            .return_net_state(active_cluster_endpoint, window)
            .await
    }

    pub fn add_header(self, key: &str, value: &str) -> WalletResult<Self> {
        self.headers.append(key, value)?;

        Ok(self)
    }

    pub fn set_body(self, json_body: &str) -> Self {
        self.options.set_body(&json_body.into());

        self
    }

    pub async fn send(self, endpoint: &str, window: &Window) -> WalletResult<String> {
        let resp = self.build(endpoint, window).await?;

        JsFuture::from(resp.text()?)
            .await?
            .as_string()
            .ok_or(WalletError::Op(
                "The response body is not a JsString".to_string(),
            ))
    }

    pub async fn build(&self, endpoint: &str, window: &Window) -> WalletResult<Response> {
        self.options.set_headers(&self.headers);

        let request = Request::new_with_str_and_init(endpoint, &self.options)?;

        let fetch_promise = window.fetch_with_request(&request);

        // Await the fetch promise to get a `Response` object
        let resp_value = JsFuture::from(fetch_promise).await?;
        Ok(resp_value.dyn_into::<Response>()?)
    }

    pub async fn return_net_state(&self, endpoint: &str, window: &Window) -> ClusterNetState {
        self.options.set_headers(&self.headers);

        let request = if let Ok(inner) = Request::new_with_str_and_init(endpoint, &self.options) {
            inner
        } else {
            return ClusterNetState::Failure;
        };

        let fetch_promise = window.fetch_with_request(&request);

        // Await the fetch promise to get a `Response` object
        let resp_value = if let Ok(inner) = JsFuture::from(fetch_promise).await {
            inner
        } else {
            return ClusterNetState::Failure;
        };

        if resp_value.dyn_into::<Response>().is_err() {
            ClusterNetState::Failure
        } else {
            ClusterNetState::Success
        }
    }
}
