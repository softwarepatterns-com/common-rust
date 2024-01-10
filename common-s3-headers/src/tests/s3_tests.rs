use crate::{
  aws_math::get_sha256,
  s3,
  s3_options::{S3DateTime, S3Options},
};
use common_testing::assert;
use std::str::FromStr;
use url::Url;

#[test]
fn test_get_object() {
  let url = Url::from_str("https://jsonlog.s3.amazonaws.com/test.json").unwrap();
  let options = S3Options::new(&url)
    .set_access_key("some_access_key")
    .set_secret_key("some_secret_key")
    .set_region("some_place")
    .set_datetime(S3DateTime::UnixTimestamp(0))
    .set_method("GET")
    .set_service("s3");
  let result = s3::get_headers(options);

  assert::equal(result, vec![
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
  let sha = get_sha256(content);
  let options = S3Options::new(&url)
    .set_access_key("some_access_key")
    .set_secret_key("some_secret_key")
    .set_region("some_place")
    .set_datetime(S3DateTime::UnixTimestamp(1369324800)) // 20130524T000000Z
    .set_headers(headers)
    .set_method("PUT")
    .set_service("s3")
    .set_payload_hash(&sha);
  let result = s3::get_headers(options);

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
