//! ## BriteVerify API Client
///
// Standard Library Imports
use std::{env, ops::Deref, time::Duration};

// Third-Party Imports
use anyhow::{Context, Result};
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
/// instance with a custom configuration.
///
/// ## Basic Usage
/// ```no_run
/// # use std::time::Duration;
/// # use briteverify_rs::{BriteVerifyClient, BriteVerifyClientBuilder};
/// #
/// # fn doc() -> anyhow::Result<()> {
/// let builder: BriteVerifyClientBuilder = BriteVerifyClient::builder();
///
/// let client: BriteVerifyClient = builder
///     .api_key("YOUR API KEY")
///     .timeout(Duration::from_secs(360))
///     .connect_timeout(Duration::from_secs(360))
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new();
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Self {
        Self {
            error: None,
            api_key: None,
            builder: reqwest::ClientBuilder::new(),
        }
    }

    /// Build a custom configured [`BriteVerifyClient`](BriteVerifyClient) instance.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let client: BriteVerifyClient = BriteVerifyClient::builder()
    ///     .api_key("YOUR API KEY")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .api_key("YOUR API KEY");
    /// # Ok(())
    /// # }
    /// ```
    pub fn api_key<ApiKey: ToString>(mut self, api_key: ApiKey) -> Self {
        let api_key: String = format!("ApiKey: {}", api_key.to_string());

        match HeaderValue::from_str(&api_key) {
            Ok(mut header) => {
                header.set_sensitive(true);
                self.api_key = Some(header);
            }
            Err(error) => {
                self.api_key = None;
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
    ///
    /// #### Example
    /// ```no_run
    /// # use std::time::Duration;
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .timeout(Duration::from_secs(5));
    /// # Ok(())
    /// # }
    /// ```
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.timeout(timeout);
        self
    }

    /// Set a timeout for only the connect phase of a `Client`.
    ///
    /// Default is `None`.
    ///
    /// #### Example
    /// ```no_run
    /// # use std::time::Duration;
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .connect_timeout(Duration::from_secs(5));
    /// # Ok(())
    /// # }
    /// ```
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.builder = self.builder.connect_timeout(timeout);
        self
    }

    /// Sets the `User-Agent` header to be used by the constructed client.
    ///
    /// Unless explicitly set, the `User-Agent` header will be omitted entirely
    /// from all requests.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .user_agent("briteverify-rs");
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_agent<V>(mut self, value: V) -> BriteVerifyClientBuilder
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<http::Error>,
    {
        self.builder = self.builder.user_agent(value);
        self
    }

    /// Sets the default headers for every request.
    ///
    /// **NOTE:** [`HeaderMap`](HeaderMap)s do not enforce
    /// uniqueness of contained key-value pairs. It is *absolutely*
    /// possible to insert the same key more than once, either
    /// with the same value or wildly different values. Proceed
    /// accordingly.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// # fn doc() -> anyhow::Result<()> {
    /// use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
    ///
    /// let mut headers = HeaderMap::new();
    /// let content_type = HeaderValue::from_static("application/json");
    ///
    /// headers.insert(CONTENT_TYPE, content_type);
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .default_headers(headers);
    /// # Ok(())
    /// # }
    /// ```
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
    /// Because `briteverify-rs` explicitly enables `reqwest`'s *gzip* feature, this option is
    /// enabled by default.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .gzip(true);
    /// # Ok(())
    /// # }
    /// ```
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
    /// Because `briteverify-rs` explicitly enables `reqwest`'s *brotli* feature, this option is
    /// enabled by default.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .brotli(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn brotli(mut self, enable: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.brotli(enable);
        self
    }

    /// Disable auto response body gzip decompression.
    ///
    /// This method exists even if the optional `gzip` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use gzip decompression
    /// even if another dependency were to enable the optional `gzip` feature.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .no_gzip();
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_gzip(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_gzip();
        self
    }

    /// Disable auto response body brotli decompression.
    ///
    /// This method exists even if the optional `brotli` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use brotli decompression
    /// even if another dependency were to enable the optional `brotli` feature.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .no_brotli();
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_brotli(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_brotli();
        self
    }

    /// Disable auto response body deflate decompression.
    ///
    /// This method exists even if the optional `deflate` feature is not enabled.
    /// This can be used to ensure a `Client` doesn't use deflate decompression
    /// even if another dependency were to enable the optional `deflate` feature.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .no_deflate();
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_deflate(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_deflate();
        self
    }

    // Redirect options

    /// Set a [`RedirectPolicy`](reqwest::redirect::Policy) for this client.
    ///
    /// Default will follow redirects up to a maximum of 10.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// use reqwest::redirect::Policy as RedirectPolicy;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .redirect(RedirectPolicy::none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn redirect(mut self, policy: reqwest::redirect::Policy) -> BriteVerifyClientBuilder {
        self.builder = self.builder.redirect(policy);
        self
    }

    /// Enable or disable automatic setting of the `Referer` header.
    ///
    /// Default is `true`.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .referer(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn referer(mut self, enable: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.referer(enable);
        self
    }

    // Proxy options

    /// Add a [`Proxy`](reqwest::Proxy) to the list of proxies the
    /// constructed [`BriteVerifyClient`](BriteVerifyClient) will use.
    ///
    /// # Note
    ///
    /// Adding a proxy will disable the automatic usage of the "system" proxy.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .proxy(reqwest::Proxy::http("https://my.prox")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> BriteVerifyClientBuilder {
        self.builder = self.builder.proxy(proxy);
        self
    }

    /// Clear all [`Proxies`](reqwest::Proxy), so the constructed
    /// [`BriteVerifyClient`](BriteVerifyClient) will not use any proxies.
    ///
    /// # Note
    /// To add a proxy exclusion list, use [`reqwest::Proxy::no_proxy()`](reqwest::Proxy::no_proxy)
    /// on all desired proxies instead.
    ///
    /// This also disables the automatic usage of the "system" proxy.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .no_proxy();
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_proxy(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_proxy();
        self
    }

    /// Set whether connections should emit verbose logs.
    ///
    /// Enabling this option will emit [`log`](https://crates.io/crates/log)
    /// messages at the `TRACE` level for read and write operations on connections.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .connection_verbose(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn connection_verbose(mut self, verbose: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.connection_verbose(verbose);
        self
    }

    // HTTP options

    /// Set an optional timeout for idle sockets being kept-alive.
    ///
    /// Pass `None` to disable timeout.
    ///
    /// Unless otherwise set, the default is 90 seconds.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// use std::time::Duration;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .pool_idle_timeout(Some(Duration::from_secs(10)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn pool_idle_timeout<D: Into<Option<Duration>>>(
        mut self,
        value: D,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.pool_idle_timeout(value);
        self
    }

    /// Sets the maximum idle connection per host allowed in the pool.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .pool_max_idle_per_host(10);
    /// # Ok(())
    /// # }
    /// ```
    pub fn pool_max_idle_per_host(mut self, value: usize) -> BriteVerifyClientBuilder {
        self.builder = self.builder.pool_max_idle_per_host(value);
        self
    }

    /// Send headers as title case instead of lowercase.
    ///
    /// Enabling this means that header key-value pairs
    /// that would normally be sent as:
    ///
    /// ```yaml
    /// {
    ///   # ...
    ///   "some-header-key": "The Best Header Value Ever Conceived By Gods Or Men",
    ///   "anotherheaderkey": "A Header Value So Terrible It Must Never Be Spoken Of",
    ///   # ...
    /// }
    /// ```
    ///
    /// will instead be sent as:
    ///
    /// ```yaml
    /// {
    ///   # ...
    ///   "Some-Header-Key": "The Headerless Horseman, Terror Of Sleepy Hollow",
    ///   "AnotherHeaderKey": "The Multi-Headed Centaur, Joy Of Wakeful Solidity",
    ///   # ...
    /// }
    /// ```
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http1_title_case_headers();
    /// # Ok(())
    /// # }
    /// ```
    pub fn http1_title_case_headers(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http1_title_case_headers();
        self
    }

    /// Set whether *HTTP/1* connections will accept obsolete line folding for
    /// header values.
    ///
    /// When enabled, newline codepoints (`\r` and `\n`) will be transformed to
    /// spaces when parsing.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http1_allow_obsolete_multiline_headers_in_responses(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http1_allow_obsolete_multiline_headers_in_responses(
        mut self,
        value: bool,
    ) -> BriteVerifyClientBuilder {
        self.builder = self
            .builder
            .http1_allow_obsolete_multiline_headers_in_responses(value);
        self
    }

    /// Only use *HTTP/1*.
    ///
    /// Calling this method implicitly disables the use of
    /// *HTTP/2* and/or *HTTP/3*.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http1_only();
    /// # Ok(())
    /// # }
    /// ```
    pub fn http1_only(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http1_only();
        self
    }

    /// Allow *HTTP/0.9* responses
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http09_responses();
    /// # Ok(())
    /// # }
    /// ```
    pub fn http09_responses(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http09_responses();
        self
    }

    /// Only use *HTTP/2*.
    ///
    /// Calling this method implicitly disables the use of
    /// *HTTP/1* and/or *HTTP/3*.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_prior_knowledge();
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_prior_knowledge(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_prior_knowledge();
        self
    }

    /// Sets the `SETTINGS_INITIAL_WINDOW_SIZE` option for *HTTP/2*
    /// stream-level flow control.
    ///
    /// Default is currently 65,535 but may change internally to
    /// optimize for common uses.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_initial_stream_window_size(32_767u32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_initial_stream_window_size<WindowSize: Into<Option<u32>>>(
        mut self,
        value: WindowSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_initial_stream_window_size(value);
        self
    }

    /// Sets the max connection-level flow control for *HTTP/2*
    ///
    /// Default is currently 65,535 but may change internally to
    /// optimize for common uses.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_initial_connection_window_size(16_383u32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_initial_connection_window_size<WindowSize: Into<Option<u32>>>(
        mut self,
        value: WindowSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_initial_connection_window_size(value);
        self
    }

    /// Sets whether to use an adaptive flow control.
    ///
    /// Enabling this will override the limits set in
    /// [`http2_initial_stream_window_size`] and
    /// [`http2_initial_connection_window_size`].
    ///
    /// [`http2_initial_stream_window_size`]: #method.http2_initial_stream_window_size
    /// [`http2_initial_connection_window_size`]: #method.http2_initial_connection_window_size
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_adaptive_window(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_adaptive_window(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_adaptive_window(enabled);
        self
    }

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Default is currently 16,384 but may change internally
    /// to optimize for common uses.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_max_frame_size(8_192u32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_max_frame_size<FrameSize: Into<Option<u32>>>(
        mut self,
        value: FrameSize,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_max_frame_size(value);
        self
    }

    /// Sets the interval for sending *HTTP/2* ping frames to
    /// keep a connection alive.
    ///
    /// Pass `None` to disable *HTTP/2* keep-alive.
    /// Default is currently disabled.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// use std::time::Duration;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_keep_alive_interval(Some(Duration::from_secs(10)));
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_keep_alive_interval<Interval: Into<Option<Duration>>>(
        mut self,
        interval: Interval,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_interval(interval);
        self
    }

    /// Set the timeout for receiving an acknowledgement of
    /// *HTTP/2* keep-alive ping frames.
    ///
    /// If a ping is not acknowledged within the timeout,
    /// the connection will be closed. Does nothing if `http2_keep_alive_interval`
    /// is disabled. Default is currently disabled.
    ///
    /// [`http2_keep_alive_interval`]: #method.http2_keep_alive_interval
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// use std::time::Duration;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_keep_alive_timeout(Duration::from_secs(2));
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_keep_alive_timeout(mut self, timeout: Duration) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_timeout(timeout);
        self
    }

    /// Sets whether *HTTP/2* keep-alive should apply while the connection is idle.
    ///
    /// If disabled, keep-alive pings are only sent while there are open
    /// request/responses streams. If enabled, pings are also sent when no
    /// streams are active. Does nothing if `http2_keep_alive_interval` is disabled.
    /// Default is `false`.
    ///
    ///[`http2_keep_alive_interval`]: #method.http2_keep_alive_interval
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .http2_keep_alive_while_idle(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn http2_keep_alive_while_idle(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.http2_keep_alive_while_idle(enabled);
        self
    }

    // TCP options

    /// Set whether sockets have `TCP_NODELAY` enabled.
    ///
    /// Default is `true`.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .tcp_nodelay(false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tcp_nodelay(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tcp_nodelay(enabled);
        self
    }

    /// Bind to a local IP Address.
    ///
    /// #### Example
    ///
    /// ```no_run
    /// use std::net::IpAddr;
    ///
    /// # fn doc() -> anyhow::Result<()> {
    /// let local_addr = IpAddr::from([12, 4, 1, 8]);
    ///
    /// let client = briteverify_rs::BriteVerifyClient::builder()
    ///     .api_key("YOUR API KEY")
    ///     .local_address(local_addr)
    ///     .build()?;
    /// # Ok(())
    /// # }
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// use std::time::Duration;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .tcp_keepalive(Some(Duration::from_secs(2)));
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use std::io::Read;
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let mut buf = Vec::new();
    ///
    /// std::fs::File::open("my_cert.pem")?.read_to_end(&mut buf)?;
    ///
    /// let cert = reqwest::Certificate::from_pem(&buf)?;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .add_root_certificate(cert);
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_root_certificate(mut self, cert: reqwest::Certificate) -> BriteVerifyClientBuilder {
        self.builder = self.builder.add_root_certificate(cert);
        self
    }

    /// Controls the use of built-in/preloaded certificates during certificate validation.
    ///
    /// Defaults to `true`, meaning built-in system certs will be used.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .tls_built_in_root_certs(false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tls_built_in_root_certs(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tls_built_in_root_certs(enabled);
        self
    }

    /// Sets the identity to be used for client certificate authentication.
    ///
    /// #### Example
    /// ```no_run
    /// # use std::io::Read;
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let mut buf = Vec::new();
    ///
    /// std::fs::File::open("my_cert.pem")?.read_to_end(&mut buf)?;
    ///
    /// let identity = reqwest::Identity::from_pem(&buf)?;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .identity(identity);
    /// # Ok(())
    /// # }
    /// ```
    pub fn identity(mut self, value: reqwest::Identity) -> BriteVerifyClientBuilder {
        self.builder = self.builder.identity(value);
        self
    }

    /// Controls the use of certificate validation.
    ///
    /// Defaults to `false`.
    ///
    /// ## **Warning**
    ///
    /// You should think very carefully before using this method. If
    /// invalid certificates are trusted, *any* certificate for *any* site
    /// will be trusted for use. This includes expired certificates. This
    /// introduces significant vulnerabilities, and should only be used
    /// as a last resort.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// // NOTE: Read the warning above, then read it again.
    /// //       You can do this, but it's a virtual guarantee
    /// //       that you shouldn't.
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .danger_accept_invalid_certs(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn danger_accept_invalid_certs(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.danger_accept_invalid_certs(enabled);
        self
    }

    /// Controls the use of TLS server name indication.
    ///
    /// Defaults to `true`.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .tls_sni(false);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tls_sni(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.tls_sni(enabled);
        self
    }

    /// Set the minimum required TLS version for connections.
    ///
    /// By default the TLS backend's own default is used.
    ///
    /// #### Errors
    ///
    /// A value of `tls::Version::TLS_1_3` will cause an error with `reqwest`'s
    /// `native-tls` or `default-tls` backends. This does not mean the version
    /// isn't supported, just that it can't be set as a minimum due to
    /// technical limitations.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .min_tls_version(reqwest::tls::Version::TLS_1_1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn min_tls_version(mut self, version: reqwest::tls::Version) -> BriteVerifyClientBuilder {
        self.builder = self.builder.min_tls_version(version);
        self
    }

    /// Set the maximum allowed TLS version for connections.
    ///
    /// By default there's no maximum.
    ///
    /// #### Errors
    ///
    /// A value of `tls::Version::TLS_1_3` will cause an error with `reqwest`'s
    /// `native-tls` or `default-tls` backends. This does not mean the version
    /// isn't supported, just that it can't be set as a maximum due to
    /// technical limitations.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .max_tls_version(reqwest::tls::Version::TLS_1_2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn max_tls_version(mut self, version: reqwest::tls::Version) -> BriteVerifyClientBuilder {
        self.builder = self.builder.max_tls_version(version);
        self
    }

    /// Disables the trust-dns async resolver.
    ///
    /// This method exists even if `reqwest`'s optional `trust-dns`
    /// feature is not enabled. This can be used to ensure a `BriteVerifyClient`
    /// doesn't use the trust-dns async resolver even if another dependency were
    /// to enable the optional `trust-dns` feature.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .no_trust_dns();
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_trust_dns(mut self) -> BriteVerifyClientBuilder {
        self.builder = self.builder.no_trust_dns();
        self
    }

    /// Restrict the constructed `BriteVerifyClient` using only HTTPS requests.
    ///
    /// Defaults to false.
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .https_only(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn https_only(mut self, enabled: bool) -> BriteVerifyClientBuilder {
        self.builder = self.builder.https_only(enabled);
        self
    }

    /// Override DNS resolution for specific domains to a particular IP address.
    ///
    /// ## **Warning**
    ///
    /// Since the DNS protocol has no notion of ports, if you wish to send
    /// traffic to a particular port you must include this port in the URL
    /// itself, any port in the overridden address will be ignored and traffic
    /// will be sent to the conventional port for the given scheme (e.g. 80 for http).
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let resolver: std::net::SocketAddr = "[::]:53".parse()?;
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .resolve("my.super-awesome-domain.net", resolver);
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve(
        mut self,
        domain: &str,
        address: std::net::SocketAddr,
    ) -> BriteVerifyClientBuilder {
        self.builder = self.builder.resolve(domain, address);
        self
    }

    /// Override DNS resolution for specific domains to a set of particular IP addresses.
    ///
    /// ## **Warning**
    ///
    /// Since the DNS protocol has no notion of ports, if you wish to send
    /// traffic to a particular port you must include this port in the URL
    /// itself, any port in the overridden addresses will be ignored and traffic
    /// will be sent to the conventional port for the given scheme (e.g. 80 for http).
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let resolvers: [std::net::SocketAddr; 3] = [
    ///     "1.1.1.1:53".parse()?,
    ///     "8.8.8.8:53".parse()?,
    ///     "2001:4860:4860::8844:53".parse()?,
    /// ];
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .resolve_to_addrs("my.super-awesome-domain.net", &resolvers);
    /// # Ok(())
    /// # }
    /// ```
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
    /// Pass an [`Arc`](std::sync::Arc) wrapping any object that implements
    /// [`Resolve`](reqwest::dns::Resolve). Overrides for specific names passed
    /// to [`resolve`] and [`resolve_to_addrs`] will still be applied on top of this
    /// resolver.
    ///
    /// [`resolve`]: #method.resolve
    /// [`resolve_to_addrs`]: #method.resolve_to_addrs
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc<Resolver: reqwest::dns::Resolve + 'static>() -> anyhow::Result<()> {
    /// # type Resolver = ();
    /// // NOTE: expected type of `Resolver` is reqwest::dns::Resolve + 'static
    /// //       when used, the actual object will likely be specific to your implementation
    /// let my_resolver: Resolver = ();
    ///
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .dns_resolver(std::sync::Arc::new(my_resolver));
    /// # Ok(())
    /// # }
    /// ```
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
///
/// ## Basic Usage
/// ```no_run
/// # use std::time::Duration;
/// # use briteverify_rs::{BriteVerifyClient, types::AccountCreditBalance};
/// #
/// # #[tokio::main]
/// # async fn doc() -> anyhow::Result<()> {
/// let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
///
/// let balance: AccountCreditBalance = client.get_account_balance().await?;
///
/// println!("{balance:#?}");
/// # Ok(())
/// # }
/// ```
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
    // <editor-fold desc="// Constructors ... ">

    /// Create a new [`BriteVerifyClient`][BriteVerifyClient] instance
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<ApiKey: ToString>(api_key: ApiKey) -> Result<Self, errors::BriteVerifyClientError> {
        Self::builder().api_key(api_key).build()
    }

    /// Create a new [builder][BriteVerifyClientBuilder] to incrementally
    /// build a [`BriteVerifyClient`][BriteVerifyClient] with a customised
    /// configuration
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, BriteVerifyClientBuilder};
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClient::builder();
    ///
    /// // ... call various builder methods
    ///
    /// let client: BriteVerifyClient = builder.build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> BriteVerifyClientBuilder {
        BriteVerifyClientBuilder::new()
    }

    // </editor-fold desc="// Constructors ... ">

    // <editor-fold desc="// Internal Utility Methods ... ">

    #[allow(dead_code, unused_variables)]
    /// [internal-implementation]
    /// Send the supplied request, automatically handling
    /// rate limit error responses by sleeping until the
    /// rate limit expires and re-sending the request
    async fn _with_retry(request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        todo!()
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

    // </editor-fold desc="// Internal Utility Methods ... ">

    // <editor-fold desc="// Real-Time Single Transaction Endpoints ... ">

    /// Get your current account credit balance
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let balance: u32 = client.current_credits().await?;
    ///
    /// println!("Current BriteVerify API credit balance: {balance}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_credits(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits)
    }

    /// Get the total number of credits your account currently has in reserve
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let reserved: u32 = client.current_credits_in_reserve().await?;
    ///
    /// println!("Current BriteVerify API reserve credit balance: {reserved}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_credits_in_reserve(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits_in_reserve)
    }

    /// Get your account credit balance, total number of credits
    /// in reserve, and the timestamp of when your balance was
    /// most recently recorded
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::AccountCreditBalance};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let balance_report: AccountCreditBalance = client.get_account_balance().await?;
    ///
    /// println!("Current BriteVerify API credit data: {balance_report}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_account_balance(&self) -> Result<types::AccountCreditBalance> {
        let response = self
            .get(format!("{V3_API_BASE_URL}/accounts/credits"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::AccountCreditBalance>().await?),
            StatusCode::TOO_MANY_REQUESTS => {
                todo!("Add automatic rate limit handling")
            }
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Verify a "complete" contact record
    /// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::FullVerificationResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let verified: FullVerificationResponse = client.verify_contact(
    ///     "test@example.com",
    ///     "+15555555555",
    ///     "123 Main St",
    ///     Some("P.O. Box 456"),
    ///     "Any Town",
    ///     "CA",
    ///     "90210",
    /// ).await?;
    ///
    /// println!("Verified contact data: {verified:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::EmailVerificationResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let response: EmailVerificationResponse = client.verify_email("test@example.com").await?;
    ///
    /// println!("Verified email: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::PhoneNumberVerificationResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let response: PhoneNumberVerificationResponse = client.verify_phone_number("+15555555555").await?;
    ///
    /// println!("Verified phone number: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::AddressVerificationResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let verified: AddressVerificationResponse = client.verify_street_address(
    ///     "123 Main St",
    ///     Some("P.O. Box 456"),
    ///     "Any Town",
    ///     "CA",
    ///     "90210",
    /// ).await?;
    ///
    /// println!("Verified address: {verified:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::GetListStatesResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let lists: GetListStatesResponse = client.get_lists().await?;
    ///
    /// println!("Available bulk verification lists: {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use chrono::Datelike;
    /// use chrono::{NaiveDate, Utc};
    /// use briteverify_rs::{BriteVerifyClient, types::GetListStatesResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let today: NaiveDate = Utc::now().date_naive();
    ///
    /// let page: Option<u32> = Some(5u32);
    /// let state: Option<&str> = Some("open");
    /// let date: Option<NaiveDate> = today.with_day(today.day() - 2);
    ///
    /// let lists: GetListStatesResponse = client.get_filtered_lists(page, date, state).await?;
    ///
    /// println!("Filtered bulk verification lists: {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    /// ___
    /// **NOTE:** Regardless of specified date, the BriteVerify API
    /// does not appear to persist bulk verification lists older than
    /// 7 calendar days
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use chrono::Datelike;
    /// use chrono::{NaiveDate, Utc};
    /// use briteverify_rs::{BriteVerifyClient, types::GetListStatesResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let today: NaiveDate = Utc::now().date_naive();
    /// let date: NaiveDate = today.with_day(today.day() - 2).unwrap();
    ///
    /// let lists: GetListStatesResponse = client.get_lists_by_date(&date).await?;
    ///
    /// println!("Bulk verification lists for '{date}': {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Example
    /// ```no_run
    /// # use chrono::Datelike;
    /// use briteverify_rs::{BriteVerifyClient, types::GetListStatesResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let page: u32 = 2;
    /// let lists: GetListStatesResponse = client.get_lists_by_page(page).await?;
    ///
    /// println!("Bulk verification lists page {page}: {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_lists_by_page(&self, page: u32) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            Some(page),
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
        )
        .await
    }

    /// Retrieve the complete list of all bulk verification lists created
    /// within the last 7 calendar days whose status matches the specified
    /// value
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    ///
    /// #### Example
    /// ```no_run
    /// # use chrono::Datelike;
    /// use briteverify_rs::{BriteVerifyClient, types::{BatchState, GetListStatesResponse}};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let state: BatchState = BatchState::Closed;
    /// let lists: GetListStatesResponse = client.get_lists_by_state(state).await?;
    ///
    /// println!("Bulk verification lists w/ state '{state}': {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// #### Examples
    ///
    /// ##### Create Empty List
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::CreateListResponse;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list: CreateListResponse = client.create_list(None, false).await?;
    ///
    /// println!("New bulk verification list: {list:#?}");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ##### Create Populated List & Start Immediately
    /// ```no_run
    /// use briteverify_rs::{
    /// #    BriteVerifyClient,
    ///     types::{
    ///       CreateListResponse,
    ///       VerificationRequest,
    ///     },
    /// };
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let contacts: [VerificationRequest; 2] = [
    ///     VerificationRequest::try_from("test@example.com")?,
    ///     VerificationRequest::try_from("+15555555555")?
    /// ];
    ///
    /// let list: CreateListResponse = client.create_list(Some(contacts), true).await?;
    ///
    /// println!("New bulk verification list: {list:#?}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_list<
        Contact: Into<types::VerificationRequest>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        contacts: Option<ContactCollection>,
        auto_start: bool,
    ) -> Result<types::CreateListResponse> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)

        if let Some(data) = contacts {
            self._create_or_update_list(
                <Option<String>>::None, // no explicit list id
                data,                   // supplied contacts
                auto_start,             // untouched auto-start value
            )
            .await
        } else {
            self._create_or_update_list(
                <Option<String>>::None,                   // no explicit list id
                Vec::<types::VerificationRequest>::new(), // no contacts
                false, // without contacts, we can't auto-start no matter what
            )
            .await
        }
    }

    /// Append records to the specified bulk verification list and (optionally)
    /// queue it for immediate processing
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1:~:text=customer%2DID/lists-,list_id,-(optional))]
    ///
    /// #### Example
    /// ```no_run
    /// use briteverify_rs::{
    /// #    BriteVerifyClient,
    ///     types::{
    ///       UpdateListResponse,
    ///       VerificationRequest,
    ///     },
    /// };
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let contacts: [VerificationRequest; 2] = [
    ///     VerificationRequest::try_from("some-email@bounce-me.net")?,
    ///     VerificationRequest::try_from("another-email@a-real-domain.org")?,
    /// ];
    ///
    /// let list: UpdateListResponse = client.update_list("some-list-id", contacts, false).await?;
    ///
    /// println!("Updated bulk verification list: {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
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

    /// Retrieve current "state" of the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#b09c09dc-e11e-44a8-b53d-9f1fd9c6792d)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::VerificationListState;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;    ///
    ///
    /// let list_id: &str = "some-list-id";
    /// let list: VerificationListState = client.get_list_by_id(list_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}': {list:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    /// - [`BatchState::Prepped`](types::enums::BatchState::Prepped)
    /// - [`BatchState::Complete`](types::enums::BatchState::Complete)
    /// - [`BatchState::Delivered`](types::enums::BatchState::Delivered)
    /// - [`BatchState::ImportError`](types::enums::BatchState::ImportError)
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::DeleteListResponse;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list_id: &str = "some-list-id";
    /// let response: DeleteListResponse = client.delete_list_by_id(list_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}' final state: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// ___
    /// **NOTE:** This action is only applicable to lists
    /// that have *not yet* begun processing. For any list
    /// that has already been "started", the equivalent
    /// action would be [`delete_list_by_id`].
    /// ___
    ///
    /// [`delete_list_by_id`]: #delete_list_by_id
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::UpdateListResponse;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list_id: &str = "some-list-id";
    /// let response: UpdateListResponse = client.terminate_list_by_id(list_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}' final state: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
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

    /// Queue the specified (open) bulk verification list for immediate processing
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831:~:text=immediately%20start%20a%20list)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::UpdateListResponse;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list_id: &str = "some-list-id";
    /// let response: UpdateListResponse = client.queue_list_for_processing(list_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}' state: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
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

    /// Get the verification results for the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831)]
    ///
    /// ___
    /// **NOTE:** Verification results are only available once
    /// a list has finished verifying in its entirety. It is not
    /// possible to retrieve verification results piecemeal.
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::BulkVerificationResult;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list_id: &str = "some-list-id";
    /// let data: Vec<BulkVerificationResult> = client.get_results_by_list_id(list_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}' results: {data:#?}");
    /// # Ok(())
    /// # }
    /// ```
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

    // </editor-fold desc="// Bulk Verification (v3) Endpoints ... ">
}

// </editor-fold desc="// Client ...">
