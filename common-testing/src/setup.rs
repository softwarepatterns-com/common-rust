use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

static SEQUENTIAL: Lazy<Mutex<()>> = Lazy::new(Mutex::default);

/// Allow tests with side-effects to run without interfering with each other. The
/// lock is released when the MutexGuard variable goes out of scope. Will ignore
/// poison errors from other tests so that our test can continue even if theirs fails.
///
/// Use this when working with global variables, OS calls, the file system, or other
/// shared resources.
///
/// # Example
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let _lock = setup::sequential();
///   // test code
/// }
///
/// #[test]
/// fn test_2() {
///   let _lock = setup::sequential();
///   // test code
/// }
/// ```
///
/// # See Also
///
/// [std::sync::Mutex::lock](https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock)
///
/// [std::sync::PoisonError](https://doc.rust-lang.org/std/sync/struct.PoisonError.html)
///
pub fn sequential<'a>() -> MutexGuard<'a, ()> {
  // If another test panics while holding the lock, the lock will be poisoned.
  // We ignore the poison error and return the lock anyway so we can continue
  // with other tests.
  SEQUENTIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Get an empty vector wrapped in an Rc<RefCell<>>.
///
/// Use to avoid random dependencies in test files for rare test cases.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let vec = setup::get_rc_ref_cell_empty_vec::<u8>();
///   // test code
/// }
/// ```
pub fn get_rc_ref_cell_empty_vec<T>() -> Rc<RefCell<std::vec::Vec<T>>> {
  Rc::new(RefCell::new(vec![]))
}

/// Get a read-only file handle. Use for fixtures you never want to change.
///
/// Prefer setup::get_file_contents() when you need to compare the contents
/// of a file. Prefer setup::get_reader_for_file() when you need a BufReader.
///
/// Use to avoid random dependencies in test files for rare test cases.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let mut file = setup::get_read_only_file("./fixtures/test.txt").unwrap();
///   // some test code
/// }
/// ```
pub fn get_read_only_file(path: &str) -> Result<File> {
  OpenOptions::new().read(true).open(path)
}

/// Get a BufReader for a file. Use for fixtures you never want to change.
///
/// Prefer setup::get_file_contents() when you need to compare the contents
/// of a file. Prefer setup::get_read_only_file() when you need a File handle.
///
/// Use to avoid random dependencies in test files for rare test cases.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let mut reader = setup::get_reader_for_file("./fixtures/test.txt").unwrap();
///   // some test code
/// }
/// ```
pub fn get_reader_for_file(path: &str) -> Result<BufReader<File>> {
  let file: File = get_read_only_file(path)?;
  Ok(BufReader::new(file))
}

/// Read the contents of a file into a vector of bytes. Use for fixtures you
/// never want to change.
///
/// Prefer this over get_reader_for_file() or get_read_only_file() when you
/// need to compare the contents of a file.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let contents = setup::get_file_contents("./fixtures/test.txt").unwrap();
///   // some test code
/// }
/// ```
pub fn get_file_contents(path: &str) -> Result<Vec<u8>> {
  let mut buf = Vec::new();
  get_reader_for_file(path)?.read_to_end(&mut buf)?;
  Ok(buf.to_owned())
}

/// Get a read and write file handle. Use this to create temporary files for
/// testing and comparison. The file will be created if it does not exist, and
/// it will be overridden if it does.
///
/// Prefer this function if you are testing dynamic content being written to a
/// file during the test. Prefer setup::get_file_contents() when you need to
/// get the contents of a file to load a fixture or large data. Prefer
/// assert::equal_file_contents() when you need to compare the contents of a
/// file as the result of a test.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   let mut file = setup::get_read_and_write_file("./test.txt").unwrap();
///   // some test code
/// }
/// ```
pub fn get_read_and_write_file(path: &str) -> Result<File> {
  remove_file(path)?;
  let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(false)
    .read(true)
    .open(path)?;

  Ok(file)
}

/// Get a writer for a file, creating the file if it does not exist.
/// Use this to create temporary files for testing and comparison.
///
/// Prefer this function if you are testing dynamic content being written to a
/// file during the test, and remember to call flush() when you are done. Prefer
/// setup::write_file_contents() when you need to create content in a file for
/// the purpose of a test.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///  let mut writer = setup::get_writer_for_file("./test.txt").unwrap();
///  // some test code
/// }
pub fn get_writer_for_file(path: &str) -> Result<BufWriter<File>> {
  let file: File = get_read_and_write_file(path)?;
  Ok(BufWriter::new(file))
}

/// Write bytes to a file, creating the file if it does not exist.
///
/// Prefer this function if you are creating file content for the purpose of a
/// test. Prefer setup::get_writer_for_file() when you are testing the act
/// of writing to a file.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   setup::write_file_contents("./test.txt", &[1, 2, 3]).unwrap();
///   // some test code
/// }
/// ```
pub fn write_file_contents(path: &str, contents: &[u8]) -> Result<()> {
  let file: File = get_read_and_write_file(path)?;
  BufWriter::new(file).write_all(contents)
}

/// Create a directory path if it does not exist. Will not throw an error if the
/// directory already exists. Use to guarantee the filesystem state before a test
/// runs.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   setup::create_dir_all("./.tmp/tests/test_1").unwrap();
///   setup::write_file_contents("./.tmp/tests/test_1/test.txt", &[1, 2, 3]).unwrap();
///   // some test code
/// }
/// ```
pub fn create_dir_all(path_dir: &str) -> Result<()> {
  if !Path::new(path_dir).is_dir() {
    std::fs::create_dir_all(path_dir)?;
  }
  Ok(())
}

/// Remove a file if it exists. Will not throw an error if the file does not exist.
///
/// Use to clean up temporary files created during a test. Prefer calling this
/// function at the beginning of a test to ensure filesystem state is clean and to
/// make debugging easier.
///
/// # Example
///
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///   setup::remove_file("./test.txt").unwrap();
///   // some test code
///   setup::write_file_contents("./test.txt", &[1, 2, 3]).unwrap();
///   // some more test code
/// }
/// ```
pub fn remove_file(file_path: &str) -> Result<()> {
  if Path::new(file_path).is_file() {
    std::fs::remove_file(file_path)?
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use std::io::Seek;

  use super::*;

  #[test]
  fn test_sequential_no_error() {
    let _lock = sequential();
  }

  #[test]
  fn test_get_rc_ref_cell_empty_vec() {
    let _lock = sequential();
    let vec = get_rc_ref_cell_empty_vec::<u8>();
    assert_eq!(vec.borrow().len(), 0);
  }

  #[test]
  fn test_get_read_only_file() {
    let _lock = sequential();
    let mut file = get_read_only_file("./fixtures/test.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "some file content\n");
  }

  #[test]
  fn test_get_reader_for_file() {
    let _lock = sequential();
    let mut reader = get_reader_for_file("./fixtures/test.txt").unwrap();
    let mut contents = String::new();
    reader.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "some file content\n");
  }

  #[test]
  fn test_get_file_contents() {
    let _lock = sequential();
    let contents = get_file_contents("./fixtures/test.txt").unwrap();
    assert_eq!(contents, b"some file content\n");
  }

  #[test]
  fn test_get_read_and_write_file() {
    let _lock = sequential();
    let mut file = get_read_and_write_file("./test.txt").unwrap();

    // write some content to the file
    file.write_all(b"test\n").unwrap();
    file.flush().unwrap();

    // read the content back
    let mut contents = String::new();
    file.seek(std::io::SeekFrom::Start(0)).unwrap();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "test\n");
  }

  #[test]
  fn test_get_writer_for_file() {
    let _lock = sequential();
    let mut writer = get_writer_for_file("./test.txt").unwrap();
    writer.write_all(b"test\n").unwrap();
    writer.flush().unwrap();
    let mut file = get_read_only_file("./test.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "test\n");
  }

  #[test]
  fn test_write_file_contents() {
    let _lock = sequential();
    write_file_contents("./test.txt", b"test\n").unwrap();
    let mut file = get_read_only_file("./test.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "test\n");
  }
}
