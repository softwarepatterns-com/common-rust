pub use common_testing::assert::*;
use reqwest::{
  header::{HeaderMap, HeaderName, HeaderValue},
  StatusCode,
};
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};
use url::Url;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct AwsError {
  #[serde(rename = "Code")]
  code: String,
  #[serde(rename = "Message")]
  message: String,
  #[serde(rename = "RequestId")]
  request_id: String,
}

/// We shouldn't care which library is used for testing.
fn to_reqwest_headers(request_headers: Vec<(&'static str, String)>) -> HeaderMap {
  HeaderMap::from_iter(
    request_headers
      .into_iter()
      .map(|(k, v)| (HeaderName::from_str(k).unwrap(), HeaderValue::from_str(&v).unwrap())),
  )
}

/// We shouldn't care which library is used for testing.
fn from_reqwest_response_headers(reqwest_headers: HeaderMap) -> BTreeMap<String, String> {
  BTreeMap::from_iter(
    reqwest_headers
      .into_iter()
      .map(|(k, v)| (k.unwrap().as_str().to_owned(), v.to_str().unwrap().to_owned()))
      .collect::<Vec<(String, String)>>(),
  )
}

fn assert_not_aws_error(status_code: &StatusCode, response_headers: &HeaderMap, body: &str) {
  if status_code.as_u16() >= 400 {
    if let Some(content_type) = response_headers.get("content-type") {
      if content_type == "application/xml" {
        let aws_error: AwsError = serde_xml_rs::from_str(body).unwrap();
        println!("aws_error {:#?}\n", aws_error);
        panic!("Error response from s3");
      }
    }
  }
}

pub fn content_type<S: Into<String>>(response_headers: BTreeMap<String, String>, content_type: S) {
  assert!(
    response_headers.contains_key("content-type"),
    "Expected response headers to have a content type."
  );

  let content_type_str = content_type.into();
  assert_eq!(
    response_headers.get("content-type"),
    Some(&content_type_str),
    "Expected headers to have Content-Type: \"{}\"",
    content_type_str
  );
}

pub fn request_get(url: Url, request_headers: Vec<(&'static str, String)>) -> (u16, BTreeMap<String, String>, String) {
  let client = reqwest::blocking::Client::new();
  let headers = to_reqwest_headers(request_headers);
  let response = client.get(url).headers(headers).send().unwrap();

  let status_code = response.status();
  let response_headers = response.headers().clone();
  let body = response.text().unwrap();

  assert_not_aws_error(&status_code, &response_headers, &body);

  (
    status_code.as_u16(),
    from_reqwest_response_headers(response_headers),
    body,
  )
}

pub fn request_put(
  url: Url,
  request_headers: Vec<(&'static str, String)>,
  body: Vec<u8>,
) -> (u16, BTreeMap<String, String>, String) {
  let client = reqwest::blocking::Client::new();
  let headers = to_reqwest_headers(request_headers);
  let response = client.put(url).headers(headers).body(body).send().unwrap();

  let status_code = response.status();
  let response_headers = response.headers().clone();
  let body = response.text().unwrap();

  assert_not_aws_error(&status_code, &response_headers, &body);

  (
    status_code.as_u16(),
    from_reqwest_response_headers(response_headers),
    body,
  )
}
