//! The HTTP the chat client speaks, in the shape of `reqwest::blocking`.
//!
//! Inside the sandbox every request goes through the host's `net.fetch`, which
//! checks it against `allowedHosts` (any public server, for a homeserver the
//! user chooses) and never reaches the local network. Natively, for the unit
//! tests against a mock homeserver, it is reqwest.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// A failed request, or a body that would not parse.
#[derive(Debug, Clone)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// An HTTP status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusCode(pub u16);

impl StatusCode {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.0)
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Builds a [`Client`].
pub struct ClientBuilder {
    timeout: Duration,
}

impl ClientBuilder {
    /// The whole-request timeout (natively; inside the sandbox the host sets
    /// it: longer in a task, for the long poll).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Kept for the call sites; the host sets its own user agent.
    pub fn user_agent(self, _agent: &str) -> Self {
        self
    }

    pub fn build(self) -> Result<Client, Error> {
        Ok(Client {
            timeout: self.timeout,
        })
    }
}

/// Makes requests.
#[derive(Clone)]
pub struct Client {
    timeout: Duration,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            timeout: Duration::from_secs(30),
        }
    }

    fn request(&self, method: &str, url: &str) -> RequestBuilder {
        RequestBuilder {
            method: method.to_owned(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: None,
            timeout: self.timeout,
        }
    }

    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request("GET", url.as_ref())
    }

    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request("POST", url.as_ref())
    }

    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request("PUT", url.as_ref())
    }
}

/// One request, being built.
pub struct RequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    timeout: Duration,
}

impl RequestBuilder {
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// A JSON body, with its content type.
    pub fn json<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        self.body = serde_json::to_vec(value).ok();
        self.headers
            .push(("Content-Type".to_owned(), "application/json".to_owned()));
        self
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn send(self) -> Result<Response, Error> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|e| Error(e.to_string()))?;
        let method = reqwest::Method::from_bytes(self.method.as_bytes())
            .map_err(|e| Error(e.to_string()))?;
        let mut req = client.request(method, &self.url);
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }
        if let Some(body) = self.body {
            req = req.body(body);
        }
        let resp = req.send().map_err(|e| Error(e.to_string()))?;
        let status = StatusCode(resp.status().as_u16());
        let body = resp.bytes().map_err(|e| Error(e.to_string()))?.to_vec();
        Ok(Response { status, body })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn send(self) -> Result<Response, Error> {
        let _ = self.timeout;
        let resp = sicompass_pdk::net::fetch(&sicompass_pdk::net::HttpRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: self.body,
        })
        .map_err(Error)?;
        Ok(Response {
            status: StatusCode(resp.status),
            body: resp.body,
        })
    }
}

/// A response, read whole.
pub struct Response {
    status: StatusCode,
    body: Vec<u8>,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        serde_json::from_slice(&self.body).map_err(|e| Error(e.to_string()))
    }

    /// The body as text (the sync task passes it on whole).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn text(self) -> Result<String, Error> {
        Ok(String::from_utf8_lossy(&self.body).into_owned())
    }
}
