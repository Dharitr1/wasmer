// !!! THIS IS A GENERATED FILE !!!
// ANY MANUAL EDITS MAY BE OVERWRITTEN AT ANY TIME
// Files autogenerated with cargo build (build/wasitests.rs).

#[test]
fn test_poll_oneoff() {
    assert_wasi_output!(
        "../../wasitests/poll_oneoff.wasm",
        "poll_oneoff",
        vec![],
        vec![
            (
                "hamlet".to_string(),
                ::std::path::PathBuf::from("wasitests/test_fs/hamlet")
            ),
            (
                "temp".to_string(),
                ::std::path::PathBuf::from("wasitests/test_fs/temp")
            ),
        ],
        vec![],
        "../../wasitests/poll_oneoff.out"
    );
}
