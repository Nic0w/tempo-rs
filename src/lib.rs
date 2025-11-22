#![warn(missing_docs)]
//! This crate is a Rust client library allowing to access RTE's API for EDF's "Tempo" contract option.
//!
//! [EDF][1] is a French state-owned electricity utility company in charge of selling energy (electricity) to homes and business.
//! [RTE][2] is a subsidiary of EDF in charge of the operation, maintenance and development of the French high-voltage transmission system.
//!
//! EDF's "Tarif Bleu" contract comes with an option called "Tempo", with which electricity price fluctuates depending on days and time of day.
//! There are three types of day:
//!  - Blue : cheapest prices
//!  - White : average prices
//!  - Red : prices can be five times higher than average
//!
//! Each day is split in two periods:
//!  - Peak hours: from 6AM to 10PM
//!  - Off-peak hours: from 10PM to 6AM
//!
//! For example, at the time of writing :
//!  - during a Blue day, off-peak hour price is 12.96 ct€/kWh
//!  - during a White day, off-peak hour price is 14.86 ct€/kWh
//!  - during a Red day, peak hour price is 75.62 ct€/kWh
//!
//! See [the official prices table][3] for all prices.
//!
//! Given the high differences between periods and days, it can be interesting to have advance knowledge about future days' color.
//! As a matter of fact, RTE is contractually bound to publish next-day color at 10:30AM each day.
//!
//! It also publishes a REST API allowing to programmaticaly request:
//!  - historical data (past days' color)
//!  - next day color
//!
//! This API is authenticated through OAuth2. To obtain a client id and client secret, you need to create an account at RTE's [Data Portal][4] and subscribe to the ["Tempo-like supply contract" API][5].
//!
//! For an example of how to use this crate, see [../bin/tempo.rs].
//!
//! [1]: https://en.wikipedia.org/wiki/%C3%89lectricit%C3%A9_de_France
//! [2]: https://en.wikipedia.org/wiki/R%C3%A9seau_de_Transport_d%27%C3%89lectricit%C3%A9
//! [3]: https://particulier.edf.fr/content/dam/2-Actifs/Documents/Offres/Grille_prix_Tarif_Bleu.pdf
//! [4]: https://data.rte-france.com/
//! [5]: https://data.rte-france.com/catalog/-/api/consumption/Tempo-Like-Supply-Contract/v1.1

use std::{fs, path::Path};

use base64::{prelude::BASE64_STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use reqwest::{
    header::{self, HeaderValue, ACCEPT},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

mod model;

pub use model::{CalendarValue, TempoCalendars, TempoColor};

//const RTE_API_DOMAIN: &str = "digital.iservices.rte-france.com";

const RTE_API_AUTH_URL: &str = "https://digital.iservices.rte-france.com/token/oauth/";
//const RTE_API_PATH: &str =
//    "https://digital.iservices.rte-france.com/open_api/tempo_like_supply_contract/v1";

const RTE_API_TEMPO_CALENDARS: &str =
    "https://digital.iservices.rte-france.com/open_api/tempo_like_supply_contract/v1/tempo_like_calendars";

/// Something went wrong while using the API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Problem with the OAuth2 flow.
    #[error(transparent)]
    OAuth2(
        #[from]
        oauth2::RequestTokenError<
            oauth2::HttpClientError<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),

    /// Something went wrong doing an HTTP request.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// Bad JSON was supplied.
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Remote server returned an error.
    /// Description and code are described in the API's official documentation.
    #[error("bad request - {description} ({code}) ")]
    BadRequest {
        /// Error's description
        description: String,

        /// Error code
        code: String,
    },

    /// There was a problem while using the user provided credentials file for OAuth2.
    #[error(transparent)]
    BadCredendials(#[from] BadCreds),
}

type OAuth2TokenResponse =
    oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>;

struct TokenState {
    response: OAuth2TokenResponse,
    expiry: Option<(DateTime<Utc>, u64)>,
}

/// Main object for interacting with the API.
///
/// ```no_run
/// use tempo_rs::Tempo;
///
/// # async fn example() {
/// let tempo: Tempo = tempo_rs::authorize_with_file("./credentials.secret").await.unwrap();
/// let next_day = tempo.next_day().await.unwrap();
///
/// println!("{:?}", next_day);
/// # }
/// ```
pub struct Tempo {
    state: Mutex<TokenState>,

    oauth2_client: oauth2::Client<
        oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
        oauth2::StandardTokenIntrospectionResponse<
            oauth2::EmptyExtraTokenFields,
            oauth2::basic::BasicTokenType,
        >,
        oauth2::StandardRevocableToken,
        oauth2::StandardErrorResponse<oauth2::RevocationErrorResponseType>,
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
    http_client: reqwest::Client,
}

/// There was a problem while using the user provided credentials file for OAuth2.
#[derive(Debug, Error)]
pub enum BadCreds {
    /// File cannot be read or accessed (OS error).
    #[error(transparent)]
    File(#[from] std::io::Error),

    /// There was a problem decoding base64.
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),

    /// The decoded base64 string contains invalid UTF-8 characters.
    #[error(transparent)]
    Utf8(#[from] core::str::Utf8Error),

    /// The decoded string's format is invalid. Valid format is `client_id:client_secret`.
    #[error("Failed to split client id from secret. Where is the colon ?")]
    Format,
}

/// Given a file containing a client id and client secret, get authorization through OAuth2 from the server.
/// It is assumed the file is the one given by RTE.
pub async fn authorize_with_file<P: AsRef<Path>>(path: P) -> Result<Tempo, ApiError> {
    let raw_content = fs::read_to_string(path).map_err(BadCreds::File)?;

    let decoded = BASE64_STANDARD
        .decode(raw_content)
        .map_err(BadCreds::Base64)?;

    let as_string = core::str::from_utf8(&decoded).map_err(BadCreds::Utf8)?;

    let parts = as_string.split_once(':').ok_or(BadCreds::Format)?;

    authorize(parts.0.to_owned(), parts.1.to_owned()).await
}

/// Directly supply a client id and a client secret to get authorization through OAuth2 from the server.
pub async fn authorize(client_id: String, client_secret: String) -> Result<Tempo, ApiError> {
    let client_id = ClientId::new(client_id);
    let client_secret = ClientSecret::new(client_secret);

    let auth_url = AuthUrl::new(RTE_API_AUTH_URL.to_owned()).unwrap();
    let token_url = TokenUrl::new(RTE_API_AUTH_URL.to_owned()).unwrap();

    let oauth2_client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url);

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(ApiError::Reqwest)?;

    let token_response = oauth2_client
        .exchange_client_credentials()
        .add_scope(Scope::new("tempo_like_supply_contract".to_string()))
        .request_async(&http_client)
        .await?;

    let now: DateTime<Utc> = Utc::now();

    let expiry = token_response
        .expires_in()
        .map(|duration| (now + duration, duration.as_secs()));

    let state = Mutex::new(TokenState {
        response: token_response,
        expiry,
    });

    Ok(Tempo {
        state,
        oauth2_client,
        http_client,
    })
}

fn parse_www_authenticate(value: &HeaderValue) -> Option<(&str, &str)> {
    let parts  = value.to_str()
        .inspect_err(|e| log::warn!("Got 401 Unauthorized from server but WWW-Authenticate header is not valid UTF-8 ({})",e))
        .ok()
        .map(|a| a.split(';'));

    let mut error = None;
    let mut error_description = None;

    if let Some(parts) = parts {
        for part in parts {
            match part.split_once('=') {
                Some(("error", value)) => error = Some(value),
                Some(("error_description", value)) => error_description = Some(value),
                _ => {}
            }
        }
    }

    error.zip(error_description)
}

impl Tempo {
    async fn get_oauth_token(&self) -> Result<String, ApiError> {
        let mut state = self.state.lock().await;

        if let Some((expiry, _duration)) = state.expiry {
            let now: DateTime<Utc> = Utc::now();

            let delta = expiry.signed_duration_since(now).num_seconds();

            log::debug!(target: "tempo-rs::get_oauth_token", 
                "Time is {} and token expires in {} seconds", now, delta);

            if delta.is_positive() {
                //token hasn't expired yet
                return Ok(state.response.access_token().secret().clone());
            }

            let new_token_response = self
                .oauth2_client
                .exchange_client_credentials()
                .request_async(&self.http_client)
                .await?;

            log::debug!(target: "tempo-rs::get_oauth_token", 
                "Successfully renewed token");

            if let Some(new_expiry) = new_token_response.expires_in() {
                state.expiry = Some((now + new_expiry, new_expiry.as_secs()));
            }

            state.response = new_token_response;

            Ok(state.response.access_token().secret().clone())
        } else {
            Ok(state.response.access_token().secret().clone())
        }
    }

    async fn authenticated_call<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        method: Method,
        url: &str,
        query: &T,
    ) -> Result<R, ApiError> {
        let bearer_token = self.get_oauth_token().await?;

        let req_builder = self
            .http_client
            .request(method, url)
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .bearer_auth(bearer_token)
            .query(query);

        let req = req_builder.build()?;

        log::debug!(target: "tempo-rs::authenticated_call", "Request: {:?}", req);

        let resp = self.http_client.execute(req).await?;

        let headers = resp.headers();
        let status = resp.status();

        log::debug!(target: "tempo-rs::authenticated_call", "Response status: {}", status);

        match status {
            StatusCode::UNAUTHORIZED => {
                if let Some((error, error_desc)) = headers
                    .get(header::WWW_AUTHENTICATE)
                    .and_then(parse_www_authenticate)
                {
                    log::error!(target: "tempo-rs::authenticated_call", "Server returned 401: {} - {}", error, error_desc);

                    Err(ApiError::BadRequest {
                        description: error_desc.into(),
                        code: error.into(),
                    })
                } else {
                    let body: String = resp.text().await?;
                    log::error!(target: "tempo-rs::authenticated_call", "Server returned 401, logging response body:\n{}", body);

                    Err(ApiError::BadRequest {
                        description: body,
                        code: String::default(),
                    })
                }
            }

            status if status.is_client_error() || status.is_server_error() => {
                let body: String = resp.text().await?;
                let error: model::Error = serde_json::from_str(&body)?;

                Err(ApiError::BadRequest {
                    description: error.error_description,
                    code: error.error,
                })
            }

            //assume success ?
            status if status.is_success() => {
                let body: String = resp.text().await?;
                log::trace!(target: "tempo-rs::authenticated_call", "{}", body);

                let json = serde_json::from_str(&body)?;

                Ok(json)
            }

            unhandled_status => {
                log::warn!(target: "tempo-rs::authenticated_call", "Got response with unhandled status: {}", unhandled_status);
                let body: String = resp.text().await?;
                log::warn!(target: "tempo-rs::authenticated_call", "Unhandled status - body:\n{}", body);

                unimplemented!()
            }
        }
    }

    /// Used for requesting historical data.
    /// Even though `start_date` and `end_date` are UTC based, official documentation mentions that dates can be supplied from any timezone.
    /// Thus it is not clear what is the effect of the time component.
    ///
    /// The two valid ways to call this function are:
    ///  - `start_date` and `end_date` both containing `Some` date/time. In this case, historical data is returned for the period.
    ///  - `None` of `start_date` and `end_date` contain a date/time. In this case, next-day data is returned. See [`Self::next_day()`].
    ///
    /// Official documentation **does not recommand** to request more than 366 days at a time.
    /// Earliest possible date is 09/01/2014.
    /// It is not clear what `fallback` is or what it is used for. Official doc refers to a *degraded mode*.
    pub async fn calendars(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        fallback: Option<bool>,
    ) -> Result<TempoCalendars, ApiError> {
        let mut query = vec![];

        if let Some(start_date) = start_date {
            query.push(("start_date", start_date.format("%FT%T%:z").to_string()))
        }

        if let Some(end_date) = end_date {
            query.push(("end_date", end_date.format("%FT%T%:z").to_string()))
        }

        if let Some(fallback) = fallback {
            query.push(("fallback_status", fallback.to_string()))
        }

        self.authenticated_call(Method::GET, RTE_API_TEMPO_CALENDARS, query.as_slice())
            .await
    }

    /// To request next-day color.
    /// Basically a short-hand for [`Self::calendars()`] with all parameters set to `None`
    pub async fn next_day(&self) -> Result<TempoCalendars, ApiError> {
        self.calendars(None, None, None).await
    }
}
