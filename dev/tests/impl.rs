fn is_send<T: Send>() {}
fn is_sync<T: Sync>() {}

#[test]
fn assert_impls() {
    is_send::<byte_reader::Reader>();
    is_sync::<byte_reader::Reader>();
}
