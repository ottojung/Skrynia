use std::{fs, process::Command};

const BANNED: &str = "Web Push|WebPush|root grant|issuer credential|issuerToken|X-Skrynia-If-SHA256|GET /store|PUT /store|SKRYNIA_URL|STORE_DIR|/grants/|GET /version|GET /health|quota|grant|subscribe|notification";
const INV: &str =
    "MUST NOT implement grants|The only endpoint is `POST /`|SKRYNIA_ROOT_WASM|ERR_PERM";
const TAGS: &str = "`docs/v3-vm-spec.md`|README|";
const BASE: &str = "8c0c14655584a7dead3c7df5ce490d23e59f9242";

fn git(a: &[&str]) -> String {
    String::from_utf8(Command::new("git").args(a).output().unwrap().stdout).unwrap()
}

fn ids(s: &str) -> Vec<&str> {
    s.split("$id-").skip(1).map(|c| &c[..16]).collect()
}

#[test]
fn f1_supersession() {
    let doc = fs::read_to_string("docs/v3/intent-supersession.md").unwrap();
    let spec = fs::read_to_string("docs/v3-vm-spec.md").unwrap();
    let live = git(&["grep", "-h", "", "--", "docs/intent-records/"]);
    let old = git(&["grep", "-hF", "$id-", BASE, "--", "docs/intent-records/"]);
    let part = |a: &str, b: &str| doc.split(a).nth(1).unwrap().split(b).next().unwrap();
    let retired = ids(part("Retired CURRENT", "Retained records"));
    let trimmed = ids(part("Retained records", "Non-intent doc"));
    for id in ids(&old) {
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
