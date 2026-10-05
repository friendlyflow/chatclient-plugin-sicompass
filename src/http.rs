//! The HTTP the chat client speaks, in the shape of `reqwest::blocking`.
//!
//! A blocking `ureq` client with rustls and bundled roots. The homeserver is the
//! user's choice, so any server is reachable (`plugin.json` declares
//! `allowedHosts: ["*"]`). The unit tests point it at a mock homeserver.

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

/// The most a response may hold. An initial `/sync` of a busy account is the
/// largest thing the homeserver sends.
const MAX_RESPONSE_BYTES: u64 = 64 * 1024 * 1024;

/// Builds a [`Client`].
pub struct ClientBuilder {
    timeout: Duration,
    user_agent: Option<String>,
}

impl ClientBuilder {
    /// The whole-request timeout, start to end.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn user_agent(mut self, agent: &str) -> Self {
        self.user_agent = Some(agent.to_owned());
        self
    }

    pub fn build(self) -> Result<Client, Error> {
        let mut config = ureq::Agent::config_builder()
            // Every status comes back as it is: the callers read Matrix's
            // error bodies themselves.
            .http_status_as_error(false)
            .timeout_global(Some(self.timeout));
        if let Some(agent) = self.user_agent {
            config = config.user_agent(agent);
        }
        Ok(Client {
            agent: config.build().into(),
        })
    }
}

/// Makes requests.
#[derive(Clone)]
pub struct Client {
    agent: ureq::Agent,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            timeout: Duration::from_secs(30),
            user_agent: None,
        }
    }

    fn request(&self, method: &str, url: &str) -> RequestBuilder {
        RequestBuilder {
            agent: self.agent.clone(),
            method: method.to_owned(),
            url: url.to_owned(),
            headers: Vec::new(),
            body: None,
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
    agent: ureq::Agent,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
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

    pub fn send(self) -> Result<Response, Error> {
        let fail = |e: &dyn std::fmt::Display| Error(format!("{}: {e}", self.url));
        let mut builder = ureq::http::Request::builder()
            .method(self.method.as_str())
            .uri(&self.url);
        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }
        let result = match &self.body {
            Some(body) => self
                .agent
                .run(builder.body(body.as_slice()).map_err(|e| fail(&e))?),
            None => self.agent.run(builder.body(()).map_err(|e| fail(&e))?),
        };
        let mut resp = result.map_err(|e| fail(&e))?;
        let status = StatusCode(resp.status().as_u16());
        let body = resp
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_vec()
            .map_err(|e| fail(&e))?;
        Ok(Response { status, body })
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
}
