use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub trait Stats {
  fn add(
    self: &mut Self,
    l0: usize,
    l0_name: Option<&str>,
    l1: usize,
    l1_name: Option<&str>,
    l2: Option<usize>,
    value: &str,
  );

  fn describe(self: &Self, l0: Option<usize>, l1: Option<usize>, l2: Option<usize>) -> String;
}

#[derive(Default, Deserialize, Serialize)]
pub struct LeafStats {
  #[serde(skip_serializing_if = "String::is_empty", default)]
  pub name: String,
  pub len: usize,
}

#[derive(Default, Deserialize, Serialize)]
pub struct NodeStats<T> {
  #[serde(skip_serializing_if = "String::is_empty", default)]
  pub name: String,
  pub children: Vec<T>,
}

impl<T> NodeStats<T>
where
  T: Default,
{
  fn add_level(self: &mut Self, index: usize) -> &mut T {
    if index >= self.children.len() {
      self.children.resize_with(index + 1, Default::default);
    }

    &mut self.children[index]
  }
}

/** Statistics for L0 and L1 levels */
pub type L0L1Stats = NodeStats<NodeStats<LeafStats>>;

impl L0L1Stats {
  pub fn new(name: String) -> Self {
    L0L1Stats {
      name,
      children: Vec::new(),
    }
  }
}

impl Stats for L0L1Stats {
  fn add(
    self: &mut Self,
    l0: usize,
    l0_name: Option<&str>,
    l1: usize,
    l1_name: Option<&str>,
    _l2: Option<usize>,
    value: &str,
  ) {
    let mut l0_stats = self.add_level(l0);
    if l0_stats.name.len() == 0 {
      if let Some(n) = l0_name {
        l0_stats.name = String::from(n);
      }
    }

    let mut l1_stats = l0_stats.add_level(l1);
    if l1_stats.name.len() == 0 {
      if let Some(n) = l1_name {
        l1_stats.name = String::from(n);
      }
    }

    l1_stats.len += value.len() + 1;
  }

  fn describe(self: &Self, l0: Option<usize>, l1: Option<usize>, l2: Option<usize>) -> String {
    let l0_child = l0.and_then(|loc| self.children.get(loc));
    let l1_child = l0_child
      .zip(l1)
      .and_then(|(child, loc)| child.children.get(loc));

    let l1_name = l1_child
      .and_then(|l| {
        if l.name.len() == 0 {
          None
        } else {
          Some(Cow::Borrowed(&l.name))
        }
      })
      .or_else(|| l1.map(|l| Cow::Owned((l + 1).to_string())));

    // This is hardcoded for Bible-style references right now.
    match (l0_child, l1_name, l2) {
      (Some(l0c), Some(l1_name), Some(l2)) => format!("{} {}:{}", l0c.name, l1_name, l2 + 1),
      (Some(l0c), Some(l1_name), None) => format!("{} {}", l0c.name, l1_name),
      (Some(l0c), None, None) => l0c.name.clone(),
      _ => String::new(),
    }
  }
}
