use crate::test_util::{assert, setup};
use common_s3_headers::{self, S3Options};
use std::str::FromStr;
use url::Url;

#[test]
fn test_put_object() {
  let (access_key, secret_key, region) = setup::get_config_from_env("TEST_S3");

  let url = Url::from_str("https://jsonlog.s3.amazonaws.com/test/test2.json").unwrap();
  let content = "{\"c\":\"d\"}\n".as_bytes().to_vec();
  let sha = common_s3_headers::get_sha256(&content);
  let options = S3Options::new(&url)
    .set_access_key(&access_key)
    .set_secret_key(&secret_key)
    .set_region(&region)
    .set_method("PUT")
    .set_service("s3")
    .set_payload_hash(&sha);

  let headers: Vec<(&str, String)> = common_s3_headers::get_headers(options);

  let (status_code, _, body) = assert::request_put(url, headers, content);
  assert::equal(status_code, 200);
  assert::equal(body, "");
}
