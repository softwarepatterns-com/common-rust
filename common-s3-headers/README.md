# Common S3 Headers

Only the headers necessary for communicating with S3-like services. It's all you need.

## Get

```
// Get from S3 from some location.
let url = Url::from_str("https://example.s3.amazonaws.com/test/test.json").unwrap();

let headers = S3HeadersBuilder::new(&url)
  .set_access_key(&access_key)
  .set_secret_key(&secret_key)
  .set_region(&region)
  .set_method("GET")
  .set_service("s3")
  .build();

// Use Reqwest or any other library!
let response = reqwest::blocking::Client::new()
  .get(url)
  .headers(reqwest::header::HeaderMap::from_iter(headers.into_iter().map(
    |(k, v)| {
      (
        reqwest::header::HeaderName::from_str(k).unwrap(),
        reqwest::header::HeaderValue::from_str(&v).unwrap(),
      )
    },
  )))
  .send()
  .unwrap();
```

## Put

```
// We want to put content into S3.
let url = Url::from_str("https://example.s3.amazonaws.com/test/test.json").unwrap();
let content = "{\"c\":\"d\"}\n".as_bytes().to_vec();

// Build the S3 headers.
let (access_key, secret_key, region) = setup::get_config_from_env("TEST_S3");
let headers = S3HeadersBuilder::new(&url)
  .set_access_key(&access_key)
  .set_secret_key(&secret_key)
  .set_region(&region)
  .set_method(PUT_METHOD)
  .set_service("s3")
  .set_payload_hash_with_content(&content)
  .build();

// Use Reqwest or any other library!
let response = reqwest::blocking::Client::new()
  .get(url)
  .headers(reqwest::header::HeaderMap::from_iter(headers.into_iter().map(
    |(k, v)| {
      (
        reqwest::header::HeaderName::from_str(k).unwrap(),
        reqwest::header::HeaderValue::from_str(&v).unwrap(),
      )
    },
  )))
  .send()
  .unwrap();
```

## List

```
let url = Url::from_str("https://example.s3.amazonaws.com/").unwrap();

let headers = S3HeadersBuilder::new(&url)
  .set_access_key(&access_key)
  .set_secret_key(&secret_key)
  .set_region(&region)
  .set_method("GET")
  .set_service("s3")
  .build();

// Use Reqwest or any other library!
let response = reqwest::blocking::Client::new()
  .get(url)
  .headers(reqwest::header::HeaderMap::from_iter(headers.into_iter().map(
    |(k, v)| {
      (
        reqwest::header::HeaderName::from_str(k).unwrap(),
        reqwest::header::HeaderValue::from_str(&v).unwrap(),
      )
    },
  )))
  .send()
  .unwrap();
```
