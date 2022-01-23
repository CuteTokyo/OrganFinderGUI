
use std::io;
use rustc_serialize::Decodable;
use rustc_serialize::json;
use hyper::client::IntoUrl;
use std::io::Read;
use libcoinche::pos;
use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use url;
use hyper;

use {NewPartyInfo, Event, EventType, ContractBody, CardBody};

use super::Backend;

/// HTTP coinched client.
///
/// Provides an abstraction over HTTP requests.
pub struct HttpBackend {
    player_id: u32,
    pub pos: pos::PlayerPos,

    event_id: usize,

    host: String, /* It used to include a re-usable hyper::Client,
                   * but it would lead to failed request if too
                   * long happened between two queries. */
}

#[derive(Debug)]
pub enum Error {
    Url(url::ParseError),
    Hyper(hyper::Error),
    Json(json::DecoderError),
    Coinched(::Error),
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Url(err)
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Hyper(err)
    }
}

impl From<json::ParserError> for Error {
    fn from(err: json::ParserError) -> Self {
        Error::Json(json::DecoderError::ParseError(err))
    }
}

impl From<json::DecoderError> for Error {
    fn from(err: json::DecoderError) -> Self {
        Error::Json(err)
    }
}

/// Helper method to decode a `T: Decodable` from a reader.
///
/// (`json::decode` only works from a string)
fn from_reader<R: Read, T: Decodable>(r: &mut R) -> Result<T, Error> {
    let json = try!(json::Json::from_reader(r));

    let has_error = match json.as_object() {
        Some(obj) => obj.contains_key("error"),