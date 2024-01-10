use common_testing::assert;
use hex_literal::hex;

#[test]
fn hex_test() {
  let bytes = hex!(
    r#"
      00010203 04050607
      08090a0b 0c0d0e0f
    "#
  );
  assert::equal(bytes, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
}
