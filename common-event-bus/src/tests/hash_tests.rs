use crate::{Bus, Meta};
use common_testing::{assert, setup};

fn spy1(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "1"
}

fn spy2(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "2"
}

fn spy3(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "3"
}

#[test]
fn hash_allows_missing_words() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a.#", spy1);

  let result = bus.emit_message("a", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_missing_back() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a.#", spy1);

  let result = bus.emit_message("a.b", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_multiple_missing_back() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a.#", spy1);

  let result = bus.emit_message("a.b.c", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_missing_front() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("#.b", spy1);

  let result = bus.emit_message("a.b", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_multiple_missing_front() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("#.c", spy1);

  let result = bus.emit_message("a.b.c", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_single_missing_word() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a.#.c", spy1);

  let result = bus.emit_message("a.b.c", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_matches_any_count() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a.#", spy1);

  let result = bus.emit_message("a.b.c", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn hash_means_optional() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("metrics.#.changed", spy1);

  let result = bus.emit_message("metrics.changed", "b");

  assert::equal(result, ["b1"]);
}

#[test]
fn avoids_multiples() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("metrics.#", spy1);
  bus.on("ads.#", spy2);
  bus.on("#.changed", spy3);

  let result = bus.emit_message("metrics.changed", "b");

  assert::equal(result, ["b1", "b3"]);
}

#[test]
fn handler_reuse_is_still_selective() {
  let mut bus = Bus::<&str, _>::new();
  let list = setup::get_rc_ref_cell_empty_vec();

  let list_clone = list.clone();
  let handler = move |e: &str, _meta: &Meta| {
    list_clone.borrow_mut().push(e.to_owned());
  };

  bus.on("metrics.#", handler.clone());
  bus.on("ads.#", handler.clone());
  bus.on("#.changed", handler);

  let result = bus.emit_message("metrics.changed", "b");

  assert::equal(result, [(), ()]);
  assert::equal(list.borrow().as_slice(), ["b", "b"]);
}
