//! ## BriteVerify API Client
///
// Standard Library Imports
use std::{env, ops::Deref, time::Duration};

// Third-Party Imports
use anyhow::{Context, Result};
use reqwest;
use reqwest::{
    header::{HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION},
    StatusCode,
};
use tracing_subscriber::{
    fmt::layer as tracing_layer, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

// Crate Imports
use crate::{errors, types};

// <editor-fold desc="// Constants ...">

static V1_API_BASE_URL: &'static str = "https://bpi.briteverify.com/api/v1";
static V3_API_BASE_URL: &'static str = "https://bulk-api.briteverify.com/api/v3";
static DEFAULT_LOG_FILTER: &'static str = "briteverify_rs=debug,reqwest=info";

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// ClientBuilder ...">

/// Helper for incrementally building a [`BriteVerifyClient`](BriteVerifyClient)
/// instance with custom configuration.
///
/// ## Usage
/// ```no_run
/// # use std::time::Duration;
/// # use briteverify_rs::{BriteVerifyClient, BriteVerifyClientBuilder};
/// #
/// # #[tokio::main]
/// # async fn doc() -> anyhow::Result<()> {
/// let builder: BriteVerifyClientBuilder = BriteVerifyClient::builder();
/// let client: BriteVerifyClient = builder
///     .cookie_store(true)                         // reqwest::ClientBuilder::cookie_store
///     .api_key("YOUR API KEY")                    // BriteVerifyClientBuilder::api_key
///     .timeout(Duration::from_secs(360))          // reqwest::ClientBuilder::timeout
///     .connect_timeout(Duration::from_secs(360))  // reqwest::ClientBuilder::connect_timeout
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct BriteVerifyClientBuilder {
    error: Option<InvalidHeaderValue>,
    api_key: Option<HeaderValue>,
    builder: reqwest::ClientBuilder,
}

impl From<reqwest::ClientBuilder> for BriteVerifyClientBuilder {
    fn from(builder: reqwest::ClientBuilder) -> Self {
        let build_repr = format!("{:?}", builder);

        let mut instance = Self {
            builder,
            ..Self::default()
        };

        if build_repr.contains("\"authorization\": Sensitive")
            || build_repr.contains("\"authorization\": \"ApiKey:")
        {
            instance.api_key = Some(HeaderValue::from_static("IGNORE ME"));
        }

        instance
    }
}

impl BriteVerifyClientBuilder {
    /// Create a new [`BriteVerifyClientBuilder`][BriteVerifyClientBuilder] instance
    pub fn new() -> Self {
        Self {
            error: None,
            api_key: None,
            builder: reqwest::ClientBuilder::new(),
        }
    }

    /// Build a `BriteVerifyClient` that uses the customized configuration.
    pub fn build(mut self) -> Result<BriteVerifyClient, errors::BriteVerifyClientError> {
        if let Some(error) = self.error {
            return Err(error.into());
        }

        match self.api_key {
            None => Err(errors::BriteVerifyClientError::MissingApiKey),
            Some(key) => {
                let logging_conf = env::var("LOG_LEVELS").unwrap_or(DEFAULT_LOG_FILTER.to_string());

                tracing_subscriber::registry()
                    .with(EnvFilter::new(logging_conf))
                    .with(tracing_layer())
                    .init();

                if key.is_sensitive() {
                    let headers = HeaderMap::from_iter([(AUTHORIZATION, key)].into_iter());
                    self.builder = self.builder.default_headers(headers);
                }

                Ok(BriteVerifyClient(
                    self.builder
                        .build()
                        .context("Could not create a usable `reqwest` client")?,
                ))
            }
        }
    }

    /// Set the API key to use for requests to the BriteVerify API
    /// [[ref](https://docs.briteverify.com/#intro:~:text=API%20Suite%20Documentation-,Authorization,-To%20get%20started)]
    pub fn api_key<ApiKey: ToString>(mut self, api_key: ApiKey) -> Self {
        let api_key: String = format!("ApiKey: {}", api_key.to_string());

        match HeaderValue::from_str(&api_key) {
            Ok(mut header) => {
                header.set_sensitive(true);
                self.api_key = Some(header);
            }
            Err(error) => {
                self.error = Some(error.into());
            }
        }

        self
    }

    // Timeout options

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished.
    ///
    /// Default is no timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.timeout(timeout);
        self
    }

    /// Set a timeout for only the connect phase of a `Client`.
    ///
    /// Default is `None`.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.connect_timeout(timeout);
        self
    }

    /// Sets the `User-Agent` header to be used by the constructed client.
    pub fn user_agent<V>(mut self, value: V) -> BriteVerifyClientBuilder
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<http::Error>,
    {
        self.builder = self.builder.user_agent(value);
        self
    }

    /// Sets the default headers for every request.
    pub fn default_headers(mut self, headers: HeaderMap) -> BriteVerifyClientBuilder {
        self.builder = self.builder.default_headers(headers);
        self
    }

    /// Enable auto gzip decompression by checking the `Content-Encoding` response header.
    ///
    /// If auto gzip decompression is turned on:
    ///
    /// - When sending a request and if the request's headers do not already contain
    ///   an `Accept-Encoding` **and** `Range` values, the `Accept-Encoding` header is set to `gzip`.
    ///   The request body is **not** automatically compressed.
    /// - When receiving a response, if its headers contain a `Content-Encoding` value of
    ///   `gzip`, both `Content-Encoding` and `Content-Length` are removed from the
    ///   headers' set. The response body is automatically decompressed.
    ///
    /// By default this option is enabled.
    pub fn gzip(mut self, enable: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.gzip(enable);
        self
    }

    /// Enable auto brotli decompression by checking the `Content-Encoding` response header.
    ///
    /// If auto brotli decompression is turned on:
    ///
    /// - When sending a request and if the request's headers do not already contain
    ///   an `Accept-Encoding` **and** `Range` values, the `Accept-Encoding` header is set to `br`.
    ///   The request body is **not** automatically compressed.
    /// - When receiving a response, if its headers contain a `Content-Encoding` value of
    ///   `br`, both `Content-Encoding` and `Content-Length` are removed from the
    ///   headers' set. The response body is automatically decompressed.
    ///
    /// By default this option is enabled.
    pub fn brotli(mut self, enable: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.brotli(enable);
        self
    }

    /// Disable auto response body gzip decompression.
    ///
    /// This method exists even if the optional `gzip` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use gzip decompression
    /// even if another dependency were to enable the optional `gzip` feature.
    pub fn no_gzip(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_gzip();
        self
    }

    /// Disable auto response body brotli decompression.
    ///
    /// This method exists even if the optional `brotli` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use brotli decompression
    /// even if another dependency were to enable the optional `brotli` feature.
    pub fn no_brotli(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_brotli();
        self
    }

    /// Disable auto response body deflate decompression.
    ///
    /// This method exists even if the optional `deflate` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use deflate decompression
    /// even if another dependency were to enable the optional `deflate` feature.
    pub fn no_deflate(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_deflate();
        self
    }

    // Redirect options

    /// Set a `RedirectPolicy` for this client.
    ///
    /// Default will follow redirects up to a maximum of 10.
    pub fn redirect(mut self, policy: reqwest::redirect::Policy) -> BriteVerifyClientBuilder {
        self.builder = self.builder.redirect(policy);
        self
    }

    /// Enable or disable automatic setting of the `Referer` header.
    ///
    /// Default is `true`.
    pub fn referer(mut self, enable: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.referer(enable);
        self
    }

    // Proxy options

    /// Add a `Proxy` to the list of proxies the `Client` will use.
    ///
    /// # Note
    ///
    /// Adding a proxy will disable the automatic usage of the "system" proxy.
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> BriteVerifyClientBuilder {
        self.builder = self.builder.proxy(proxy);
        self
    }

    /// Clear all `Proxies`, so `Client` will use no proxy anymore.
    ///
    /// # Note
    /// To add a proxy exclusion list, use [reqwest::proxy::Proxy::no_proxy()]
    /// on all desired proxies instead.
    ///
    /// This also disables the automatic usage of the "system" proxy.
    pub fn no_proxy(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_proxy();
        self
    }

    /// Set whether connections should emit verbose logs.
    ///
    /// Enabling this option will emit [log][] messages at the `TRACE` level
    /// for read and write operations on connections.
    ///
    /// [log]: https://crates.io/crates/log
    pub fn connection_verbose(mut self, verbose: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.connection_verbose(verbose);
        self
    }

    // HTTP options

    /// Set an optional timeout for idle sockets being kept-alive.
    ///
    /// Pass `None` to disable timeout.
    ///
    /// Default is 90 seconds.
    pub fn pool_idle_timeout<D: Into<Option<Duration>>>(
        mut self,
        value: D,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.pool_idle_timeout(value);
        self
    }

    /// Sets the maximum idle connection per host allowed in the pool.
    pub fn pool_max_idle_per_host(mut self, value: usize) -> BriteVerifyClientBuilder {
        self.builder = self.builder.pool_max_idle_per_host(value);
        self
    }

    /// Send headers as title case instead of lowercase.
    pub fn http1_title_case_headers(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http1_title_case_headers();
        self
    }

    /// Set whether HTTP/1 connections will accept obsolete line folding for
    /// header values.
    ///
    /// Newline codepoints (`\r` and `\n`) will be transformed to spaces when
    /// parsing.
    pub fn http1_allow_obsolete_multiline_headers_in_responses(
        mut self,
        value: bool,
    ) -> BriteVerifyClientBuilder {
        self.builder = self
            .builder
            .http1_allow_obsolete_multiline_headers_in_responses(value);
        self
    }

    /// Only use HTTP/1.
    pub fn http1_only(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http1_only();
        self
    }

    /// Allow HTTP/0.9 responses
    pub fn http09_responses(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http09_responses();
        self
    }

    /// Only use HTTP/2.
    pub fn http2_prior_knowledge(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_prior_knowledge();
        self
    }

    /// Sets the `SETTINGS_INITIAL_WINDOW_SIZE` option for HTTP2 stream-level flow control.
    ///
    /// Default is currently 65,535 but may change internally to optimize for common uses.
    pub fn http2_initial_stream_window_size<WindowSize: Into<Option<u32>>>(
        mut self,
        value: WindowSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_initial_stream_window_size(value);
        self
    }

    /// Sets the max connection-level flow control for HTTP2
    ///
    /// Default is currently 65,535 but may change internally to optimize for common uses.
    pub fn http2_initial_connection_window_size<WindowSize: Into<Option<u32>>>(
        mut self,
        value: WindowSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_initial_connection_window_size(value);
        self
    }

    /// Sets whether to use an adaptive flow control.
    ///
    /// Enabling this will override the limits set in `http2_initial_stream_window_size` and
    /// `http2_initial_connection_window_size`.
    pub fn http2_adaptive_window(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_adaptive_window(enabled);
        self
    }

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Default is currently 16,384 but may change internally to optimize for common uses.
    pub fn http2_max_frame_size<FrameSize: Into<Option<u32>>>(
        mut self,
        value: FrameSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_max_frame_size(value);
        self
    }

    /// Sets an interval for HTTP2 Ping frames should be sent to keep a connection alive.
    ///
    /// Pass `None` to disable HTTP2 keep-alive.
    /// Default is currently disabled.
    pub fn http2_keep_alive_interval<Interval: Into<Option<Duration>>>(
        mut self,
        interval: Interval,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_interval(interval);
        self
    }

    /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will be closed.
    /// Does nothing if `http2_keep_alive_interval` is disabled.
    /// Default is currently disabled.
    pub fn http2_keep_alive_timeout(mut self, timeout: Duration) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_timeout(timeout);
        self
    }

    /// Sets whether HTTP2 keep-alive should apply while the connection is idle.
    ///
    /// If disabled, keep-alive pings are only sent while there are open request/responses streams.
    /// If enabled, pings are also sent when no streams are active.
    /// Does nothing if `http2_keep_alive_interval` is disabled.
    /// Default is `false`.
    pub fn http2_keep_alive_while_idle(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_while_idle(enabled);
        self
    }

    // TCP options

    /// Set whether sockets have `TCP_NODELAY` enabled.
    ///
    /// Default is `true`.
    pub fn tcp_nodelay(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tcp_nodelay(enabled);
        self
    }

    /// Bind to a local IP Address.
    ///
    /// # Example
    ///
    /// ```
    /// use std::net::IpAddr;
    /// let local_addr = IpAddr::from([12, 4, 1, 8]);
    /// let client = briteverify_rs::BriteVerifyClient::builder()
    ///     .api_key("YOUR API KEY")
    ///     .local_address(local_addr)
    ///     .build()?;
    /// ```
    pub fn local_address<T: Into<Option<std::net::IpAddr>>>(
        mut self,
        address: T,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.local_address(address);
        self
    }

    /// Set that all sockets have `SO_KEEPALIVE` set with the supplied duration.
    ///
    /// If `None`, the option will not be set.
    pub fn tcp_keepalive<D: Into<Option<Duration>>>(
        mut self,
        value: D,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tcp_keepalive(value);
        self
    }

    // TLS options

    /// Add a custom root certificate.
    ///
    /// This can be used to connect to a server that has a self-signed
    /// certificate for example.
    pub fn add_root_certificate(mut self, cert: reqwest::Certificate) -> BriteVerifyClientBuilder {
        self.builder = self.builder.add_root_certificate(cert);
        self
    }

    /// Controls the use of built-in/preloaded certificates during certificate validation.
    ///
    /// Defaults to `true` -- built-in system certs will be used.
    pub fn tls_built_in_root_certs(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tls_built_in_root_certs(enabled);
        self
    }

    /// Sets the identity to be used for client certificate authentication.
    pub fn identity(mut self, value: reqwest::Identity) -> BriteVerifyClientBuilder {
        self.builder = self.builder.identity(value);
        self
    }

    /// Controls the use of certificate validation.
    ///
    /// Defaults to `false`.
    ///
    /// # Warning
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort.
    pub fn danger_accept_invalid_certs(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.danger_accept_invalid_certs(enabled);
        self
    }

    /// Controls the use of TLS server name indication.
    ///
    /// Defaults to `true`.
    pub fn tls_sni(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tls_sni(enabled);
        self
    }

    /// Set the minimum required TLS version for connections.
    ///
    /// By default the TLS backend's own default is used.
    ///
    /// # Errors
    ///
    /// A value of `tls::Version::TLS_1_3` will cause an error with the
    /// `native-tls`/`default-tls` backend. This does not mean the version
    /// isn't supported, just that it can't be set as a minimum due to
    /// technical limitations.
    pub fn min_tls_version(mut self, version: reqwest::tls::Version) -> BriteVerifyClientBuilder {
        self.builder = self.builder.min_tls_version(version);
        self
    }

    /// Set the maximum allowed TLS version for connections.
    ///
    /// By default there's no maximum.
    ///
    /// # Errors
    ///
    /// A value of `tls::Version::TLS_1_3` will cause an error with the
    /// `native-tls`/`default-tls` backend. This does not mean the version
    /// isn't supported, just that it can't be set as a maximum due to
    /// technical limitations.
    pub fn max_tls_version(mut self, version: reqwest::tls::Version) -> BriteVerifyClientBuilder {
        self.builder = self.builder.max_tls_version(version);
        self
    }

    /// Disables the trust-dns async resolver.
    ///
    /// This method exists even if the optional `trust-dns` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use the trust-dns async resolver
    /// even if another dependency were to enable the optional `trust-dns` feature.
    pub fn no_trust_dns(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_trust_dns();
        self
    }

    /// Restrict the Client to be used with HTTPS only requests.
    ///
    /// Defaults to false.
    pub fn https_only(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.https_only(enabled);
        self
    }

    /// Override DNS resolution for specific domains to a particular IP address.
    ///
    /// Warning
    ///
    /// Since the DNS protocol has no notion of ports, if you wish to send
    /// traffic to a particular port you must include this port in the URL
    /// itself, any port in the overridden addr will be ignored and traffic sent
    /// to the conventional port for the given scheme (e.g. 80 for http).
    pub fn resolve(
        mut self,
        domain: &str,
        address: std::net::SocketAddr,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.resolve(domain, address);
        self
    }

    /// Override DNS resolution for specific domains to particular IP addresses.
    ///
    /// Warning
    ///
    /// Since the DNS protocol has no notion of ports, if you wish to send
    /// traffic to a particular port you must include this port in the URL
    /// itself, any port in the overridden addresses will be ignored and traffic sent
    /// to the conventional port for the given scheme (e.g. 80 for http).
    pub fn resolve_to_addrs(
        mut self,
        domain: &str,
        addresses: &[std::net::SocketAddr],
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.resolve_to_addrs(domain, addresses);
        self
    }

    /// Override the DNS resolver implementation.
    ///
    /// Pass an `Arc` wrapping a trait object implementing `Resolve`.
    /// Overrides for specific names passed to `resolve` and `resolve_to_addrs` will
    /// still be applied on top of this resolver.
    pub fn dns_resolver<R: reqwest::dns::Resolve + 'static>(
        mut self,
        resolver: std::sync::Arc<R>,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.dns_resolver(resolver);
        self
    }
}

// </editor-fold desc="// ClientBuilder ...">

// <editor-fold desc="// Client ...">

/// `briteverify-rs`'s [`reqwest`](https://docs.rs/reqwest/latest/reqwest/)-based client
#[derive(Debug)]
pub struct BriteVerifyClient(reqwest::Client);

impl Deref for BriteVerifyClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<reqwest::Client> for BriteVerifyClient {
    type Error = errors::BriteVerifyClientError;

    fn try_from(client: reqwest::Client) -> Result<Self, Self::Error> {
        let client_repr = format!("{:?}", &client);

        if client_repr.contains("\"authorization\": Sensitive")
            || client_repr.contains("\"authorization\": \"ApiKey:")
        {
            Ok(Self(client))
        } else {
            Err(errors::BriteVerifyClientError::MissingApiKey)
        }
    }
}

impl BriteVerifyClient {
    /// Create a new [`BriteVerifyClient`][BriteVerifyClient] instance
    pub fn new<ApiKey: ToString>(api_key: ApiKey) -> Result<Self, errors::BriteVerifyClientError> {
        Self::builder()
            .api_key(api_key)
            .timeout(Duration::from_secs(360))
            .connect_timeout(Duration::from_secs(360))
            .build()
    }

    /// Create a new [builder][BriteVerifyClientBuilder] to incrementally
    /// build a [`BriteVerifyClient`][BriteVerifyClient] with a customised
    /// configuration
    pub fn builder() -> BriteVerifyClientBuilder {
        BriteVerifyClientBuilder::new()
    }

    // <editor-fold desc="// Real-Time Single Transaction Endpoints ... ">

    /// Get your current account credit balance
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn current_credits(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits)
    }

    /// Get the total number of credits your account currently has in reserve
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn current_credits_in_reserve(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits_in_reserve)
    }

    /// Get your account credit balance, total number of credits
    /// in reserve, and the timestamp of when your balance was
    /// most recently recorded
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn get_account_balance(&self) -> Result<types::AccountCreditBalance> {
        let response = self
            .get(format!("{V3_API_BASE_URL}/accounts/credits"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::AccountCreditBalance>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// [internal-implementation]
    /// Actually perform a single-transaction verification
    async fn _full_verify<Displayable: ToString>(
        &self,
        email: Option<Displayable>,
        phone: Option<Displayable>,
        address1: Option<Displayable>,
        address2: Option<Displayable>,
        city: Option<Displayable>,
        state: Option<Displayable>,
        zip: Option<Displayable>,
    ) -> Result<types::VerificationResponse> {
        let request = types::VerificationRequest::from_values(
            email, phone, address1, address2, city, state, zip,
        )?;

        let response = self
            .0
            .post(format!("{V1_API_BASE_URL}/fullverify"))
            .json(&request)
            .send()
            .await?;

        match &response.status() {
            &StatusCode::OK => Ok(response.json::<types::VerificationResponse>().await?),
            _ => {
                println!("{:#?}", &response);
                let data = response.text().await?;
                println!("Content: {data:#?}");
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Verify a "complete" contact record
    /// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
    pub async fn verify_contact<Displayable: ToString>(
        &self,
        email: Displayable,
        phone: Displayable,
        address1: Displayable,
        address2: Option<Displayable>,
        city: Displayable,
        state: Displayable,
        zip: Displayable,
    ) -> Result<types::FullVerificationResponse> {
        let response = self
            ._full_verify(
                Some(email),
                Some(phone),
                Some(address1),
                address2,
                Some(city),
                Some(state),
                Some(zip),
            )
            .await?;

        match response {
            types::VerificationResponse::Full(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single email address
    /// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
    pub async fn verify_email<EmailAddress: ToString>(
        &self,
        email: EmailAddress,
    ) -> Result<types::EmailVerificationResponse> {
        let response = self
            ._full_verify(Some(email), None, None, None, None, None, None)
            .await?;

        match response {
            types::VerificationResponse::Email(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single phone number
    /// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
    pub async fn verify_phone_number<PhoneNumber: ToString>(
        &self,
        phone: PhoneNumber,
    ) -> Result<types::PhoneNumberVerificationResponse> {
        let response = self
            ._full_verify(None, Some(phone), None, None, None, None, None)
            .await?;

        match response {
            types::VerificationResponse::Phone(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single street address
    /// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
    pub async fn verify_street_address<Displayable: ToString>(
        &self,
        address1: Displayable,
        address2: Option<Displayable>,
        city: Displayable,
        state: Displayable,
        zip: Displayable,
    ) -> Result<types::AddressVerificationResponse> {
        let response = self
            ._full_verify(
                None,
                None,
                Some(address1),
                address2,
                Some(city),
                Some(state),
                Some(zip),
            )
            .await?;

        match response {
            types::VerificationResponse::Address(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    // </editor-fold desc="// Real-Time Single Transaction Endpoints ... ">

    // <editor-fold desc="// Bulk Verification (v3) Endpoints ... ">

    /// Retrieve the complete, unfiltered list of all bulk verification
    /// lists created within the last 7 calendar days
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists(&self) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            <Option<chrono::NaiveDate>>::None,
            <Option<String>>::None,
        )
        .await
    }

    /// Retrieve the complete list of all bulk verification lists created
    /// within the last 7 calendar days filtered by the specified criteria
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_filtered_lists<'header, Date: chrono::Datelike, State: ToString>(
        &self,
        page: Option<u32>,
        date: Option<Date>,
        state: Option<State>,
    ) -> Result<types::GetListStatesResponse> {
        let mut params: Vec<(&'header str, String)> = Vec::new();

        if let Some(page) = page {
            params.push(("page", page.to_string()));
        }

        if let Some(date) = date {
            params.push((
                "date",
                format!("{}-{:0>2}-{:0>2}", date.year(), date.month(), date.day()),
            ));
        }

        if let Some(state) = state {
            let value = state.to_string();
            let filter = types::BatchState::from(value.as_str());

            if filter == types::BatchState::Unknown {
                tracing::warn!(
                    "Declining to include unknown list state as request filter: {value:#?}"
                );
            } else {
                params.push(("state", filter.to_string()));
            }
        }

        let mut request = self.get(format!("{V3_API_BASE_URL}/lists"));

        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::GetListStatesResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Retrieve the complete list of all bulk verification lists filtered
    /// by the specified date [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    ///
    /// > **NOTE:** Regardless of specified date, the BriteVerify API
    /// > does not appear to persist bulk verification lists older than
    /// > 7 calendar days
    pub async fn get_lists_by_date<Date: chrono::Datelike>(
        &self,
        date: Date,
    ) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            Some(date),
            <Option<types::BatchState>>::None,
        )
        .await
    }

    /// Retrieve the specified "page" of bulk verification lists
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists_by_page(&self, page: u32) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            Some(page),
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
        )
        .await
    }

    /// Retrieve the complete list of all bulk verification lists whose status
    /// matches the specified one created within the last 7 calendar days
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists_by_state(
        &self,
        state: types::BatchState,
    ) -> Result<types::GetListStatesResponse> {
        match state {
            types::BatchState::Unknown => {
                tracing::warn!("Declining to request lists using 'unknown' as list state filter");
                Ok(types::GetListStatesResponse::default())
            }
            _ => {
                self.get_filtered_lists(
                    <Option<u32>>::None,
                    <Option<chrono::NaiveDate>>::None,
                    Some(state),
                )
                .await
            }
        }
    }

    /// Create a new bulk verification list with the supplied records
    /// and (optionally) queue it for immediate processing
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1)]
    pub async fn create_list<
        Contact: Into<types::VerificationRequest>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        contacts: ContactCollection,
        auto_start: bool,
    ) -> Result<types::CreateListResponse> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)

        self._create_or_update_list(<Option<String>>::None, contacts, auto_start)
            .await
    }

    /// Append records to the specified bulk verification list and (optionally)
    /// queue it for immediate processing
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1:~:text=customer%2DID/lists-,list_id,-(optional))]
    pub async fn update_list<
        ListId: ToString,
        Contact: Into<types::VerificationRequest>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        list_id: ListId,
        contacts: ContactCollection,
        auto_start: bool,
    ) -> Result<types::UpdateListResponse> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)
        self._create_or_update_list(Some(list_id), contacts, auto_start)
            .await
    }

    /// [internal-implementation]
    /// Create a new or mutate an extant bulk verification list
    async fn _create_or_update_list<
        ListId: ToString,
        Contact: Into<types::VerificationRequest>,
        Directive: Into<types::BulkListDirective>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        list_id: Option<ListId>,
        contacts: ContactCollection,
        directive: Directive,
    ) -> Result<types::CreateListResponse> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)

        let directive = directive.into();

        let request = types::BulkVerificationRequest::new(contacts, directive);

        let url: String = {
            if let Some(id) = list_id {
                let list_id = id.to_string();
                format!("{V3_API_BASE_URL}/lists/{list_id}")
            } else {
                format!("{V3_API_BASE_URL}/lists")
            }
        };

        let response = self.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::BAD_REQUEST => {
                Ok(response.json::<types::CreateListResponse>().await?)
            }
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Retrieve current "state" of the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#b09c09dc-e11e-44a8-b53d-9f1fd9c6792d)]
    pub async fn get_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::VerificationListState> {
        let list_id: String = list_id.to_string();

        let response = self
            .get(format!("{V3_API_BASE_URL}/lists/{list_id}"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::VerificationListState>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Delete the specified batch verification list
    /// [[ref](https://docs.briteverify.com/#6c9b9c05-a4a0-435e-a064-af7d9476719d)]
    ///
    /// ___
    /// **NOTE:** This action *cannot* be reversed and
    /// once completed, the list will *no longer exist*.
    /// The list must be in one of the following states
    /// to be deleted:
    /// - [Prepped](types::enums::BatchState::Prepped)
    /// - [Complete](types::enums::BatchState::Complete)
    /// - [Delivered](types::enums::BatchState::Delivered)
    /// - [ImportError](types::enums::BatchState::ImportError)
    /// ___
    pub async fn delete_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::DeleteListResponse> {
        let list_id: String = list_id.to_string();

        let response = self
            .delete(format!("{V3_API_BASE_URL}/lists/{list_id}"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::DeleteListResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Abandon the specified unprocessed bulk verification list
    /// [[ref](https://docs.briteverify.com/#6c9b9c05-a4a0-435e-a064-af7d9476719d:~:text=To-,abandon,-an%20open%20list)]
    pub async fn terminate_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse> {
        self._create_or_update_list(
            Some(list_id),
            <Vec<types::VerificationRequest>>::new(),
            types::BulkListDirective::Terminate,
        )
        .await
    }

    /// [internal-implementation]
    /// Retrieve the specified page of results from the specified
    /// bulk verification list
    async fn _get_result_page(
        &self,
        list_id: String,
        page_number: u64,
    ) -> Result<types::BulkVerificationResponse> {
        let response = self
            .get(format!(
                "{V3_API_BASE_URL}/lists/{list_id}/export/{page_number}"
            ))
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<types::BulkVerificationResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Get the verification results for the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831)]
    ///
    /// ___
    /// **NOTE:** Verification results are only available once
    /// a list has finished verifying in its entirety. It is not
    /// possible to retrieve verification results piecemeal.
    /// ___
    pub async fn get_results_by_list_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<Vec<types::BulkVerificationResult>> {
        let list_id = list_id.to_string();
        let list_status = self.get_list_by_id(&list_id).await?;

        if list_status.page_count.is_none() {
            anyhow::bail!("Missing page count!");
        }

        let page_count = std::cmp::max(1u64, list_status.page_count.unwrap());

        let pages: Vec<_> = futures_util::future::join_all(
            (1..=page_count).map(|page_number| self._get_result_page(list_id.clone(), page_number)),
        )
        .await
        .into_iter()
        .filter(Result::is_ok) // TODO: Change this filter to log failed page fetches
        .map(|task_result| task_result.unwrap().results)
        .collect();

        let results: Vec<types::BulkVerificationResult> = itertools::concat(pages);

        Ok(results)
    }

    /// Queue the specified (open) bulk verification list for immediate processing
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831:~:text=immediately%20start%20a%20list)]
    pub async fn queue_list_for_processing<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse> {
        self._create_or_update_list(
            Some(list_id),
            <Vec<types::VerificationRequest>>::new(),
            types::BulkListDirective::Start,
        )
        .await
    }

    // </editor-fold desc="// Bulk Verification (v3) Endpoints ... ">
}

// </editor-fold desc="// Client ...">
