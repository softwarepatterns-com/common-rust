# Common Testing

Common testing shortcuts and utilities reused across projects. Freely share this code or copy into your own projects.

If there is a function that you've found useful in more than one project, consider adding it to this repo.

## Feature Highlights

### Result and Option Assertions

Asserts that a Result or Option is in a particular state. There are also `into` functions for converting the Result or Option into the inner value after asserting it is in the expected state.

- assert::ok
- assert::ok_into
- assert::err
- assert::err_into
- assert::some
- assert::some_into
- assert::none

```rust

use common_testing::assert;

#[test]
fn test_1() {
  let result: Result<u32, String> = Ok(1);
  assert::ok(&result);

  let result: Result<u32, String> = Ok(1);
  let ok = assert::ok_into(&result);
  assert::equal(ok, 1);

  let result: Result<u32, String> = Err("error".to_string());
  assert::err(&result);

  let result: Result<u32, String> = Err("error".to_string());
  let error = assert::err_into(&result);
  assert::equal(error, "error".to_string());

  let result: Option<u32> = Some(1);
  assert::some(&result);

  let result: Option<u32> = Some(1);
  let some = assert::some_into(&result);
  assert::equal(some, 1);

  let result: Option<u32> = None;
  assert::none(&result);
}
```

### COW Assertions

Asserts that a COW is in a particular state.

- assert::borrowed
- assert::owned

```rust
use common_testing::assert;

#[test]
fn test_1() {
  let cow: Cow<str> = Cow::Borrowed("borrowed");
  assert::borrowed(&cow);

  let cow: Cow<str> = Cow::Owned("owned".to_string());
  assert::owned(&cow);
}
```

### AsRef Assertions

Assert that there is a common AsRef implementation between two types, and that the AsRef values are equal. If there is more than one possible AsRef implementation, specify the type. If there is only one possible AsRef implementation, the type is inferred.

```rust
use common_testing::assert;

#[test]
fn test_1() {
  let my_string = "abc";

  // When there is more than one AsRef possible, say which one.
  assert::ref_equal::<str>(&my_string, &"abc");
  assert::ref_equal::<str>(&my_string.to_string(), &"abc".to_string());

  // When there is only one AsRef possible, the type is inferred.
  assert::ref_equal(&my_string, &b"abc");
}
```

### Testing Globals, OS calls, or File Systems

Rust will normally run tests in parallel, which can cause issues when tests are not isolated. This library provides a way to run tests sequentially, which is useful for testing code that uses global variables, OS calls, or file systems.

If you want to have tests run sequentially, use the `setup::sequential` function. This will return a lock that will prevent other tests that also use `setup::sequential` from running at the same time.

- setup::sequential

```rust
use common_testing::setup;

#[test]
fn test_1() {
 let _lock = setup::sequential();
 // test code
}

#[test]
fn test_2() {
 let _lock = setup::sequential();
 // test code
}
```

### Strict Equality Assertions

This library provides strict equality assertions that will fail if the types are not the same.

The marcos for testing equality (assert!, assert_eq!, assert_ne!) are not strict enough regarding types. For example, `assert_eq!(1, 1.0)` will pass. Also, the marcos will sometimes not detect issues until runtime. By providing non-macro functions that require PartialEq, the compiler catches more issues at compile time.

We also standardize the equality assertions to always print comparisons of the values in the error message, and require the values to be references to prevent assertions from taking accidental ownership.

- assert::equal
- assert::not_equal
- assert::default

```rust
use common_testing::assert;

#[test]
fn test_1() {
  assert::equal(&1, &1);
  assert::not_equal(&1, &2);

   // assert i is the default value for i's type.
  let i = 0;
  assert::default(i);
}
```

### File Setup and Assertions

Too many tests are written with unique boilerplate for working with fixtures, files or large data, leading to erratic expectations or side-effects that becomes difficult to maintain or to reason about when debugging asynchonous code. This library provides reusable file handling that encourage best practices that reduce side-effects or variability between tests.

- assert::equal_file_contents - Compare against fixtures or large data.
- assert::cursor_completely_consumed - Catch certain really common errors.

- setup::get_file_contents - Read fixtures or large data into the test.
- setup::create_dir_all - Guarantee a directory path exists for the test.
- setup::remove_file - Remove file if exists, useful to reset a test.
- setup::write_file_contents - Create temporary files for the test.

```rust
use common_testing::assert;

#[test]
fn test_1() {
 let result1 = "some file contents";
 assert::equal_file_contents(&result, "./fixtures/test_file1");

 // cursor_completely_consumed is useful for testing parsers.
  let mut cursor = Cursor::new("abc");
  cursor.read_u8().unwrap();
  cursor.read_u8().unwrap();
  cursor.read_u8().unwrap();
  assert::cursor_completely_consumed(&cursor);
}
```

### Binary Data Assertions

Assertions for working with binary data, making assumptions about the data types for more useful failure messages.

- assert::equal_bytes
- assert::equal_hex_bytes

```rust
use common_testing::assert;

#[test]
fn test_1() {
  // equal_bytes and equal_hex_bytes are useful for testing binary data.
  let result1 = "some file contents";
  let result2 = "some file contents";
  assert::equal_bytes(&result1, &result2);

  let result1 = "some file contents";
  let result2 = "736f6d652066696c6520636f6e74656e7473";
  assert::equal_hex_bytes(&result1, &result2);
}
```
