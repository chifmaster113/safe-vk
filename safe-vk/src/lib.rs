// Inspired by https://github.com/tokio-rs/axum/tree/main/axum/src/routing
//! A simple library for creating your own VK bot
//!
//! # Abstraction
//! SafeVk is using same approach similar to web servers such as
//! [`axum`](https://crates.io/crates/axum), where you define
//! routes to handle incoming requests. Similarly, SafeVk lets you set up routes to handle
//! updates from VK API.
//!
//! Take a look:
//! ```ignore
//! use safe_vk::{extract::Ctx, responses::Message, Filter, Result, SafeVk};
//!
//! // Route
//! async fn vkbot(update: Ctx<Message>) -> Result<()> {
//!     update.messages().send().random_id(0).message("it works!").await?;
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let token = "my super secret token";
//!     // Add as many routes as you want
//!     let bot = SafeVk::new().command("/safevk", vkbot, Filter::Strict);
//!     // Starting to listen for updates
//!     safe_vk::start_polling(token, bot).await.unwrap();
//! }
//! ```
//! You can simplify it even further by enabling the `macros` feature.
//! ```rust
//! use safe_vk::{extract::Ctx, responses::Message, Filter, auto_ok, SafeVk};
//!
//! #[auto_ok]
//! async fn vkbot(update: Ctx<Message>) {
//!     update.messages().send().random_id(0).message("it works!").await?;
//! }
//! ```
//!
//! SafeVk also provides [`filters`](crate::Filter) to adjust the strictness of command matching,
//! allowing for both precise and flexible command handling based on your needs.
//!
//! # Compatibility
//!
//! SafeVk is using [tokio] and [reqwest] under the hood, and works with
//! [5.199](https://dev.vk.com/en/reference/version/5.199) API version.
//!
#![warn(
    clippy::all,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::if_let_mutex,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::match_wildcard_for_single_variants,
    clippy::linkedlist,
    clippy::macro_use_imports,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style
)]
#![deny(unreachable_pub)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

mod reqwest_ext;

#[macro_use]
pub(crate) mod macros;

pub mod api;
pub mod extract;
pub mod handler;
pub mod responses;
pub mod routing;
pub mod service;
#[cfg(feature = "tokio")]
pub mod start_polling;
pub use safe_vk_common::*;

pub use self::reqwest_ext::{RequestBuilder, VERSION, VK, WAIT_TIME};
pub use self::routing::SafeVk;

//#[cfg(feature = "macros")]
pub use safe_vk_macros::*;

#[cfg(feature = "tokio")]
pub use self::start_polling::start_polling;

#[cfg(feature = "tokio")]
pub type Response<T> = Result<T>;

#[macro_export]
macro_rules! parse_response {
    ($value:expr, Option<$type:ty>) => {{
        use serde::de::Error;
        let object: Result<Option<$type>, serde_json::Error> = match $value.get("response") {
            Some(response) => match <$type as serde::Deserialize>::deserialize(response) {
                Ok(parsed) => Ok(Some(parsed)),
                Err(_) => {
                    if response.is_number() || response.is_string() {
                        Ok(None)
                    } else {
                        Err(serde_json::Error::custom("Unexpected response format"))
                    }
                }
            },
            None => Ok(None), // No "response" key, return None
        };
        object.map_err(|e| serde_json::Error::custom(e))
    }};
    // Case for regular type (not Option<T>)
    ($value:expr, $type:ty) => {{
        use serde::de::Error;
        let response_value = match $value.get("response") {
            Some(response) => <$type as serde::Deserialize>::deserialize(response),
            None => serde_json::from_value($value),
        };
        response_value.map_err(|e| serde_json::Error::custom(e))
    }};
}
