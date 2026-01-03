#[test]
fn links_and_runs() {
    unsafe {
        assert_eq!(cea_sys::cea_test_add(2, 3), 5);
    }
}
