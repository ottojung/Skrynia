use std::fs;
mod history;
use history::{BANNED, BASE, INV, TAGS, git, ids};

#[test]
fn f1_supersession() {
    let doc = fs::read_to_string("docs/v3/intent-supersession.md").unwrap();
    let spec = fs::read_to_string("docs/v3-vm-spec.md").unwrap();
    let live = git(&["grep", "-h", "", "--", "docs/intent-records/"]);
    let old = git(&["grep", "-hF", "$id-", BASE, "--", "docs/intent-records/"]);
    let part = |a: &str, b: &str| doc.split(a).nth(1).unwrap().split(b).next().unwrap();
    let retired = ids(part("Retired CURRENT", "Retained records"));
    let trimmed = ids(part("Retained records", "Non-intent doc"));
    let old_ids = ids(&old);
    assert!(!old_ids.is_empty(), "historical intent IDs missing");
    for id in retired.iter().chain(trimmed.iter()) {
        assert!(old_ids.contains(id), "unattested historical ID: {id}");
    }
    for id in old_ids {
        let gone = !live.contains(id);
        assert_eq!(retired.contains(&id), gone, "{id} mapping");
        assert!(!trimmed.contains(&id) || !gone, "{id} trimmed but gone");
    }
    for t in BANNED.split('|') {
        assert!(!live.contains(t), "{t}");
    }
    for ln in doc.lines().filter(|l| l.starts_with('|')) {
        for tag in TAGS.split('|').filter(|x| !x.is_empty()) {
            let Some(i) = ln.find(tag) else { continue };
            let h = ln[i + tag.len() + 2..].split('"').next().unwrap();
            let f = ["docs/v3-vm-spec.md", "README.md"][usize::from(!tag.starts_with("`docs"))];
            let t = fs::read_to_string(f).unwrap();
            assert!(t.lines().any(|l| l.trim_start_matches('#').trim() == h));
        }
    }
    for p in INV.split('|') {
        assert!(spec.contains(p), "{p}");
    }
    let auth = fs::read_to_string("docs/intent-location-authority.md").unwrap();
    assert!(auth.contains("v2 compatibility target"), "status");
}
