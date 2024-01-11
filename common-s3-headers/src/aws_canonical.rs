/// Given a list of headers, returns headers that match the AWS spec.
///
/// This includes:
/// - Lowercasing all keys.
/// - Removing all headers that don't start with "x-amz-" or are "host", "content-type", or "range".
/// - Sorting the headers by key.
///
/// # Examples
///
/// ```
/// use common_s3_headers::aws_canonical::to_canonical_headers;
///
/// let headers = vec![
///  ("Host", "examplebucket.s3.amazonaws.com"),
///  ("Range", "bytes=0-9"),
///  ("x-amz-date", "20130524T000000Z"),
///  ("x-amz-storage-class", "REDUCED_REDUNDANCY"),
/// ];
/// let canonical_headers = to_canonical_headers(&headers);
/// assert_eq!(
///  canonical_headers,
///  vec![
///   ("host".to_owned(), "examplebucket.s3.amazonaws.com"),
///   ("range".to_owned(), "bytes=0-9"),
///   ("x-amz-date".to_owned(), "20130524T000000Z"),
///   ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
///  ]
/// );
/// ```
///
pub fn to_canonical_headers<K: AsRef<str>, V: AsRef<str>>(headers: &[(K, V)]) -> Vec<(String, &str)> {
  let mut canonical_headers = headers
    .iter()
    .filter_map(|(k, v)| {
      let key = k.as_ref().to_lowercase();
      if key.starts_with("x-amz-") || key == "host" || key == "content-type" || key == "range" {
        Some((key, v.as_ref().trim()))
      } else {
        None
      }
    })
    .collect::<Vec<(String, &str)>>();
  canonical_headers.sort();
  canonical_headers
}

/// Gets the headers necessary to ask for a byte range. Allocates.
/// # Examples
///
/// ```
/// use common_s3_headers::aws_canonical::get_range_headers;
///
/// let headers = get_range_headers(1, None);
/// assert_eq!(headers, vec![("Accept", "application/octet-stream".to_string()), ("Range", "bytes=1-".to_string())]);
///
/// let headers = get_range_headers(1, Some(2));
/// assert_eq!(headers, vec![("Accept", "application/octet-stream".to_string()), ("Range", "bytes=1-2".to_string())]);
/// ```
///
pub fn get_range_headers(start: u64, end: Option<u64>) -> Vec<(&'static str, String)> {
  let mut range = format!("bytes={}-", start);

  if let Some(end) = end {
    range.push_str(&end.to_string());
  }

  // If range, then the content type must be application/octet-stream.
  vec![("Accept", "application/octet-stream".to_string()), ("Range", range)]
}

#[cfg(test)]
mod tests {
  use super::*;
  use common_testing::assert;

  #[test]
  fn to_canonical_headers_lowercases() {
    let headers = vec![
      ("Host", "examplebucket.s3.amazonaws.com"),
      ("Range", "bytes=0-9"),
      ("x-amz-date", "20130524T000000Z"),
      ("x-amz-storage-class", "REDUCED_REDUNDANCY"),
    ];
    let canonical_headers = to_canonical_headers(&headers);
    assert::equal(
      canonical_headers,
      vec![
        ("host".to_owned(), "examplebucket.s3.amazonaws.com"),
        ("range".to_owned(), "bytes=0-9"),
        ("x-amz-date".to_owned(), "20130524T000000Z"),
        ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
      ],
    );
  }

  #[test]
  fn test_to_canonical_headers_lowercases_removes_sorts() {
    // Should sort by key.
    let headers = &[
      ("x-amz-special", "a"),
      // Should lowercase this.
      ("Host", "s3.etc"),
      ("x-amz-storage-class", "REDUCED_REDUNDANCY"),
      ("content-Type", "application/json"),
      ("x-amz-date", "19700101T000000Z"),
      // Should remove this.
      ("foo", "bar"),
    ];

    let result = to_canonical_headers(headers);

    assert::equal(
      result,
      vec![
        ("content-type".to_owned(), "application/json"),
        ("host".to_owned(), "s3.etc"),
        ("x-amz-date".to_owned(), "19700101T000000Z"),
        ("x-amz-special".to_owned(), "a"),
        ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
      ],
    )
  }

  #[test]
  fn test_to_canonical_headers_accepts_ref_array_string_string() {
    let headers = &[
      ("Host".to_string(), "s3.etc".to_string()),
      ("x-amz-storage-class".to_string(), "REDUCED_REDUNDANCY".to_string()),
      ("content-Type".to_string(), "application/json".to_string()),
    ];

    let result = to_canonical_headers(headers);

    assert::equal(
      result,
      vec![
        ("content-type".to_owned(), "application/json"),
        ("host".to_owned(), "s3.etc"),
        ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
      ],
    )
  }

  #[test]
  fn test_to_canonical_headers_accepts_ref_vec_string_string() {
    let headers = vec![
      ("Host".to_string(), "s3.etc".to_string()),
      ("x-amz-storage-class".to_string(), "REDUCED_REDUNDANCY".to_string()),
      ("content-Type".to_string(), "application/json".to_string()),
    ];

    let result = to_canonical_headers(&headers);

    assert::equal(
      result,
      vec![
        ("content-type".to_owned(), "application/json"),
        ("host".to_owned(), "s3.etc"),
        ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
      ],
    )
  }

  #[test]
  fn test_to_canonical_headers_accepts_borrows() {
    let borrow1 = "s3.etc";
    let borrow2 = "x-amz-storage-class";
    let headers = &[
      ("Host", borrow1),
      (borrow2, "REDUCED_REDUNDANCY"),
      ("content-Type", "application/json"),
    ];

    let result = to_canonical_headers(headers);

    assert::equal(
      result,
      vec![
        ("content-type".to_owned(), "application/json"),
        ("host".to_owned(), borrow1),
        ("x-amz-storage-class".to_owned(), "REDUCED_REDUNDANCY"),
      ],
    )
  }

  #[test]
  fn test_get_range_headers() {
    let headers = get_range_headers(1, None);
    assert::equal(
      headers,
      vec![
        ("Accept", "application/octet-stream".to_owned()),
        ("Range", "bytes=1-".to_owned()),
      ],
    );

    let headers = get_range_headers(1, Some(2));
    assert::equal(
      headers,
      vec![
        ("Accept", "application/octet-stream".to_owned()),
        ("Range", "bytes=1-2".to_owned()),
      ],
    );
  }
}
