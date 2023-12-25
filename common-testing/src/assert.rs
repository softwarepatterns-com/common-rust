use crate::setup;
use pretty_assertions::assert_eq;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::Deref;

/// Assert that they share the same AsRef somewhere.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///   let my_string = "abc";
///
///   // When there is more than one AsRef possible, say which one.
///   assert::ref_equal::<str>(&my_string, &"abc");
///   assert::ref_equal::<str>(&my_string.to_string(), &"abc".to_string());
///
///   // When there is only one AsRef possible, the types are inferred.
///   assert::ref_equal(&my_string, &b"abc");
/// }
/// ```
#[track_caller]
pub fn ref_equal<A>(a: &(impl AsRef<A> + Debug), b: &(impl AsRef<A> + Debug))
where
  A: Debug + PartialEq + ?Sized,
{
  assert_eq!(a.as_ref(), b.as_ref());
}

/// Asserts two values are equal using PartialEq, allowing for different
/// types to be compared.
///
/// Error message will show the values that were compared using
/// `pretty_assertions` crate.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///   let result = "abc";
///   equal(result, "abc");
///   equal(&result, &"abc");
///   equal(5, 5);
///   equal(&5, &5);
/// }
/// ```
#[track_caller]
pub fn equal<E, R>(a: E, b: R)
where
  E: Debug + PartialEq + PartialEq<R>,
  R: Debug + PartialEq,
{
  assert_eq!(a, b, "Expected {:?} to equal {:?}.", a, b);
}

/// Asserts two values are not equal using PartialEq, allowing for
/// different types to be compared.
///
/// Error message will show the values that were compared using
/// `pretty_assertions` crate.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///   let result = "abc";
///   not_equal(result, "def");
///   not_equal(result.as_bytes(), b"bcd");
/// }
#[track_caller]
pub fn not_equal<E, R>(a: E, b: R)
where
  E: Debug + PartialEq + PartialEq<R>,
  R: Debug + PartialEq,
{
  assert_ne!(a, b, "Expected {:?} to not equal {:?}.", a, b);
}

/// More specific than assert::equal, must be for AsRef<[u8]>. On failure,
/// the output message will show the hex values of the bytes for easier
/// debugging of longer byte arrays.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = vec![0x01, 0x0E, 0xF3];
///  assert::equal_bytes(&result, &[0x01, 0x0E, 0xF3]);
/// }
#[track_caller]
pub fn equal_bytes<R, E>(a: &R, b: &E)
where
  R: AsRef<[u8]> + ?Sized,
  E: AsRef<[u8]> + ?Sized,
{
  assert_eq!(
    a.as_ref(),
    b.as_ref(),
    "Expected {:02x?} to equal {:02x?}.",
    a.as_ref(),
    b.as_ref()
  );
}

/// Asserts that the value is equal to the contents of the file. Works
/// for anything that implements AsRef<[u8]>. This is useful for testing
/// against large fixtures. The file is read into memory and compared
/// against the value.
///
/// The file is not read until the assertion is run, preventing side
/// effects from reading the file during test setup or teardown, or from
/// affecting assertions earlier in the test.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result1 = "some file contents";
///  assert::equal_file_contents(&result, "./fixtures/test_file1");
///
///  // Works for anything that implements AsRef<[u8]>
///  let result2 = vec![0x01, 0x0E, 0xF3];
///  assert::equal_file_contents(&result, "./fixtures/test_file2");
/// }
/// ```
///
#[track_caller]
pub fn equal_file_contents<R>(a: &R, path: &str)
where
  R: AsRef<[u8]> + ?Sized,
{
  let expected = setup::get_file_contents(path).unwrap();
  ref_equal(&a.as_ref(), &expected);
}

/// More specific than assert::equal, must be for AsRef<[u8]>.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = vec![0x01, 0x0E, 0xF3];
///  assert::equal_hex_bytes(&result, "010EF3");
///  // or
///  assert::equal_hex_bytes(&result, "010ef3");
/// }
/// ```
#[track_caller]
pub fn equal_hex_bytes<R>(a: &R, b: &str)
where
  R: AsRef<[u8]> + ?Sized,
{
  let value = hex::encode(a.as_ref());
  assert_eq!(value, b, "Expected {} to equal {}.", value, b);
}

/// Assert that the value is some.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Some("abc");
///  assert::some(&result);
/// }
/// ```
#[track_caller]
pub fn some<T>(a: &Option<T>)
where
  T: Debug,
{
  assert!(a.is_some(), "Expected to be some: {:?}", a);
}

/// Asserts that the value is some and returns the value.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Some("abc");
///  let some = assert::some_into(result);
///  assert::equal(some, "abc");
/// }
/// ```
#[track_caller]
pub fn some_into<T>(a: Option<T>) -> T
where
  T: Debug,
{
  assert!(a.is_some(), "Expected to be some: {:?}", a);
  a.unwrap()
}

/// Assert that the value is none.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = None::<&str>;
///  assert::none(&result);
/// }
/// ```
#[track_caller]
pub fn none<T>(a: &Option<T>)
where
  T: Debug,
{
  assert!(a.is_none(), "Expected to be none: {:?}", a);
}

/// Assert that the value is ok.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Ok("abc");
///  assert::ok(&result);
/// }
/// ```
#[track_caller]
pub fn ok<T, E>(a: &Result<T, E>)
where
  T: Debug,
  E: Debug,
{
  assert!(a.is_ok(), "Expected to be ok: {:?}", a);
}

/// Asserts that the value is ok and returns the value.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Ok("abc");
///  let ok = assert::ok_into(a);
///  assert::equal(ok, "abc");
/// }
///
#[track_caller]
pub fn ok_into<T, E>(a: Result<T, E>) -> T
where
  T: Debug,
  E: Debug,
{
  assert!(a.is_ok(), "Expected to be ok: {:?}", a);
  a.unwrap()
}

/// Assert that the value is err.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Err("abc");
///  assert::err(&result);
/// }
/// ```
#[track_caller]
pub fn err<T, E>(a: &Result<T, E>)
where
  T: Debug,
  E: Debug,
{
  assert!(a.is_err(), "Expected to be err: {:?}", a);
}

/// Asserts that the value is err and returns the value.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let result = Err("abc");
///  let err = assert::err_into(a);
///  assert::equal(err, "abc");
/// }
#[track_caller]
pub fn err_into<T, E>(a: Result<T, E>) -> E
where
  T: Debug,
  E: Debug,
{
  assert!(a.is_err(), "Expected to be err: {:?}", a);
  a.unwrap_err()
}

/// Asserts that the value is default.
///
/// # Example
///
/// ```
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let a = 0;
///  assert::default(&a);
/// }
/// ```
#[track_caller]
pub fn default<R>(a: &R)
where
  R: Default + Debug + PartialEq + ?Sized,
{
  assert_eq!(a, &R::default());
}

/// Asserts that the value implements Cow and is borrowed.
/// This is useful for testing that a Cow is not cloned.
///
/// # Example
///
/// ```
/// use std::borrow::Cow;
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let a = Cow::Borrowed("abc");
///  assert::borrowed(&a);
/// }
/// ```
#[allow(clippy::ptr_arg)]
#[track_caller]
pub fn borrowed<R>(a: &Cow<'_, R>)
where
  R: Debug + PartialEq + ToOwned + ?Sized,
{
  assert!(
    match a {
      Cow::Borrowed(_) => true,
      Cow::Owned(_) => false,
    },
    "Expected {:?} to be borrowed",
    a.deref(),
  );
}

/// Asserts that the value implements Cow and is owned.
/// This is useful for testing that a Cow is cloned.
///
/// # Example
///
/// ```
/// use std::borrow::Cow;
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let a = Cow::Owned("abc".to_string());
///  assert::owned(&a);
/// }
/// ```
#[allow(clippy::ptr_arg)]
#[track_caller]
pub fn owned<R>(a: &Cow<'_, R>)
where
  R: Debug + PartialEq + ToOwned + ?Sized,
{
  assert!(
    match a {
      Cow::Borrowed(_) => false,
      Cow::Owned(_) => true,
    },
    "Expected {:?} to be owned",
    a.deref(),
  );
}

/// Asserts cursor position has reached the end. This is useful for testing
/// that a cursor has been completely consumed.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use common_testing::assert;
///
/// #[test]
/// fn test_1() {
///  let cursor = Cursor::new("abc");
///  assert::cursor_completely_consumed(&cursor);
/// }
/// ```
#[track_caller]
pub fn cursor_completely_consumed<T>(cursor: &Cursor<T>)
where
  T: AsRef<[u8]>,
{
  assert_eq!(
    cursor.position(),
    cursor.get_ref().as_ref().len() as u64,
    "Cursor was not completely consumed"
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Result;

  #[test]
  fn test_ref_equal() {
    let my_string = "abc";

    // When there is more than one AsRef possible, say which one.
    ref_equal::<str>(&my_string, &"abc");
    ref_equal::<str>(&my_string.to_string(), &"abc".to_string());

    // When there is only one AsRef possible, the types are inferred.
    ref_equal(&my_string, &b"abc");
  }

  #[test]
  fn test_equal() {
    let result = "abc";
    equal(result, "abc");
    equal(&result, &"abc");
    equal(5, 5);
    equal(&5, &5);
  }

  #[test]
  fn test_not_equal() {
    let result = "abc";
    not_equal(result, "def");
    not_equal(result.as_bytes(), b"bcd");
  }

  #[test]
  fn test_equal_bytes() {
    let result = vec![0x01, 0x0E, 0xF3];
    equal_bytes(&result, &[0x01, 0x0E, 0xF3]);
  }

  #[test]
  fn test_equal_file_contents() {
    let result1 = "some file content\n";
    equal_file_contents(&result1, "./fixtures/test.txt");
  }

  #[test]
  fn test_equal_hex_bytes() {
    let result = vec![0x01, 0x0E, 0xF3];
    equal_hex_bytes(&result, "010ef3");
  }

  #[test]
  fn test_some() {
    let result = Some("abc");
    some(&result);
  }

  #[test]
  fn test_some_into() {
    let result = Some("abc");
    let some = some_into(result);
    equal(some, "abc");
  }

  #[test]
  fn test_none() {
    let result = None::<&str>;
    none(&result);
  }

  #[test]
  fn test_ok() {
    let result = Result::Ok("abc");
    ok(&result);
  }

  #[test]
  fn test_ok_into() {
    let result = Result::Ok("abc");
    let ok = ok_into(result);
    equal(ok, "abc");
  }
}
