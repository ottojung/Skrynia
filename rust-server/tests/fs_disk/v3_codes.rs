const ERR_INVALID: i32 = -1;
const ERR_OVERFLOW: i32 = -11;
const ERR_UNSUPPORTED: i32 = -12;

#[test]
fn status_codes_match_contract() {
    let all = [-1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12];
    for (i, code) in all.iter().enumerate() {
        assert_eq!(*code, -1 - i as i32);
    }
    assert_eq!(ERR_INVALID, -1);
    assert_eq!(ERR_OVERFLOW, -11);
    assert_eq!(ERR_UNSUPPORTED, -12);
}
