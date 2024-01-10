use crate::{
  aws_canonical::to_canonical_headers,
  aws_format,
  aws_math::{self, sign, HmacSha256},
  s3_options::S3Options,
};
use hmac::Mac;

/// Gets all the headers necessary to make a request to a AWS compatible service.
pub fn get_headers(options: S3Options) -> Vec<(&'static str, String)> {
  let url = options.url;
  let payload_hash = options.payload_hash;
  let datetime = options.get_offset_datetime();
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

  let auth_header = get_authorization_header(options.set_headers(&headers));

  headers.push(("Authorization", auth_header));
  headers
}

/// Gets the headers necessary to ask for a byte range.
pub fn get_range_headers(start: u64, end: Option<u64>) -> Vec<(&'static str, String)> {
  let mut range = format!("bytes={}-", start);

  if let Some(end) = end {
    range.push_str(&end.to_string());
  }

  let headers: Vec<(&'static str, String)> = vec![("Accept", "application/octet-stream".to_string()), ("Range", range)];
  headers
}

/// Only gets the authorirzation header.
pub fn get_authorization_header(options: S3Options) -> String {
  let datetime = options.get_offset_datetime();
  let region = options.region;
  let access_key = options.access_key;
  let secret_key = options.secret_key;
  let service = options.service;
  let url = options.url;
  let method = options.method;
  let payload_hash = options.payload_hash;
  let canonical_headers = to_canonical_headers(options.headers);
  let canonical_request = aws_format::canonical_request_string(method, url, &canonical_headers, payload_hash);

  println!("canonical_request: {:?}", canonical_request);

  let string_to_sign = aws_format::string_to_sign(&datetime, region, service, &canonical_request);
  let signing_key = aws_math::get_signature_key(&datetime, secret_key, region, service);

  println!("string_to_sign: {:?}", string_to_sign);
  println!("signing_key: {:?}", signing_key);

  let hmac: HmacSha256 = sign(&signing_key, string_to_sign.as_bytes());
  let signature = hex::encode(hmac.finalize().into_bytes());
  let signed_headers = aws_format::get_keys(&canonical_headers).join(";");

  aws_format::authorization_header_string(access_key, &datetime, region, service, &signed_headers, &signature)
}
