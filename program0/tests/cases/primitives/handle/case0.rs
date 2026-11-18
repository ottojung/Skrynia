use super::*;
#[test]
fn wire_register_set_invoke() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let res = handle(
        &reg,
        &disk,
        &Rng(9),
        &CatEval,
        &encode_req(&Request::Register {
            code: b"a".to_vec(),
        }),
    );
    let res = decode_res(&res).unwrap();
    assert_eq!(res.status, STATUS_OK);
    let id = res.body;
    let out = handle(
        &reg,
        &disk,
        &Rng(9),
        &CatEval,
        &encode_req(&Request::Invoke {
            id: id.clone(),
            input: b"1".to_vec(),
        }),
    );
    assert_eq!(decode_res(&out).unwrap().body, b"a:1");
    handle(
        &reg,
        &disk,
        &Rng(9),
        &CatEval,
        &encode_req(&Request::Set {
            id: id.clone(),
            code: b"b".to_vec(),
        }),
    );
    let out = handle(
        &reg,
        &disk,
        &Rng(9),
        &CatEval,
        &encode_req(&Request::Invoke {
            id,
            input: b"2".to_vec(),
        }),
    );
    assert_eq!(decode_res(&out).unwrap().body, b"b:2");
}
