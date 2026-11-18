use skrynia_program0::cap::Disk;
use skrynia_program0::reg::Registry;
use std::collections::BTreeMap;
struct Corrupted(BTreeMap<String, Vec<u8>>);
impl Disk for Corrupted {
    type Err = ();
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.get(path).cloned())
    }
    fn write_atomic(&self, _: &str, _: &[u8]) -> Result<(), ()> {
        Ok(())
    }
    fn remove(&self, _: &str) -> Result<(), ()> {
        Ok(())
    }
    fn list(&self, _: &str) -> Result<Vec<String>, ()> {
        Ok(self
            .0
            .keys()
            .map(|k| k.trim_start_matches("reg/").to_string())
            .collect())
    }
}
#[test]
fn corrupt_program_fails_closed_on_registry_recovery() {
    let name = format!("reg/{}.bin", "01".repeat(32));
    let disk = Corrupted(BTreeMap::from([(name, b"\x00corrupt".to_vec())]));
    assert!(Registry::open(&disk, "reg").is_err());
}
