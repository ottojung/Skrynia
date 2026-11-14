use crate::{QuotaIndex, Usage, matches};
use std::collections::HashMap;
#[test]
fn mutation_counters_agree_with_full_scan() {
    let scopes = ["*", "a*", "ab*", "ab", "b*", "xyz", "", "q*"];
    let keys = ["a", "ab", "abc", "ax", "ba", "b", "xyz", "q"];
    let mut index = QuotaIndex::default();
    let mut objects = HashMap::<String, u64>::new();
    for scope in scopes {
        index.scopes.insert(scope, Usage::default());
    }
    for step in 0..250u64 {
        let key = keys[(step as usize * 7 + (step / 7) as usize) % keys.len()];
        let old = objects.remove(key);
        let new = if step % 5 == 0 {
            None
        } else {
            Some((step * 37) % 1000)
        };
        if let Some(size) = new {
            objects.insert(key.into(), size);
        }
        index.scopes.change(key, old, new);
        for scope in scopes {
            let mut expected = Usage::default();
            for (key, size) in &objects {
                if matches(scope, key) {
                    expected.add(Usage::object(*size));
                }
            }
            assert_eq!(index.scopes.get(scope), expected, "step {step}: {scope}");
        }
    }
}
