use std::rc::{Rc, Weak};

use crate::{Bus, Meta};
use common_testing::{assert, setup};
use futures::FutureExt;

fn spy(_message: (), _meta: &Meta) {}

fn spy1(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "1"
}

fn spy2(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "2"
}

fn spy3(what: &str, _meta: &Meta) -> String {
  what.to_owned() + "3"
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct TestObject<T> {
  value: T,
}

#[test]
fn test_emits_and_receives() {
  let mut bus = Bus::<(), _>::new();

  bus.on("a", spy);

  let result = bus.emit("a");

  assert::equal(result, [()])
}

#[test]
fn test_emits_and_receives_multiple() {
  let mut bus = Bus::<(), _>::new();

  bus.on("a", spy);

  let result = vec![bus.emit("a"), bus.emit("a"), bus.emit("a")];

  assert::equal(result.len(), 3)
}

#[test]
fn test_emits_and_receives_messages() {
  let mut bus = Bus::new();

  bus.on("a", spy1);

  let result = bus.emit_message("a", "p");

  assert::equal(result, ["p1"])
}

#[test]
fn allows_multiple_paths() {
  let mut bus = Bus::new();

  bus.on("a", spy1);
  bus.on("a.b", spy2);
  bus.on("a.c", spy3);

  let result1 = bus.emit_message("a", "x");
  let result2 = bus.emit_message("a.b.c", "y");
  let result3 = bus.emit_message("a.b", "z");
  let result4 = bus.emit_message("a.c", "0");

  assert::equal(result1, ["x1"]);
  assert::equal(result2, [].to_vec() as Vec<String>);
  assert::equal(result3, ["z2"]);
  assert::equal(result4, ["03"]);
}

#[test]
fn can_send_strings_as_message() {
  let mut bus = Bus::new();
  let list1 = setup::get_rc_ref_cell_empty_vec();
  let list2 = setup::get_rc_ref_cell_empty_vec();

  let cloned_list1 = list1.clone();
  bus.on("a", move |a: &str, _b| {
    cloned_list1.borrow_mut().push(a.to_owned());
  });

  let cloned_list2 = list2.clone();
  bus.on("d", move |a: &str, _b| {
    cloned_list2.borrow_mut().push(a.to_owned());
  });

  bus.emit_message("a", "b");
  bus.emit_message("a", "c");
  bus.emit_message("d", "e");
  bus.emit_message("d", "f");

  assert::equal(list1.borrow().as_slice(), ["b", "c"]);
  assert::equal(list2.borrow().as_slice(), ["e", "f"]);
}

#[test]
fn can_send_object() {
  let list = setup::get_rc_ref_cell_empty_vec();
  let mut bus = Bus::new();
  let cloned_list = list.clone();
  bus.on("a", move |a: _, _b| {
    cloned_list.borrow_mut().push(a);
  });

  let test_ref = Rc::new(TestObject { value: "hello" });
  // Only one copy exists so far.
  assert::equal(Rc::strong_count(&test_ref), 1);

  bus.emit_message("a", test_ref.clone());

  // In particular, this is not three because of the "ToOwned".
  assert::equal(Rc::strong_count(&test_ref), 2);
  assert::equal(Rc::weak_count(&test_ref), 0);
  assert::equal(list.borrow().as_slice(), [Rc::new(TestObject { value: "hello" })]);
}

#[test]
fn can_send_multiple_objects() {
  let list = setup::get_rc_ref_cell_empty_vec();
  let mut bus: Bus<'_, Rc<TestObject<&str>>, ()> = Bus::new();
  let cloned_list = list.clone();
  bus.on("a", move |a: Rc<TestObject<&str>>, _b| {
    cloned_list.borrow_mut().push(a);
  });

  let test_ref = Rc::new(TestObject { value: "hello" });
  bus.emit_from("a", test_ref.clone());
  bus.emit_message("a", test_ref.clone());
  bus.emit_message("a", test_ref.clone());

  // 4 copies: one in scope, three passed into events.
  // In particular, this is not six even though we use "ToOwned".
  assert::equal(Rc::strong_count(&test_ref), 4);
  assert::equal(Rc::weak_count(&test_ref), 0);
  assert::equal(
    list.borrow().as_slice(),
    [test_ref.clone(), test_ref.clone(), test_ref.clone()],
  );
}

/// Using weak and strong references are useful across async boundaries, error handling,
/// or best effort logging of partial structures.
///
/// Use-case example: Chrome console logging of deep html elements.
#[test]
fn can_send_weak_references_to_objects() {
  let list = setup::get_rc_ref_cell_empty_vec();
  let mut bus: Bus<'_, Weak<TestObject<&str>>, ()> = Bus::new();
  let cloned_list = list.clone();
  bus.on("a", move |a: Weak<TestObject<&str>>, _b| {
    cloned_list.borrow_mut().push(a);
  });

  {
    let test_ref = Rc::new(TestObject { value: "hello" });
    bus.emit_from("a", Rc::downgrade(&test_ref));
    bus.emit_message("a", Rc::downgrade(&test_ref));
    bus.emit_message("a", Rc::downgrade(&test_ref));

    // 1 strong reference kept
    assert::equal(Rc::strong_count(&test_ref), 1);
    // 3 weak references still exist
    assert::equal(Rc::weak_count(&test_ref), 3);
    assert::equal(list.borrow().as_slice().len(), 3);

    list.borrow().iter().for_each(|item| {
      // 1 strong reference kept
      assert::equal(item.strong_count(), 1);
      // 3 weak references still exist
      assert::equal(item.weak_count(), 3);
      // Weak reference is accessible.
      assert::some(&item.upgrade());
    });
  }

  list.borrow().iter().for_each(|item| {
    // Strong references is dropped.
    assert::equal(item.strong_count(), 0);
    // Weak references are gone.
    assert::equal(item.weak_count(), 0);
    // Weak reference is empty.
    assert::none(&item.upgrade());
  });
}

#[test]
fn allow_result_to_flow_to_emitter() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a", move |_, _| "what");

  let result = bus.emit_message("a", "");

  assert::equal(result, ["what"]);
}

#[test]
fn allow_future_result_to_flow_to_emitter() {
  let mut bus = Bus::<&str, _>::new();

  bus.on("a", move |_, _| async { futures::future::ok::<i32, i32>(1).await });

  let result = bus.emit_message("a", "");

  for i in result {
    let value = i.now_or_never().unwrap();
    assert::equal(value, Ok(1));
  }
}

#[test]
fn test_can_match_only_with_at_least_one_word_front() {
  let mut bus = Bus::<(), _>::new();

  bus.on("*", spy);
  bus.on("#.*", spy);
  bus.on("#.*.*", spy);
  bus.on("#.*.*.*", spy);
  bus.on("#.*.*.*.*", spy);

  // Should match none.
  assert::equal(bus.emit("").len(), 0);

  // Should match * and #.*, which represent a single word.
  assert::equal(bus.emit("a").len(), 2);

  // Should match #.* and #.*.*
  assert::equal(bus.emit("a.a").len(), 2);

  // Should match #.*, #.*.* and #.*.*.*
  assert::equal(bus.emit("a.a.a").len(), 3);

  // Should match #.*, #.*.*, #.*.*.*, and #.*.*.*.*
  assert::equal(bus.emit("a.a.a.a").len(), 4);
}

#[test]
fn test_can_match_only_with_at_least_one_word_back() {
  let mut bus = Bus::<(), _>::new();

  bus.on("*", spy);
  bus.on("*.#", spy);
  bus.on("*.*.#", spy);
  bus.on("*.*.*.#", spy);
  bus.on("*.*.*.*.#", spy);

  // Should match none.
  assert::equal(bus.emit("").len(), 0);

  // Should match * and #.*, which represent a single word.
  assert::equal(bus.emit("a").len(), 2);

  // Should match #.* and #.*.*
  assert::equal(bus.emit("a.a").len(), 2);

  // Should match #.*, #.*.* and #.*.*.*
  assert::equal(bus.emit("a.a.a").len(), 3);

  // Should match #.*, #.*.*, #.*.*.*, and #.*.*.*.*
  assert::equal(bus.emit("a.a.a.a").len(), 4);
}

#[test]
fn test_can_match_only_with_at_least_one_word_front_and_back() {
  let mut bus = Bus::<(), _>::new();

  bus.on("*", spy);
  bus.on("#.*.#", spy);
  bus.on("#.*.*.#", spy);
  bus.on("#.*.*.*.#", spy);
  bus.on("#.*.*.*.*.#", spy);

  // Should match none.
  assert::equal(bus.emit("").len(), 0);

  // Should match * and #.*, which represent a single word.
  assert::equal(bus.emit("a").len(), 2);

  // Should match #.* and #.*.*
  assert::equal(bus.emit("a.a").len(), 2);

  // Should match #.*, #.*.* and #.*.*.*
  assert::equal(bus.emit("a.a.a").len(), 3);

  // Should match #.*, #.*.*, #.*.*.*, and #.*.*.*.*
  assert::equal(bus.emit("a.a.a.a").len(), 4);
}

#[test]
fn test_cannot_match_the_empty_path() {
  let mut bus = Bus::<(), _>::new();

  bus.on("", spy);

  assert::equal(bus.emit("").len(), 0);
  assert::equal(bus.emit("a").len(), 0);
}
