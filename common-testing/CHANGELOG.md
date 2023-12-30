# Common Testing Changelog

## v1.1.1

- Broaden accepted types for assert::equal and assert::not_equal (see unit tests for examples).

## v1.1.0

- Assume destination type for assert:equal and assert:not_equal are the second parameter.

## v1.0.0

- Add automatic unwrap functionality to assert::equal and assert::not_equal.

## v0.2.0

- Changed `setup::sequential` to absorb PoisonErrors in non-failing tests so only the failing test reports the error.

## v0.1.0

- Initial release.
