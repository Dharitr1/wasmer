// !!! THIS IS A GENERATED FILE !!!
// ANY MANUAL EDITS MAY BE OVERWRITTEN AT ANY TIME
// Files autogenerated with cargo build.


#[test]
fn test_snapshot1_fseek() {
    assert_wasi_output!(
        "../wasi_test_resources/snapshot1/fseek.wasm",
        "snapshot1_fseek",
        vec![],
        vec![(".".to_string(), ::std::path::PathBuf::from("tests/wasi_test_resources/test_fs/hamlet")),],
        vec![],
        "../wasi_test_resources/fseek.out"
    );
}
