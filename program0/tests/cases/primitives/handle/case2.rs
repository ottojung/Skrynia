use super::*;
#[test]
fn wire_lookup_revoke_and_auth() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let res = handle(
        &reg,
        &disk,
        &Rng(2),
        &CatEval,
        &encode_req(&Request::Register {
            code: b"code".to_vec(),
        }),
    );
    let id = decode_res(&res).unwrap().body;
    let got = handle(
        &reg,
        &disk,
        &Rng(2),
        &CatEval,
        &encode_req(&Request::Lookup { id: id.clone() }),
    );
    assert_eq!(decode_res(&got).unwrap().body, b"code");
    let bad = handle(
        &reg,
        &disk,
        &Rng(2),
        &CatEval,
        &encode_req(&Request::Revoke { id: vec![0; 32] }),
    );
    assert_eq!(decode_res(&bad).unwrap().status, STATUS_ERR);
    let rev = handle(
        &reg,
        &disk,
        &Rng(2),
        &CatEval,
        &encode_req(&Request::Revoke { id: id.clone() }),
    );
    assert_eq!(decode_res(&rev).unwrap().status, STATUS_OK);
    let after = handle(
        &reg,
        &disk,
        &Rng(2),
        &CatEval,
        &encode_req(&Request::Invoke { id, input: vec![] }),
    );
    assert_eq!(decode_res(&after).unwrap().status, STATUS_ERR);
}
