// !!! THIS IS A GENERATED FILE !!!
// ANY MANUAL EDITS MAY BE OVERWRITTEN AT ANY TIME
// Files autogenerated with cargo build.


#[test]
fn test_unstable_poll_oneoff() {
    assert_wasi_output!(
        "../wasi_test_resources/unstable/poll_oneoff.wasm",
        "unstable_poll_oneoff",
        vec![],
        vec![("hamlet".to_string(), ::std::path::PathBuf::from("tests/wasi_test_resources/test_fs/hamlet")),("temp".to_string(), ::std::path::PathBuf::from("tests/wasi_test_resources/test_fs/temp")),],
        vec![],
        "../wasi_test_resources/poll_oneoff.out"
    );
}
