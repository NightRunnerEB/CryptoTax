use chrono::NaiveDate;
use reqwest::Error;
use serde::de::DeserializeOwned;

use crate::{
    params::{DerivativesIncludeTickers, TickersOrder},
    response::{
        coins::History,
        derivatives::{DerivativeExchangeData, DerivativeExchangeId},
        exchanges::ExchangeTickers,
        ping::SimplePing,
    },
};

/// CoinGecko API URL
pub const COINGECKO_API_DEMO_URL: &str = "https://api.coingecko.com/api/v3";
pub const COINGECKO_API_PRO_URL: &str = "https://pro-api.coingecko.com/api/v3";
/// CoinGecko API Header
pub const COINGECKO_API_DEMO_HEADER: &str = "x-cg-demo-api-key";
pub const COINGECKO_API_PRO_HEADER: &str = "x-cg-pro-api-key";

/// CoinGecko client
pub struct CoinGeckoClient {
    host: &'static str,
    client: reqwest::Client,
    api_key: Option<String>,
    api_key_header: Option<&'static str>,
}

/// Creates a new CoinGeckoClient with host https://api.coingecko.com/api/v3
///
/// # Examples
///
/// ```rust
/// use coingecko::CoinGeckoClient;
/// let client = CoinGeckoClient::default();
/// ```
impl Default for CoinGeckoClient {
    fn default() -> Self {
        std::env::var("COINGECKO_PRO_API_KEY")
            .map(|k| CoinGeckoClient::new_with_pro_api_key(&k))
            .or_else(|_| {
                std::env::var("COINGECKO_DEMO_API_KEY")
                    .map(|k| CoinGeckoClient::new_with_demo_api_key(&k))
            })
            .unwrap_or_else(|_| CoinGeckoClient::new(COINGECKO_API_DEMO_URL))
    }
}

impl CoinGeckoClient {
    /// Creates a new CoinGeckoClient client with a custom host url
    ///
    /// # Examples
    ///
    /// ```rust
    /// use coingecko::CoinGeckoClient;
    /// let client = CoinGeckoClient::new("https://some.url");
    /// ```
    pub fn new(host: &'static str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .gzip(true)
            .build()
            .expect("failed to build reqwest client");
        CoinGeckoClient {
            host,
            client,
            api_key: None,
            api_key_header: None,
        }
    }

    /// Creates a new CoinGeckoClient client with demo api key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use coingecko::CoinGeckoClient;
    /// let client = CoinGeckoClient::new_with_demo_api_key();
    /// ```
    pub fn new_with_demo_api_key(api_key: &str) -> Self {
        let mut c = Self::new(COINGECKO_API_DEMO_URL);
        c.api_key = Some(api_key.to_string());
        c.api_key_header = Some(COINGECKO_API_DEMO_HEADER);
        c
    }

    /// Creates a new CoinGeckoClient client with pro api key
    ///
    /// # Examples
    ///
    /// ```rust
    /// use coingecko::CoinGeckoClient;
    /// let client = CoinGeckoClient::new_with_pro_api_key();
    /// ```
    pub fn new_with_pro_api_key(api_key: &str) -> Self {
        let mut c = Self::new(COINGECKO_API_PRO_URL);
        c.api_key = Some(api_key.to_string());
        c.api_key_header = Some(COINGECKO_API_PRO_HEADER);
        c
    }

    /// Send a GET request to the given endpoint
    pub async fn get<R: DeserializeOwned>(&self, endpoint: &str) -> Result<R, Error> {
        let slash = if endpoint.starts_with('/') { "" } else { "/" };
        let url = format!("{}{}{}", self.host, slash, endpoint);

        let mut req = self.client.get(&url);
        if let (Some(h), Some(k)) = (self.api_key_header, &self.api_key) {
            req = req.header(h, k);
        }

        let res = req.send().await?;
        res.error_for_status()?.json().await
    }

    /// Check API server status
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     use coingecko::CoinGeckoClient;
    ///     let client = CoinGeckoClient::default();
    ///
    ///     client.ping().await;
    /// }
    /// ```
    pub async fn ping(&self) -> Result<SimplePing, Error> {
        self.get("/ping").await
    }

    /// Get historical data (name, price, market, stats) at a given date for a coin
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     use chrono::{NaiveDate, Datelike};
    ///     use coingecko::CoinGeckoClient;
    ///     let client = CoinGeckoClient::default();
    ///
    ///     let current_date = chrono::Utc::now();
    ///     let year = current_date.year();
    ///     client.coin_history("bitcoin", NaiveDate::from_ymd(year, 1, 1), true).await;
    /// }
    /// ```
    pub async fn coin_history(
        &self,
        id: &str,
        date: NaiveDate,
        localization: bool,
    ) -> Result<History, Error> {
        let formatted_date = date.format("%d-%m-%Y").to_string();

        let req = format!("/coins/{id}/history?date={formatted_date}&localization={localization}");
        self.get(&req).await
    }

    /// Get exchange tickers (paginated)
    ///
    /// **IMPORTANT**:
    /// Ticker is_stale is true when ticker that has not been updated/unchanged from the exchange for a while.
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     use coingecko::{params::TickersOrder, CoinGeckoClient};
    ///     let client = CoinGeckoClient::default();
    ///
    ///     client.exchange_tickers("binance", Some(&["btc"]), true, 1, TickersOrder::TrustScoreAsc, true).await;
    /// }
    /// ```
    pub async fn exchange_tickers<CoinId: AsRef<str>>(
        &self,
        exchange_id: &str,
        coin_ids: Option<&[CoinId]>,
        include_exchange_logo: bool,
        page: i64,
        order: TickersOrder,
        depth: bool,
    ) -> Result<ExchangeTickers, Error> {
        let order = match order {
            TickersOrder::TrustScoreAsc => "trust_score_asc",
            TickersOrder::TrustScoreDesc => "trust_score_desc",
            TickersOrder::BaseTarget => "base_target",
        };

        let req = match coin_ids {
            Some(c_ids) => {
                let c_ids = c_ids.iter().map(AsRef::as_ref).collect::<Vec<_>>();
                format!(
                    "/exchanges/{exchange_id}/tickers?coin_ids={}&include_exchange_logo={include_exchange_logo}&page={page}&order={order}&depth={depth}",
                    c_ids.join("%2C")
                )
            }
            None => format!(
                "/exchanges/{exchange_id}/tickers?include_exchange_logo={include_exchange_logo}&page={page}&order={order}&depth={depth}"
            ),
        };

        self.get(&req).await
    }

    /// Show derivative exchange data
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     use coingecko::{params::DerivativesIncludeTickers, CoinGeckoClient};
    ///     let client = CoinGeckoClient::default();
    ///
    ///     client.derivatives_exchange("bitmex", Some(DerivativesIncludeTickers::All)).await;
    /// }
    /// ```
    pub async fn derivatives_exchange(
        &self,
        id: &str,
        include_tickers: Option<DerivativesIncludeTickers>,
    ) -> Result<DerivativeExchangeData, Error> {
        let include_tickers = match include_tickers {
            Some(ic_enum) => match ic_enum {
                DerivativesIncludeTickers::All => "all",
                DerivativesIncludeTickers::Unexpired => "unexpired",
            },
            None => "unexpired",
        };

        let req = format!("/derivatives/exchanges/{id}?include_tickers={include_tickers}");
        self.get(&req).await
    }

    /// List all derivative exchanges name and identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     use coingecko::CoinGeckoClient;
    ///     let client = CoinGeckoClient::default();
    ///
    ///     client.derivative_exchanges_list().await;
    /// }
    /// ```
    pub async fn derivative_exchanges_list(&self) -> Result<Vec<DerivativeExchangeId>, Error> {
        self.get("/derivatives/exchanges/list").await
    }
}
