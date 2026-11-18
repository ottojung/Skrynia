use skrynia_program0::wdec::{decode_req, decode_res};
use skrynia_program0::wenc::{encode_req, encode_res};
use skrynia_program0::wire::{Request, Response, STATUS_OK};

#[test]
fn req_roundtrip() {
    let cases = vec![
        Request::Register {
            code: vec![1, 2, 3],
        },
        Request::Set {
            id: vec![9; 32],
            code: vec![4, 5],
        },
        Request::Invoke {
            id: vec![7; 16],
            input: vec![8, 9],
        },
        Request::Lookup { id: vec![5; 32] },
        Request::Revoke { id: vec![6; 32] },
    ];
    for c in cases {
        let b = encode_req(&c);
        assert_eq!(decode_req(&b).unwrap(), c);
    }
}

#[test]
fn res_roundtrip() {
    let r = Response {
        status: STATUS_OK,
        body: vec![42, 43],
    };
    let b = encode_res(&r);
    assert_eq!(decode_res(&b).unwrap(), r);
}
