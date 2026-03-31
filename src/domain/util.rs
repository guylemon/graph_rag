use std::collections::HashSet;
use std::hash::Hash;

/// Returns a predicate closure that filters out duplicate items (based on `Eq + Hash`),
/// keeping only the first occurrence of each value. The predicate is stateful and
/// may be used in any iterator chain.
pub(crate) fn deduplicate<T>() -> impl FnMut(&T) -> bool
where
    T: Eq + Hash + Clone,
{
    let mut seen = HashSet::new();
    move |item| seen.insert(item.clone())
}
