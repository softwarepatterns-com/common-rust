#![allow(dead_code)]

use std::{
  collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
  fmt::Debug,
  sync::{Arc, RwLock},
};

#[derive(Debug, Default)]
pub struct Bus<'a, E: ToOwned + Default, R> {
  head: Node<'a, E, R>,
  head_count: usize,
}

impl<'a, E: ToOwned + Default, R> Bus<'a, E, R> {
  pub fn new() -> Self {
    Self {
      head: Node::new("", 0),
      head_count: 0,
    }
  }

  pub fn on(&mut self, topic: &'static str, f: impl FnMut(<E as ToOwned>::Owned, &Meta) -> R + 'a) -> &mut Self {
    let last_node = self.add(topic);
    let fn_list = &mut last_node.f_list;
    fn_list.push(Arc::new(RwLock::new(f)));
    self
  }

  pub fn emit_message<'topic, S>(&self, topic: S, message: E) -> Vec<R>
  where
    S: AsRef<str> + 'topic,
    R: 'topic,
  {
    let topic_ref = topic.as_ref();
    let words: Vec<&str> = topic_ref.split('.').filter(|&x| !x.is_empty()).collect();
    let mut results = vec![];

    for node in self.get_list(&words) {
      let meta = Meta {
        topic: topic_ref,
        words: &words,
      };
      results.extend(
        node
          .f_list
          .iter()
          .map(|f| f.write().unwrap()(message.to_owned(), &meta)),
      );
    }

    results
  }

  pub fn emit<'topic, S>(&self, topic: S) -> Vec<R>
  where
    S: AsRef<str> + 'topic,
    R: 'topic,
  {
    self.emit_message(topic, E::default())
  }

  pub fn emit_from<'topic, A: Into<E>, S>(&self, topic: S, message: A) -> Vec<R>
  where
    S: AsRef<str> + 'topic,
    R: 'topic,
  {
    self.emit_message(topic, message.into())
  }

  pub fn get_list<'local>(&self, words: &[&'local str]) -> Vec<&Node<'a, E, R>> {
    // TODO: Remove these allocations. Waiting for use-case to justify cost savings, such as processing a really large file.
    //
    // Possible solution A: Reuse memory by saving them into Bus object since there is a natural limit of the number of possible
    // events a callee subscribed to. Downside is cross-thread issues and therefore a performance penalty.
    //
    // Possible solution B: Move these to the stack instead of the heap. This would mean defining size limits for number of
    // allowed nodes and number of allowed subscribers. Benefit would be less penalties from cross-thread scenarios at the cost of
    // additional stack allocations.
    //
    // Possible solution C: Add a caching layer like the TypeScript version, since it would optimize the hot paths.
    //
    // Note that the vec for the params could be changed to an array if we could use small arrays of unknown size.
    let mut routes: VecDeque<(&Node<E, R>, usize)> = VecDeque::from([(&self.head, 0)]);
    let mut final_nodes: BTreeSet<usize> = BTreeSet::new(); // track found nodes, no duplicates should be returned
    let mut final_routes: Vec<&Node<E, R>> = Vec::with_capacity(16); // remember found functions to call

    while let Some(route) = routes.pop_front() {
      let (cursor, index) = route;
      let right = &cursor.right;
      let words_len = words.len();

      match words.get(index) {
        None => {
          if !cursor.f_list.is_empty() && !final_nodes.contains(&cursor.index) {
            // Remember that we've seen this node.
            final_nodes.insert(cursor.index);
            // Save functions we should call along with matched params.
            final_routes.push(cursor);
          }
        }
        Some(&word) => {
          if let Some(found_node) = right.get(word) {
            routes.push_back((found_node, index + 1));
          }
          if let Some(found_node) = right.get("*") {
            routes.push_back((found_node, index + 1));
          }
        }
      }

      if let Some(right_hash) = right.get("#") {
        for i in index..(words_len + 1) {
          routes.push_back((right_hash, i));
        }
      }
    }

    final_routes
  }

  pub fn add(&mut self, topic: &'static str) -> &mut Node<'a, E, R> {
    let mut cursor = &mut self.head;
    for word in topic.split('.') {
      let right = &mut cursor.right;

      cursor = match right.entry(word) {
        Entry::Occupied(occupied) => occupied.into_mut(),
        Entry::Vacant(vacant) => {
          self.head_count += 1;
          let node = Node::new(word, self.head_count);
          vacant.insert(node)
        }
      }
    }

    cursor
  }
}

type HeldFunction<'a, E, R> = Arc<RwLock<dyn for<'b, 'c> FnMut(<E as ToOwned>::Owned, &'c Meta) -> R + 'a>>;

#[derive(Default)]
pub struct Node<'a, E: ToOwned + Default, R> {
  /// Child nodes are words that are to the right in a string.
  right: BTreeMap<&'a str, Node<'a, E, R>>,
  /// The functions can be mutably called and therefore hold state, but the list won't be edited unless the
  /// Node is mutable.
  f_list: Vec<HeldFunction<'a, E, R>>,
  /// Used to track visited nodes. It's an unique value within a bus. Maybe could be replaced with a weak
  /// ref of the node instead.
  index: usize,
}

impl<'a, E: ToOwned + Default, R> Node<'a, E, R> {
  pub fn new(_word: &'a str, index: usize) -> Self {
    Self {
      right: BTreeMap::new(),
      f_list: Vec::new(),
      index,
    }
  }
}

impl<'a, E: ToOwned + Default, R> Debug for Node<'a, E, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Node")
      .field("right", &self.right)
      .field("index", &self.index)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct Meta<'topic> {
  /// Note: These strings will always be static because of the library defintion.
  pub topic: &'topic str,
  /// Convenience, we already have the split.
  pub words: &'topic [&'topic str],
}

#[cfg(test)]
mod tests;
