mod fixture;
mod scoped;
mod subscriptions;
#[test]
fn scoped_quota_and_metadata() {
    let (data, child) = scoped::init();
    assert_eq!(data.usage(&child, None), (1000 + 512 + 2048, 1));
    assert_eq!(
        data.usage(&child, Some(("alpha.one", Some(1000), 2000))),
        (2000 + 512 + 2048, 1)
    );
    assert_eq!(
        data.usage(&child, Some(("alpha.two", None, 10))),
        (1000 + 512 + 10 + 512 + 2048, 2)
    );
    assert!(
        data.check_quota(&child, "alpha.one", Some(1000), 1000)
            .is_err()
    );
    assert!(data.check_quota(&child, "alpha.two", None, 9000).is_err());
}
