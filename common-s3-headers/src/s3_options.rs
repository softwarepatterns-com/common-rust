//! There are too many possible optional params to AWS functions, so we organize them into a struct.
//!
//! Will never have ownership of any of the data it references.
//!
//!
use url::Url;

const EMPTY_PAYLOAD_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Debug, Default, Clone, Copy)]
pub enum S3DateTime {
  #[default]
  Now,
  UnixTimestamp(i64),
}

#[derive(Debug, Clone)]
pub struct S3Options<'a> {
  pub datetime: S3DateTime,
  pub access_key: &'a str,
  pub secret_key: &'a str,
  pub region: &'a str,
  pub service: &'a str,
  pub url: &'a Url,
  pub method: &'a str,
  pub headers: &'a [(&'static str, std::string::String)],
  pub payload_hash: &'a str,
}

impl<'a> S3Options<'a> {
  pub fn new(url: &'a Url) -> Self {
    Self {
      datetime: Default::default(),
      access_key: Default::default(),
      secret_key: Default::default(),
      region: Default::default(),
      service: Default::default(),
      url,
      method: Default::default(),
      headers: Default::default(),
      payload_hash: EMPTY_PAYLOAD_SHA,
    }
  }

  pub fn set_access_key(mut self, value: &'a str) -> Self {
    self.access_key = value;
    self
  }

  pub fn set_secret_key(mut self, value: &'a str) -> Self {
    self.secret_key = value;
    self
  }

  pub fn set_region(mut self, value: &'a str) -> Self {
    self.region = value;
    self
  }
  pub fn set_datetime(mut self, value: S3DateTime) -> Self {
    self.datetime = value;
    self
  }

  pub fn set_payload_hash(mut self, value: &'a str) -> Self {
    self.payload_hash = value;
    self
  }

  pub fn set_method(mut self, value: &'a str) -> Self {
    self.method = value;
    self
  }

  pub fn set_service(mut self, value: &'a str) -> Self {
    self.service = value;
    self
  }

  pub fn set_url(mut self, url: &'a Url) -> Self {
    self.url = url;
    self
  }

  pub fn set_headers(mut self, headers: &'a [(&'static str, std::string::String)]) -> Self {
    self.headers = headers;
    self
  }

  pub fn get_offset_datetime(&self) -> time::OffsetDateTime {
    match self.datetime {
      S3DateTime::Now => time::OffsetDateTime::now_utc(),
      S3DateTime::UnixTimestamp(timestamp) => {
        time::OffsetDateTime::from_unix_timestamp(timestamp).expect("Always valid")
      }
    }
  }
}
