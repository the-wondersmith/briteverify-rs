#![allow(unused_qualifications)]
//! ## BriteVerify API Client
//
// Standard Library Imports
#[allow(unused_imports)]
use std::{fmt::Debug, net::SocketAddr, ops::Deref, time::Duration};

// Third-Party Imports
use anyhow::{Context, Result};
use futures_timer::Delay;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    StatusCode,
};

#[cfg(feature = "tracing")]
use instrumentation as tracing;

// Crate-Level Imports
use crate::errors::BriteVerifyClientError;
use crate::{errors, types, utils::ExtensibleUrl};

// <editor-fold desc="// Constants ...">

type Nullable = Option<String>;
static V1_API_BASE_URL: &str = "https://bpi.briteverify.com/api/v1";
static V3_API_BASE_URL: &str = "https://bulk-api.briteverify.com/api/v3";

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
#[derive(Debug)]
#[cfg_attr(test, visible::StructFields(pub))]
pub struct BriteVerifyClientBuilder {
    error: Option<errors::BriteVerifyClientError>,
    api_key: Option<HeaderValue>,
    v1_base_url: url::Url,
    v3_base_url: url::Url,
    retry_enabled: bool,
    builder: reqwest::ClientBuilder,
}

impl From<reqwest::ClientBuilder> for BriteVerifyClientBuilder {
    fn from(builder: reqwest::ClientBuilder) -> Self {
        Self {
            api_key: if !crate::utils::has_auth_header(&builder) {
                None
            } else {
                Some(HeaderValue::from_static("IGNORE ME"))
            },
            builder,
            ..Self::default()
        }
    }
}

impl Default for BriteVerifyClientBuilder {
    fn default() -> Self {
        Self {
            error: None,
            api_key: None,
            v1_base_url: url::Url::parse(V1_API_BASE_URL)
                .expect("Couldn't parse default v1 base url"),
            v3_base_url: url::Url::parse(V3_API_BASE_URL)
                .expect("Couldn't parse default v1 base url"),
            retry_enabled: false,
            builder: reqwest::Client::builder(),
        }
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
        Self::default()
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn build(mut self) -> Result<BriteVerifyClient, errors::BriteVerifyClientError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        match self.api_key {
            None => Err(errors::BriteVerifyClientError::MissingApiKey),
            Some(key) => {
                if key.is_sensitive() {
                    let headers = HeaderMap::from_iter([(AUTHORIZATION, key)]);
                    self.builder = self.builder.default_headers(headers);
                }

                Ok(BriteVerifyClient {
                    client: self
                        .builder
                        .build()
                        .context("Could not create a usable `reqwest` client")?,
                    v1_base_url: self.v1_base_url,
                    v3_base_url: self.v3_base_url,
                    retry_enabled: self.retry_enabled,
                })
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
        let api_key: String = format!(
            "ApiKey: {}",
            api_key.to_string().replace("ApiKey: ", "").trim()
        );

        match HeaderValue::from_str(&api_key) {
            Ok(mut header) => {
                header.set_sensitive(true);
                self.api_key = Some(header);

                if self.error.as_ref().is_some_and(|err| {
                    matches!(err, &errors::BriteVerifyClientError::InvalidHeaderValue(_))
                }) {
                    self.error = None;
                }
            }
            Err(error) => {
                self.api_key = None;
                self.error = Some(error.into());
            }
        }

        self
    }

    /// Enabled or disable automatic rate limit handling via retry.
    ///
    /// ___
    /// **NOTE:** Automatic retry is `disabled` by default. It must be
    /// explicitly enabled by calling `.retry_enabled(true)` on a
    /// [`BriteVerifyClientBuilder`](BriteVerifyClientBuilder) instance.
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .retry_enabled(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry_enabled(mut self, value: bool) -> Self {
        self.retry_enabled = value;
        self
    }

    /// Override the base URL for requests to the BriteVerify v1 API
    /// [[ref](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f)]
    ///
    /// ___
    /// **NOTE:** Unless overridden (specifically by calling [`v1_base_url`]
    /// on a builder instance), the default value of `https://bpi.briteverify.com/api/v1`
    /// will be used as the base url for single-transaction requests.
    ///
    /// If you set a custom url, be aware that no additional logic, formatting,
    /// or validity checks will be applied to whatever value you specify.
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .v1_base_url("https://my-custom-domain.net/briteverify/v1");
    /// # Ok(())
    /// # }
    /// ```
    pub fn v1_base_url<URL>(mut self, url: URL) -> Self
    where
        URL: TryInto<url::Url>,
        URL::Error: Into<BriteVerifyClientError>,
    {
        let url = url.try_into();

        match url {
            Ok(value) => {
                self.v1_base_url = value;
            }
            Err(error) => {
                self.error = Some(error.into());
            }
        }

        self
    }

    /// Override the base URL for requests to the BriteVerify v3 API
    /// [[ref](https://docs.briteverify.com/#382f454d-dad2-49c3-b320-c7d117fcc20a)]
    ///
    /// ___
    /// **NOTE:** Unless overridden (specifically by calling [`v3_base_url`]
    /// on a builder instance), the default value of `https://bulk-api.briteverify.com/api/v3`
    /// will be used as the base url for bulk transaction requests.
    ///
    /// If you set a custom url, be aware that no additional logic, formatting,
    /// or validity checks will be applied to whatever value you specify.
    /// ___
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClientBuilder;
    /// #
    /// # fn doc() -> anyhow::Result<()> {
    /// let builder: BriteVerifyClientBuilder = BriteVerifyClientBuilder::new()
    ///     .v3_base_url("https://my-custom-domain.net/briteverify/v3");
    /// # Ok(())
    /// # }
    /// ```
    pub fn v3_base_url<URL>(mut self, url: URL) -> Self
    where
        URL: TryInto<url::Url>,
        URL::Error: Into<BriteVerifyClientError>,
    {
        let url = url.try_into();

        match url {
            Ok(value) => {
                self.v3_base_url = value;
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    #[cfg_attr(tarpaulin, coverage(off))]
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

        if enabled {
            self.v1_base_url
                .set_scheme(http::uri::Scheme::HTTPS.as_str())
                .unwrap_or_else(|_| log::error!("Could not set `v1_base_url` scheme to HTTPS"));
            self.v3_base_url
                .set_scheme(http::uri::Scheme::HTTPS.as_str())
                .unwrap_or_else(|_| log::error!("Could not set `v3_base_url` scheme to HTTPS"));
        }

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
    #[cfg_attr(tarpaulin, coverage(off))]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn resolve(
        mut self,
        domain: &str,
        address: std::net::SocketAddr,
    ) -> BriteVerifyClientBuilder {
        log::debug!("DNS resolver installed for: '{domain}' -> {:?}", &address);
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
    /// ```ignore
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
    #[cfg_attr(tarpaulin, coverage(off))]
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
#[cfg_attr(test, visible::StructFields(pub))]
pub struct BriteVerifyClient {
    client: reqwest::Client,
    v1_base_url: url::Url,
    v3_base_url: url::Url,
    retry_enabled: bool,
}

impl Deref for BriteVerifyClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl TryFrom<reqwest::Client> for BriteVerifyClient {
    type Error = errors::BriteVerifyClientError;

    fn try_from(client: reqwest::Client) -> Result<Self, Self::Error> {
        if crate::utils::has_auth_header(&client) {
            Ok(Self {
                client,
                retry_enabled: true,
                v1_base_url: V1_API_BASE_URL.parse::<url::Url>().unwrap(),
                v3_base_url: V3_API_BASE_URL.parse::<url::Url>().unwrap(),
            })
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

    /// [internal-implementation]
    /// Build and send the supplied request
    ///
    /// If `retry_enabled` is true, rate limit error responses
    /// will be automatically handled by sleeping until the rate
    /// limit expires and re-sending the request
    async fn _build_and_send(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, errors::BriteVerifyClientError> {
        loop {
            let response = (match builder.try_clone() {
                Some(instance) => instance,
                None => break Err(errors::BriteVerifyClientError::UnclonableRequest),
            })
            .send()
            .await?;

            match (&self.retry_enabled, response.status()) {
                (_, StatusCode::UNAUTHORIZED) => {
                    break Err(errors::BriteVerifyClientError::InvalidApiKey);
                }
                (&true, StatusCode::TOO_MANY_REQUESTS) => {
                    let retry_after = 1 + response
                        .headers()
                        .get("retry-after")
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(60);

                    log::warn!(
                        "Request to '{}' responded 429, waiting {} seconds before retry...",
                        response.url(),
                        &retry_after
                    );

                    Delay::new(Duration::from_secs(retry_after)).await;
                }
                _ => {
                    break Ok(response);
                }
            }
        }
    }

    /// [internal-implementation]
    /// Actually perform a single-transaction verification
    #[allow(clippy::too_many_arguments)]
    async fn _full_verify<
        EmailAddress: ToString,
        PhoneNumber: ToString,
        AddressLine1: ToString,
        AddressLine2: ToString,
        CityName: ToString,
        StateNameOrAbbr: ToString,
        ZipCode: ToString,
    >(
        &self,
        email: Option<EmailAddress>,
        phone: Option<PhoneNumber>,
        address1: Option<AddressLine1>,
        address2: Option<AddressLine2>,
        city: Option<CityName>,
        state: Option<StateNameOrAbbr>,
        zip: Option<ZipCode>,
    ) -> Result<types::VerificationResponse, errors::BriteVerifyClientError> {
        let request = types::VerificationRequest::from_values(
            email, phone, address1, address2, city, state, zip,
        )?;

        let url = self.v1_base_url.append_path("fullverify");

        let response = self._build_and_send(self.post(url).json(&request)).await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::VerificationResponse>().await?),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
        }
    }

    /// [internal-implementation]
    /// Actually fetch a given [`VerificationListState`](types::VerificationListState)
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    async fn _get_list_state<ListId: ToString + Debug, ExternalId: std::fmt::Display + Debug>(
        &self,
        list_id: ListId,
        external_id: Option<ExternalId>,
    ) -> Result<types::VerificationListState, errors::BriteVerifyClientError> {
        let list_id = list_id.to_string();
        let url = external_id
            .map(|ext_id| {
                self.v3_base_url
                    .extend_path(["accounts".to_string(), ext_id.to_string()])
            })
            .as_ref()
            .unwrap_or(&self.v3_base_url)
            .extend_path(["lists", &list_id]);

        let response = self._build_and_send(self.get(url)).await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::VerificationListState>().await?),
            StatusCode::NOT_FOUND => Err(errors::BriteVerifyClientError::BulkListNotFound(
                Box::new(types::BulkListCRUDError {
                    list_id: Some(list_id),
                    ..response.json::<types::BulkListCRUDError>().await?
                }),
            )),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
        }
    }

    /// [internal-implementation]
    /// Retrieve the specified page of results from the specified
    /// bulk verification list
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    async fn _get_result_page(
        &self,
        list_id: String,
        page_number: u64,
    ) -> Result<types::BulkVerificationResponse, errors::BriteVerifyClientError> {
        let page_url = self.v3_base_url.extend_path([
            "lists",
            &list_id,
            "export",
            page_number.to_string().as_str(),
        ]);

        let response = self._build_and_send(self.get(page_url)).await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::BulkVerificationResponse>().await?),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
        }
    }

    /// [internal-implementation]
    /// Create a new or mutate an extant bulk verification list
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    async fn _create_or_update_list<
        ListId: ToString + Debug,
        Contact: Into<types::VerificationRequest> + Debug,
        Directive: Into<types::BulkListDirective> + Debug,
        ContactCollection: IntoIterator<Item = Contact> + Debug,
    >(
        &self,
        list_id: Option<ListId>,
        contacts: ContactCollection,
        directive: Directive,
    ) -> Result<types::CreateListResponse, errors::BriteVerifyClientError> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)

        let directive = directive.into();
        let request = types::BulkVerificationRequest::new(contacts, directive);

        let mut url = self.v3_base_url.append_path("lists");

        if let Some(id) = list_id.as_ref() {
            url = url.append_path(id.to_string());
        }

        let response = self._build_and_send(self.post(url).json(&request)).await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                Ok(response.json::<types::CreateListResponse>().await?)
            }
            StatusCode::NOT_FOUND | StatusCode::BAD_REQUEST => {
                Err(errors::BriteVerifyClientError::BulkListNotFound(Box::new(
                    types::BulkListCRUDError {
                        list_id: list_id.as_ref().map(|id| id.to_string()),
                        ..response.json::<types::BulkListCRUDError>().await?
                    },
                )))
            }
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_account_balance(
        &self,
    ) -> Result<types::AccountCreditBalance, errors::BriteVerifyClientError> {
        let url = format!("{}/accounts/credits", &self.v3_base_url);
        let response = self._build_and_send(self.get(url)).await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::AccountCreditBalance>().await?),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
        }
    }

    /// Verify a "complete" contact record
    /// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::VerificationResponse};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let verified: VerificationResponse = client.verify_contact(
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
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn verify_contact<
        EmailAddress: ToString + Debug,
        PhoneNumber: ToString + Debug,
        AddressLine1: ToString + Debug,
        AddressLine2: ToString + Debug,
        CityName: ToString + Debug,
        StateNameOrAbbr: ToString + Debug,
        ZipCode: ToString + Debug,
    >(
        &self,
        email: EmailAddress,
        phone: PhoneNumber,
        address1: AddressLine1,
        address2: Option<AddressLine2>,
        city: CityName,
        state: StateNameOrAbbr,
        zip: ZipCode,
    ) -> Result<types::VerificationResponse, errors::BriteVerifyClientError> {
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
            .await;

        match response {
            Ok(data) => Ok(data),
            Err(error) => Err(error),
        }
    }

    /// Verify a single email address
    /// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::EmailVerificationArray};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let response: EmailVerificationArray = client.verify_email("test@example.com").await?;
    ///
    /// println!("Verified email: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn verify_email<EmailAddress: ToString + Debug>(
        &self,
        email: EmailAddress,
    ) -> Result<types::EmailVerificationArray, errors::BriteVerifyClientError> {
        let response = self
            ._full_verify(
                Some(email),
                Nullable::None,
                Nullable::None,
                Nullable::None,
                Nullable::None,
                Nullable::None,
                Nullable::None,
            )
            .await?;

        match response.email {
            Some(data) => Ok(data),
            None => Err(
                errors::BriteVerifyClientError::MismatchedVerificationResponse(Box::new(response)),
            ),
        }
    }

    /// Verify a single phone number
    /// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::PhoneNumberVerificationArray};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let response: PhoneNumberVerificationArray = client.verify_phone_number("+15555555555").await?;
    ///
    /// println!("Verified phone number: {response:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn verify_phone_number<PhoneNumber: ToString + Debug>(
        &self,
        phone: PhoneNumber,
    ) -> Result<types::PhoneNumberVerificationArray, errors::BriteVerifyClientError> {
        let response = self
            ._full_verify(
                Nullable::None,
                Some(phone),
                Nullable::None,
                Nullable::None,
                Nullable::None,
                Nullable::None,
                Nullable::None,
            )
            .await?;

        match response.phone {
            Some(data) => Ok(data),
            None => Err(
                errors::BriteVerifyClientError::MismatchedVerificationResponse(Box::new(response)),
            ),
        }
    }

    /// Verify a single street address
    /// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::{BriteVerifyClient, types::AddressVerificationArray};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    /// let verified: AddressVerificationArray = client.verify_street_address(
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn verify_street_address<
        AddressLine1: ToString + Debug,
        AddressLine2: ToString + Debug,
        CityName: ToString + Debug,
        StateNameOrAbbr: ToString + Debug,
        ZipCode: ToString + Debug,
    >(
        &self,
        address1: AddressLine1,
        address2: Option<AddressLine2>,
        city: CityName,
        state: StateNameOrAbbr,
        zip: ZipCode,
    ) -> Result<types::AddressVerificationArray, errors::BriteVerifyClientError> {
        let response = self
            ._full_verify(
                Nullable::None,
                Nullable::None,
                Some(address1),
                address2,
                Some(city),
                Some(state),
                Some(zip),
            )
            .await?;

        match response.address {
            Some(data) => Ok(data),
            None => Err(
                errors::BriteVerifyClientError::MismatchedVerificationResponse(Box::new(response)),
            ),
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_lists(
        &self,
    ) -> Result<types::GetListStatesResponse, errors::BriteVerifyClientError> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
            Nullable::None,
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
    /// let ext_id: Option<&str> = None;
    ///
    /// let lists: GetListStatesResponse = client.get_filtered_lists(page, date, state, ext_id).await?;
    ///
    /// println!("Filtered bulk verification lists: {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_filtered_lists<
        'header,
        Date: chrono::Datelike + Debug,
        Page: Into<u32> + Debug,
        State: Clone + Debug + Into<types::BatchState>,
        ExternalId: std::fmt::Display + Debug,
    >(
        &self,
        page: Option<Page>,
        date: Option<Date>,
        state: Option<State>,
        ext_id: Option<ExternalId>,
    ) -> Result<types::GetListStatesResponse, errors::BriteVerifyClientError> {
        let mut params: Vec<(&'header str, String)> = Vec::new();

        if let Some(page) = page {
            params.push(("page", page.into().to_string()));
        }

        if let Some(date) = date {
            params.push((
                "date",
                format!("{}-{:0>2}-{:0>2}", date.year(), date.month(), date.day()),
            ));
        }

        if let Some(state) = state {
            let filter = state.clone().into();

            if matches!(filter, types::BatchState::Unknown) {
                log::warn!("Declining to include unknown list state as request filter: {state:#?}");
            } else {
                params.push(("state", filter.to_string()));
            }
        }

        let url = ext_id
            .map(|id| {
                self.v3_base_url
                    .extend_path(["accounts".to_string(), id.to_string()])
            })
            .as_ref()
            .unwrap_or(&self.v3_base_url)
            .append_path("lists");

        let mut request = self.get(url);

        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = self._build_and_send(request).await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::GetListStatesResponse>().await?),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
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
    /// let lists: GetListStatesResponse = client.get_lists_by_date(date.clone()).await?;
    ///
    /// println!("Bulk verification lists for '{date}': {lists:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_lists_by_date<Date: chrono::Datelike + Debug>(
        &self,
        date: Date,
    ) -> Result<types::GetListStatesResponse, errors::BriteVerifyClientError> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            Some(date),
            <Option<types::BatchState>>::None,
            Nullable::None,
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_lists_by_page<Page: Into<u32> + Debug>(
        &self,
        page: Page,
    ) -> Result<types::GetListStatesResponse, errors::BriteVerifyClientError> {
        self.get_filtered_lists(
            Some(page),
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
            Nullable::None,
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_lists_by_state(
        &self,
        state: types::BatchState,
    ) -> Result<types::GetListStatesResponse, errors::BriteVerifyClientError> {
        if !state.is_unknown() {
            self.get_filtered_lists(
                <Option<u32>>::None,
                <Option<chrono::NaiveDate>>::None,
                Some(state),
                Nullable::None,
            )
            .await
        } else {
            let message = "to request lists using 'unknown' as list state filter";

            log::warn!("Declining {message}");

            Ok(types::GetListStatesResponse {
                message: Some(format!("Declined {message}")),
                lists: Vec::new(),
            })
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
    /// use briteverify_rs::types::{CreateListResponse, VerificationRequest};
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let contacts = <Option<Vec<VerificationRequest>>>::None;
    /// let list: CreateListResponse = client.create_list(contacts, false).await?;
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn create_list<
        Contact: Into<types::VerificationRequest> + Debug,
        ContactCollection: IntoIterator<Item = Contact> + Debug,
    >(
        &self,
        contacts: Option<ContactCollection>,
        auto_start: bool,
    ) -> Result<types::CreateListResponse, errors::BriteVerifyClientError> {
        // TODO(the-wondersmith): Apply bulk "rate" limit to supplied contacts
        //                        Bulk rate limits are:
        //                          - 100k Emails per page
        //                          - 1M Email addresses per job (or 20 pages of 50k)

        if let Some(data) = contacts {
            self._create_or_update_list(
                Nullable::None, // no explicit list id
                data,           // supplied contacts
                auto_start,     // untouched auto-start value
            )
            .await
        } else {
            self._create_or_update_list(
                Nullable::None,                           // no explicit list id
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
    /// println!("Updated bulk verification list: {list:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn update_list<
        ListId: ToString + Debug,
        Contact: Into<types::VerificationRequest> + Debug,
        ContactCollection: IntoIterator<Item = Contact> + Debug,
    >(
        &self,
        list_id: ListId,
        contacts: ContactCollection,
        auto_start: bool,
    ) -> Result<types::UpdateListResponse, errors::BriteVerifyClientError> {
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_list_by_id<ListId: ToString + Debug>(
        &self,
        list_id: ListId,
    ) -> Result<types::VerificationListState, errors::BriteVerifyClientError> {
        self._get_list_state(list_id, Nullable::None).await
    }

    /// Retrieve current "state" of a bulk verification list tied to an
    /// externally supplied / customer-specific identifier
    /// [[ref](https://docs.briteverify.com/#b09c09dc-e11e-44a8-b53d-9f1fd9c6792d)]
    ///
    /// #### Example
    /// ```no_run
    /// # use briteverify_rs::BriteVerifyClient;
    /// use briteverify_rs::types::VerificationListState;
    /// #
    /// # async fn doc() -> anyhow::Result<()> {
    /// # let client: BriteVerifyClient = BriteVerifyClient::new("YOUR API KEY")?;
    ///
    /// let list_id: &str = "some-list-id";
    /// let customer_id: &str = "some-customer-id";
    /// let list: VerificationListState = client.get_list_by_external_id(list_id, customer_id).await?;
    ///
    /// println!("Bulk verification list '{list_id}': {list:#?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_list_by_external_id<
        ListId: ToString + Debug,
        ExternalId: std::fmt::Display + Debug,
    >(
        &self,
        list_id: ListId,
        external_id: ExternalId,
    ) -> Result<types::VerificationListState, errors::BriteVerifyClientError> {
        self._get_list_state(list_id, Some(external_id)).await
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn delete_list_by_id<ListId: ToString + Debug>(
        &self,
        list_id: ListId,
    ) -> Result<types::DeleteListResponse, errors::BriteVerifyClientError> {
        let list_id: String = list_id.to_string();
        let url = self.v3_base_url.extend_path(["lists", &list_id]);

        let response = self.delete(url).send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::ACCEPTED | StatusCode::NO_CONTENT => {
                Ok(response.json::<types::DeleteListResponse>().await?)
            }
            StatusCode::NOT_FOUND => Err(errors::BriteVerifyClientError::BulkListNotFound(
                Box::new(types::BulkListCRUDError {
                    list_id: Some(list_id),
                    ..response.json::<types::BulkListCRUDError>().await?
                }),
            )),
            _ => Err(errors::BriteVerifyClientError::UnusableResponse(Box::new(
                response,
            ))),
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn terminate_list_by_id<ListId: ToString + Debug>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse, errors::BriteVerifyClientError> {
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn queue_list_for_processing<ListId: ToString + Debug>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse, errors::BriteVerifyClientError> {
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
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub async fn get_results_by_list_id<ListId: ToString + Debug>(
        &self,
        list_id: ListId,
    ) -> Result<Vec<types::BulkVerificationResult>, errors::BriteVerifyClientError> {
        let list_id = list_id.to_string();
        let list_status = self.get_list_by_id(&list_id).await?;

        if list_status.page_count.is_none() {
            return Err(errors::BriteVerifyClientError::Other(anyhow::Error::msg(
                "Missing page count!",
            )));
        }

        let page_count = std::cmp::max(1u64, list_status.page_count.unwrap());

        let pages: Vec<_> = futures_util::future::join_all(
            (1..=page_count).map(|page_number| self._get_result_page(list_id.clone(), page_number)),
        )
        .await
        .into_iter()
        .filter(|page| {
            if let Err(error) = page {
                log::error!("{error:#?}");
                false
            } else {
                true
            }
        })
        .map(|task_result| task_result.unwrap().results)
        .collect();

        let results: Vec<types::BulkVerificationResult> = itertools::concat(pages);

        Ok(results)
    }

    // </editor-fold desc="// Bulk Verification (v3) Endpoints ... ">
}

// </editor-fold desc="// Client ...">

// <editor-fold desc="// Test Utilities ...">

#[cfg(any(test, tarpaulin, feature = "ci"))]
impl BriteVerifyClientBuilder {
    #[doc(hidden)]
    /// Get the current `v1_base_url` value
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn v1_url_mut(&mut self) -> &mut url::Url {
        &mut self.v1_base_url
    }

    #[doc(hidden)]
    /// Get the current `v3_base_url` value
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn v3_url_mut(&mut self) -> &mut url::Url {
        &mut self.v3_base_url
    }

    #[doc(hidden)]
    /// Get host portion of the current `v1_base_url` value
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn v1_url_host(&self) -> String {
        self.v1_base_url.host_str().unwrap().to_string()
    }

    #[doc(hidden)]
    /// Get host portion of the current `v3_base_url` value
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn v3_url_host(&self) -> String {
        self.v3_base_url.host_str().unwrap().to_string()
    }

    #[doc(hidden)]
    /// Set the target port of the current `v1_base_url`
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn set_v1_url_port(mut self, port: u16) -> Self {
        self.v1_url_mut().set_port(Some(port)).unwrap_or(());
        self
    }

    #[doc(hidden)]
    /// Set the target port of the current `v3_base_url`
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn set_v3_url_port(mut self, port: u16) -> Self {
        self.v3_url_mut().set_port(Some(port)).unwrap_or(());
        self
    }

    #[doc(hidden)]
    /// Set the scheme of the current `v1_base_url`
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn set_v1_url_scheme(mut self, scheme: http::uri::Scheme) -> Self {
        self.v1_url_mut().set_scheme(scheme.as_str()).unwrap_or(());
        self
    }

    #[doc(hidden)]
    /// Set the scheme of the current `v3_base_url`
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn set_v3_url_scheme(mut self, scheme: http::uri::Scheme) -> Self {
        self.v3_url_mut().set_scheme(scheme.as_str()).unwrap_or(());
        self
    }

    #[doc(hidden)]
    /// Force DNS resolution for the current `v1_base_url` to the IP address
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn resolve_v1_url_to(mut self, address: SocketAddr) -> Self {
        let v1_host = self.v1_url_host();

        self.builder = self.builder.resolve(&v1_host, address);

        self
    }

    #[doc(hidden)]
    /// Force DNS resolution for the current `v3_base_url` to the IP address
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn resolve_v3_url_to(mut self, address: SocketAddr) -> Self {
        let v3_host = self.v3_url_host();

        self.builder = self.builder.resolve(&v3_host, address);

        self
    }
}

#[cfg(any(test, tarpaulin, feature = "ci"))]
impl BriteVerifyClient {
    #[doc(hidden)]
    /// Use a client instance's [`_build_and_send`](BriteVerifyClient::_build_and_send)
    /// method directly in a unit or end-to-end test
    #[cfg_attr(tarpaulin, coverage(off))]
    pub async fn build_and_send(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, errors::BriteVerifyClientError> {
        self._build_and_send(request).await
    }
}

// </editor-fold desc="// Test Utilities ...">

// <editor-fold desc="// I/O-Free Tests ...">

#[cfg(test)]
mod tests {
    // Third-Party Dependencies
    use anyhow::Result;
    use http::uri::Scheme;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    // Crate-Level Imports
    use super::{
        BriteVerifyClient, BriteVerifyClientBuilder, HeaderMap, HeaderValue, AUTHORIZATION,
    };

    // <editor-fold desc="// Constants ...">

    const GOOD_KEY: &str = "I guess it's better to be lucky than good";
    const BAD_KEY: &str =
        "\r\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\r";

    // </editor-fold desc="// Constants ...">

    // <editor-fold desc="// Tests ...">

    #[rstest::rstest]
    /// Test that the `BriteVerifyClientBuilder`'s `new`
    /// method properly creates the expected client instance
    fn test_new_bv_client() {
        let client = BriteVerifyClient::new(BAD_KEY);

        assert!(
            client.as_ref().is_err_and(|error| matches!(
                error,
                super::errors::BriteVerifyClientError::InvalidHeaderValue(_)
            )),
            "Expected Err(BriteVerifyClientError::InvalidHeaderValue), got: {:#?}",
            client
        );

        let client = BriteVerifyClient::new(GOOD_KEY);

        assert!(
            client.as_ref().is_ok_and(|instance| {
                let client_repr = format!("{:?}", instance);
                !client_repr.contains(GOOD_KEY)
            }),
            "Expected the configured API key to be obfuscated, but got: {:#?}",
            client
        );
    }

    //noinspection HttpUrlsUsage
    #[rstest::rstest]
    /// Test that the `BriteVerifyClientBuilder`'s `http_only`
    /// method properly mutates the appropriate inner `v?_base_url`
    /// fields and passes the expected values through to constructed
    /// client instances
    fn test_builder_http_only() {
        // Create a new builder instance with no base url(s) set
        let builder = BriteVerifyClient::builder();

        // Regardless of the value of the supplied flag, if the value
        // of `v?_base_url` is currently `None`, the default base url(s)
        // will be implicitly configured as the base url value(s).
        //
        // However, because toggling `http_only` to `false` only *enables*
        // HTTP-scheme urls (as opposed to explicitly *disabling* HTTPS ones),
        // no mutation of the existing url scheme will be performed. Therefore
        // when starting with a value of `None` for either base url and toggling
        // `http_only` to `false`, the url scheme should always remain HTTPS
        let builder = builder.https_only(false);

        // when none -> base url w/ https scheme
        assert_str_eq!(builder.v1_base_url.scheme(), Scheme::HTTPS.as_str());
        assert_str_eq!(builder.v3_base_url.scheme(), Scheme::HTTPS.as_str());

        let (v1_url, v3_url) = (builder.v1_base_url.clone(), builder.v3_base_url.clone());

        // Toggling `https_only` to `true` should not change the configured URL
        // but regardless of their previous scheme, they should now both be HTTPS
        // (or *still* HTTPS, in this case)
        let builder = builder.https_only(true);

        // when some && enabled -> self.v?_base_url w/ https scheme
        assert_eq!(&builder.v1_base_url, &v1_url);
        assert_eq!(&builder.v3_base_url, &v3_url);

        // There's no restriction against setting HTTP-scheme
        // values as base urls when `http_only` is `true` built
        // into the `Builder` itself, but doing so would cause
        // the constructed reqwest client to throw an error the
        // first time it's instructed to make a request, so the
        // lack of tight coupling between `BriteVerifyClientBuilder`
        // and `reqwest::ClientBuilder` is acceptable in this
        // specific regard.
        //
        // tl;dr we're gonna set the base urls to HTTP-scheme
        // values before we toggle `https_only` again so we can
        // verify that *disabling* HTTPS doesn't mutate the currently
        // configured base urls unexpectedly.'
        let builder = builder
            .v1_base_url("http://example.com/api/v1")
            .v3_base_url("http://example.com/api/v3");

        assert_str_eq!(builder.v1_base_url.scheme(), Scheme::HTTP.as_str());
        assert_str_eq!(builder.v3_base_url.scheme(), Scheme::HTTP.as_str());

        let (v1_url, v3_url) = (builder.v1_base_url.clone(), builder.v3_base_url.clone());

        // when some && disabled -> self.v?_base_url w/ untouched scheme
        let builder = builder.https_only(false);

        assert_eq!(&builder.v1_base_url, &v1_url);
        assert_eq!(&builder.v3_base_url, &v3_url);

        // Finally, when the currently configured base urls are
        // non-`None` http-scheme values, toggling `https_only`
        // should mutate *ONLY* the scheme of the configured urls
        // to be HTTPS.

        // when some && enabled -> self.v?_base_url w/ https scheme
        let builder = builder.https_only(true);

        assert_str_eq!(builder.v1_base_url.scheme(), Scheme::HTTPS.as_str());
        assert_str_eq!(builder.v3_base_url.scheme(), Scheme::HTTPS.as_str());

        assert_str_eq!(
            v1_url
                .as_str()
                .strip_prefix("http://")
                .map_or(v1_url.to_string(), |url| url.to_string()),
            builder
                .v1_base_url
                .as_str()
                .strip_prefix("https://")
                .map_or(builder.v1_base_url.as_str().to_string(), |url| {
                    url.to_string()
                })
        );
        assert_str_eq!(
            v3_url
                .as_str()
                .strip_prefix("http://")
                .map_or(v3_url.to_string(), |url| url.to_string()),
            builder
                .v3_base_url
                .as_str()
                .strip_prefix("https://")
                .map_or(builder.v3_base_url.as_str().to_string(), |url| {
                    url.to_string()
                })
        );
    }

    #[rstest::rstest]
    /// Test that the `BriteVerifyClientBuilder`'s `api_key` method
    /// sets/clears the inner `api_key` and `error` fields as expected
    fn test_builder_api_key_handling() {
        // ⇣ this *should* cause an `InvalidHeaderValue` error to be set
        let builder = BriteVerifyClient::builder().api_key(BAD_KEY);

        // when error -> clears api_key, sets error
        assert!(builder.error.is_some());
        assert!(builder.api_key.is_none());

        let builder = builder.api_key(GOOD_KEY);

        // when ok -> sets api_key, clears error (if set)
        assert!(builder.error.is_none());
        assert!(builder
            .api_key
            .as_ref()
            .is_some_and(|value| value.is_sensitive()));

        assert!(builder.build().is_ok());

        assert!(BriteVerifyClient::builder()
            .api_key(BAD_KEY)
            .build()
            .is_err())
    }

    #[rstest::rstest]
    /// Test that the `BriteVerifyClientBuilder`'s `v?_base_url`
    /// methods properly set/clear the appropriate inner `v?_base_url`
    /// and `error` fields and pass the expected values through
    /// to their constructed client instances
    fn test_builder_base_url_handling() {
        let builder = BriteVerifyClient::builder()
            .v1_base_url("https://testing.example.com:443")
            .v3_base_url("https://testing.example.com:443");

        assert!(builder.error.is_none());
        assert_ne!(super::V1_API_BASE_URL, builder.v1_base_url.as_str());
        assert_ne!(super::V3_API_BASE_URL, builder.v3_base_url.as_str());

        let (v1_url, v3_url) = (builder.v1_base_url.clone(), builder.v3_base_url.clone());

        let builder = builder.v1_base_url("").v3_base_url("");

        // The base URLs should be unchanged, but an error should now
        // be set, and it *should* specifically be an `InvalidBaseUrl`
        assert_str_eq!(v1_url.as_str(), builder.v1_base_url.as_str());
        assert_str_eq!(v3_url.as_str(), builder.v3_base_url.as_str());
        assert!(
            builder.error.as_ref().is_some_and(|error| matches!(
                error,
                super::errors::BriteVerifyClientError::InvalidBaseUrl(_)
            )),
            "Expected an `InvalidBaseUrl` error, got: {:?}",
            builder.error.as_ref()
        );
    }

    #[rstest::rstest]
    /// Test the `BriteVerifyClient`'s
    /// `From<reqwest::Client>` implementation
    fn test_bv_client_from_reqwest_client() -> Result<()> {
        let client = BriteVerifyClient::try_from(reqwest::Client::new());

        assert!(
            client.as_ref().is_err_and(|error| matches!(
                error,
                super::errors::BriteVerifyClientError::MissingApiKey
            )),
            "Expected Err(BriteVerifyClientError::MissingApiKey), got: {:#?}",
            client,
        );

        let headers = HeaderMap::from_iter(
            [(
                AUTHORIZATION,
                HeaderValue::from_str("ApiKey: well, that's certainly good to know")?,
            )]
            .into_iter(),
        );
        let client = BriteVerifyClient::try_from(
            reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        );

        Ok(assert!(
            client.as_ref().is_ok(),
            "Expected Ok(BriteVerifyClient), got: {:#?}",
            client
        ))
    }

    #[rstest::rstest]
    /// Test the `BriteVerifyClientBuilder`'s
    /// `From<reqwest::ClientBuilder>` implementation
    fn test_bv_builder_from_reqwest_builder() -> Result<()> {
        let req_builder = BriteVerifyClientBuilder::from(reqwest::Client::builder());

        assert!(req_builder.api_key.is_none());
        assert!(req_builder.build().is_err());

        let req_builder = BriteVerifyClientBuilder::from(
            reqwest::Client::builder().default_headers(HeaderMap::from_iter(
                [(
                    AUTHORIZATION,
                    HeaderValue::from_str("ApiKey: fate protects little children and fools")?,
                )]
                .into_iter(),
            )),
        );

        assert!(req_builder
            .api_key
            .as_ref()
            .is_some_and(|val| !val.is_sensitive()));
        Ok(assert!(req_builder.build().is_ok()))
    }

    // </editor-fold desc="// Tests ...">
}

// </editor-fold desc="// I/O-Free Tests ...">
