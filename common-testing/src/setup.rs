use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

static SEQUENTIAL: Lazy<Mutex<()>> = Lazy::new(Mutex::default);

/// Allow tests with global variables to run without interfering with each other. The
/// lock is released when the MutexGuard variable goes out of scope.
///
/// # Example
/// ```
/// use common_testing::setup;
///
/// #[test]
/// fn test_1() {
///  let _lock = setup::sequential();
///  // test code
/// }
///
/// #[test]
/// fn test_2() {
///  let _lock = setup::sequential();
///  // test code
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
  SEQUENTIAL.lock().unwrap()
}

/// Use to avoid random dependencies in test files for rare test cases.
///
/// # Example
///
/// ```
/// use common_testing::setup::get_rc_ref_cell_empty_vec;
///
/// #[test]
/// fn test_1() {
///  let vec = get_rc_ref_cell_empty_vec::<u8>();
///  // test code
/// }
/// ```
pub fn get_rc_ref_cell_empty_vec<T>() -> Rc<RefCell<std::vec::Vec<T>>> {
  Rc::new(RefCell::new(vec![]))
}

/// Get a read-only file handle. Use to avoid random dependencies in test
/// files for rare test cases.
pub fn get_read_only_file(path: &str) -> Result<File> {
  OpenOptions::new().read(true).open(path)
}

/// Get a reader for a file. Use to avoid random dependencies in test files
/// for rare test cases.
pub fn get_reader_for_file(path: &str) -> Result<BufReader<File>> {
  let file: File = get_read_only_file(path)?;
  Ok(BufReader::new(file))
}

/// Read the contents of a file into a vector of bytes.
///
/// # Example
///
/// ```
/// use common_testing::setup::get_file_contents;
///
/// #[test]
/// fn test_1() {
///  let contents = get_file_contents("test_file").unwrap();
/// }
/// ```
pub fn get_file_contents(path: &str) -> Result<Vec<u8>> {
  let mut buf = Vec::new();
  get_reader_for_file(path)?.read_to_end(&mut buf)?;
  Ok(buf.to_owned())
}

/// Get a read and write file handle. Use this to create temporary files for
/// testing and comparison.
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
pub fn get_writer_for_file(path: &str) -> Result<BufWriter<File>> {
  let file: File = get_read_and_write_file(path)?;
  Ok(BufWriter::new(file))
}

/// Write bytes to a file, creating the file if it does not exist.
/// Use this to create temporary files for testing and comparison.
pub fn write_file_contents(path: &str, contents: &[u8]) -> Result<()> {
  let file: File = get_read_and_write_file(path)?;
  BufWriter::new(file).write_all(contents)
}

/// Create a directory path if it does not exist.
///
/// # Example
///
/// ```
/// use common_testing::setup::create_dir_all;
///
/// #[test]
/// fn test_1() {
///  create_dir_all("test_dir").unwrap();
/// }
/// ```
pub fn create_dir_all(path_dir: &str) -> Result<()> {
  if !Path::new(path_dir).is_dir() {
    std::fs::create_dir_all(path_dir)?;
  }
  Ok(())
}

/// Remove a file if it exists.
///
/// # Example
///
/// ```
/// use common_testing::setup::remove_file;
///
/// #[test]
/// fn test_1() {
///  remove_file("test_file").unwrap();
/// }
/// ```
pub fn remove_file(file_path: &str) -> Result<()> {
  if Path::new(file_path).is_file() {
    std::fs::remove_file(file_path)?
  }
  Ok(())
}
