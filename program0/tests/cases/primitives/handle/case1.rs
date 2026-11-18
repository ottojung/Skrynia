use super::*;
#[test]
fn unknown_id_fails() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let out = handle(
        &reg,
        &disk,
        &Rng(1),
        &CatEval,
        &encode_req(&Request::Invoke {
            id: vec![0; 32],
            input: vec![],
        }),
    );
    assert_eq!(decode_res(&out).unwrap().status, STATUS_ERR);
}
