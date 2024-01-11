use crate::{aws_canonical, aws_format, aws_math};
use hmac::Mac;
use std::borrow::Cow;
use url::Url;

pub const EMPTY_PAYLOAD_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// Used to specify the datetime to use when building the headers. Defaults to
/// `S3DateTime::Now` which will use the current time when the headers are built.
///
/// Note: This is designed for future expansion or variations of plain timestamps.
#[derive(Debug, Default, Clone, Copy)]
pub enum S3DateTime {
  #[default]
  Now,
  UnixTimestamp(i64),
}

impl S3DateTime {
  pub fn get_offset_datetime(&self) -> time::OffsetDateTime {
    match self {
      S3DateTime::Now => time::OffsetDateTime::now_utc(),
      S3DateTime::UnixTimestamp(timestamp) => {
        time::OffsetDateTime::from_unix_timestamp(*timestamp).expect("Always valid")
      }
    }
  }
}

/// Builder for S3 headers. Main entry point for this crate. Used to build
/// the headers necessary to make a request to a AWS compatible service.
///
/// The returned headers are just strings and can be used with any HTTP client.
///
/// # Example
///
/// ```
/// use common_s3_headers::S3HeadersBuilder;
/// use url::Url;
///
/// let url = Url::parse("https://jsonlog.s3.amazonaws.com/test/test.json").unwrap();
/// let headers: Vec<(&str, String)> = S3HeadersBuilder::new(&url)
///  .set_access_key("access_key")
///  .set_secret_key("secret_key")
///  .set_region("us-east-1")
///  .set_method("GET")
///  .set_service("s3")
///  .build();
/// ```
#[derive(Debug, Clone)]
pub struct S3HeadersBuilder<'a> {
  pub datetime: S3DateTime,
  pub access_key: &'a str,
  pub secret_key: &'a str,
  pub region: &'a str,
  pub service: &'a str,
  pub url: &'a Url,
  pub method: &'a str,
  pub headers: &'a [(&'static str, std::string::String)],
  pub payload_hash: Cow<'a, str>,
  pub range: Option<(u64, Option<u64>)>,
}

impl<'a> S3HeadersBuilder<'a> {
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
      payload_hash: Cow::Borrowed(EMPTY_PAYLOAD_SHA),
      range: Default::default(),
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
    self.payload_hash = Cow::Borrowed(value);
    self
  }

  pub fn set_payload_hash_with_content(mut self, content: &[u8]) -> Self {
    let sha = aws_math::get_sha256(content);
    self.payload_hash = Cow::Owned(sha);
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

  pub fn set_range(mut self, start: u64, end: Option<u64>) -> Self {
    self.range = Some((start, end));
    self
  }

  pub fn set_headers(mut self, headers: &'a [(&'static str, std::string::String)]) -> Self {
    self.headers = headers;
    self
  }

  pub fn build(self) -> Vec<(&'static str, String)> {
    get_headers(self)
  }
}

/// Gets all the headers necessary to make a request to a AWS compatible service. Consumes the builder.
fn get_headers(options: S3HeadersBuilder) -> Vec<(&'static str, String)> {
  let url = options.url;
  let payload_hash = &options.payload_hash;
  let datetime = options.datetime.get_offset_datetime();
  let amz_date = aws_format::to_long_datetime(&datetime);

  let mut headers: Vec<(&'static str, String)> = [
    options.headers,
    &[
      ("Host", url.host_str().unwrap().to_owned()),
      ("x-amz-content-sha256", payload_hash.to_string()),
      ("x-amz-date", amz_date),
    ],
  ]
  .concat();

  if let Some((start, end)) = options.range {
    let range_headers = aws_canonical::get_range_headers(start, end);
    headers.extend(range_headers);
  }

  let auth_header = get_authorization_header(options.set_headers(&headers));

  headers.push(("Authorization", auth_header));
  headers
}

/// Only gets the authorirzation header. Consumes the builder.
fn get_authorization_header(options: S3HeadersBuilder) -> String {
  let datetime = options.datetime.get_offset_datetime();
  let region = options.region;
  let access_key = options.access_key;
  let secret_key = options.secret_key;
  let service = options.service;
  let url = options.url;
  let method = options.method;
  let payload_hash = options.payload_hash;
  let canonical_headers = aws_canonical::to_canonical_headers(options.headers);
  let canonical_request = aws_format::canonical_request_string(method, url, &canonical_headers, &payload_hash);
  let string_to_sign = aws_format::string_to_sign(&datetime, region, service, &canonical_request);
  let signing_key = aws_math::get_signature_key(&datetime, secret_key, region, service);
  let hmac: aws_math::HmacSha256 = aws_math::sign(&signing_key, string_to_sign.as_bytes());
  let signature = hex::encode(hmac.finalize().into_bytes());
  let signed_headers = aws_format::get_keys(&canonical_headers).join(";");

  aws_format::authorization_header_string(access_key, &datetime, region, service, &signed_headers, &signature)
}

#[cfg(test)]
mod tests {
  use super::*;
  use common_testing::assert;
  use std::str::FromStr;

  #[test]
  fn test_get_object() {
    let url = Url::from_str("https://jsonlog.s3.amazonaws.com/test/test.json").unwrap();
    let headers = S3HeadersBuilder::new(&url)
      .set_access_key("some_access_key")
      .set_secret_key("some_secret_key")
      .set_region("some_place")
      .set_datetime(S3DateTime::UnixTimestamp(0))
      .set_method("GET")
      .set_service("s3")
      .build();

    assert::equal(
      headers,
      vec![
        ("Host", "jsonlog.s3.amazonaws.com".to_owned()),
        (
          "x-amz-content-sha256",
          "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned(),
        ),
        ("x-amz-date", "19700101T000000Z".to_owned()),
        (
          "Authorization",
          "AWS4-HMAC-SHA256 Credential=some_access_key/19700101/some_place/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature=ac9a3c846f7368e934f31980d9df58d14cec3863a1a8be60bdeea708972b5a7b".to_owned(),
        ),
      ],
    )
  }

  #[test]
  fn test_get_object_2() {
    let url = Url::from_str("https://jsonlog.s3.amazonaws.com/test.json").unwrap();
    let headers = S3HeadersBuilder::new(&url)
      .set_access_key("some_access_key")
      .set_secret_key("some_secret_key")
      .set_region("some_place")
      .set_datetime(S3DateTime::UnixTimestamp(0))
      .set_method("GET")
      .set_service("s3")
      .build();

    assert::equal(headers, vec![
    ("Host", "jsonlog.s3.amazonaws.com".to_owned()),
    ("x-amz-content-sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned()),
    ("x-amz-date", "19700101T000000Z".to_owned()),
    (
      "Authorization",
      "AWS4-HMAC-SHA256 Credential=some_access_key/19700101/some_place/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date,Signature=521595a9eeee7092d3b2cc49d4db7cb828a5db5c7ad5136c149db0b0e7277f83".to_owned()
    )
  ])
  }

  #[test]
  fn test_put_object() {
    let url = Url::from_str("https://examplebucket.s3.amazonaws.com/test$file.text").unwrap();
    let headers = &[("x-amz-storage-class", "REDUCED_REDUNDANCY".to_owned())];
    let content = b"".as_slice();
    let result = S3HeadersBuilder::new(&url)
      .set_access_key("some_access_key")
      .set_secret_key("some_secret_key")
      .set_region("some_place")
      .set_datetime(S3DateTime::UnixTimestamp(1369324800)) // 20130524T000000Z
      .set_headers(headers)
      .set_method("PUT")
      .set_service("s3")
      .set_payload_hash_with_content(content)
      .build();

    assert::equal(result, vec![
    ("x-amz-storage-class", "REDUCED_REDUNDANCY".to_owned()),
    ("Host", "examplebucket.s3.amazonaws.com".to_owned()),
    ("x-amz-content-sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_owned()),
    ("x-amz-date", "20130523T160000Z".to_owned()),
    (
      "Authorization",
      "AWS4-HMAC-SHA256 Credential=some_access_key/20130523/some_place/s3/aws4_request,SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-storage-class,Signature=7e2911c8225f7591609bcbdc2faf8c443a898d8c83fc35b6a23f0b0e8084da60".to_owned()
    )
  ])
  }
}
