// Rust test file autogenerated with cargo build (src/build_spectests.rs).
// Please do NOT modify it by hand, as it will be reseted on next build.
// Test based on spectests/memory.wast
#![allow(
    warnings,
    dead_code
)]
use wabt::wat2wasm;

use crate::webassembly::{instantiate, compile, ImportObject, ResultObject, Instance, Export};
use super::_common::{
    spectest_importobject,
    NaNCheck,
};


// Line 2
fn create_module_1() -> ResultObject {
    let module_str = "(module
      (memory (;0;) 0 0))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_1(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 3

#[test]
fn test_module_1() {
    let result_object = create_module_1();
    // We group the calls together
    start_module_1(&result_object);
}
fn create_module_2() -> ResultObject {
    let module_str = "(module
      (memory (;0;) 0 1))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_2(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 4

#[test]
fn test_module_2() {
    let result_object = create_module_2();
    // We group the calls together
    start_module_2(&result_object);
}
fn create_module_3() -> ResultObject {
    let module_str = "(module
      (memory (;0;) 1 256))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_3(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 5

#[test]
fn test_module_3() {
    let result_object = create_module_3();
    // We group the calls together
    start_module_3(&result_object);
}
fn create_module_4() -> ResultObject {
    let module_str = "(module
      (memory (;0;) 0 65536))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_4(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 7
#[test]
fn c4_l7_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 5, 2, 0, 0, 0, 0];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 8
#[test]
fn c5_l8_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 2, 20, 1, 8, 115, 112, 101, 99, 116, 101, 115, 116, 6, 109, 101, 109, 111, 114, 121, 2, 0, 0, 5, 3, 1, 0, 0];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 10

#[test]
fn test_module_4() {
    let result_object = create_module_4();
    // We group the calls together
    start_module_4(&result_object);
}
fn create_module_5() -> ResultObject {
    let module_str = "(module
      (type (;0;) (func (result i32)))
      (func (;0;) (type 0) (result i32)
        memory.size)
      (memory (;0;) 0 0)
      (export \"memsize\" (func 0))
      (data (;0;) (i32.const 0) \"\"))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_5(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 11
fn c7_l11_action_invoke(result_object: &ResultObject) {
    println!("Executing function {}", "c7_l11_action_invoke");
    let func_index = match result_object.module.info.exports.get("memsize") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&Instance) -> i32 = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&result_object.instance);
    assert_eq!(result, 0 as i32);
}

// Line 12

#[test]
fn test_module_5() {
    let result_object = create_module_5();
    // We group the calls together
    start_module_5(&result_object);
    c7_l11_action_invoke(&result_object);
}
fn create_module_6() -> ResultObject {
    let module_str = "(module
      (type (;0;) (func (result i32)))
      (func (;0;) (type 0) (result i32)
        memory.size)
      (memory (;0;) 0 0)
      (export \"memsize\" (func 0))
      (data (;0;) (i32.const 0) \"\"))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_6(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 13
fn c9_l13_action_invoke(result_object: &ResultObject) {
    println!("Executing function {}", "c9_l13_action_invoke");
    let func_index = match result_object.module.info.exports.get("memsize") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&Instance) -> i32 = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&result_object.instance);
    assert_eq!(result, 0 as i32);
}

// Line 14

#[test]
fn test_module_6() {
    let result_object = create_module_6();
    // We group the calls together
    start_module_6(&result_object);
    c9_l13_action_invoke(&result_object);
}
fn create_module_7() -> ResultObject {
    let module_str = "(module
      (type (;0;) (func (result i32)))
      (func (;0;) (type 0) (result i32)
        memory.size)
      (memory (;0;) 1 1)
      (export \"memsize\" (func 0))
      (data (;0;) (i32.const 0) \"x\"))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(wasm_binary, spectest_importobject()).expect("WASM can't be instantiated")
}

fn start_module_7(result_object: &ResultObject) {
    result_object.instance.start();
}

// Line 15
fn c11_l15_action_invoke(result_object: &ResultObject) {
    println!("Executing function {}", "c11_l15_action_invoke");
    let func_index = match result_object.module.info.exports.get("memsize") {
        Some(&Export::Function(index)) => index,
        _ => panic!("Function not found"),
    };
    let invoke_fn: fn(&Instance) -> i32 = get_instance_function!(result_object.instance, func_index);
    let result = invoke_fn(&result_object.instance);
    assert_eq!(result, 1 as i32);
}

// Line 17
#[test]
fn c12_l17_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 11, 6, 1, 0, 65, 0, 11, 0];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 18
#[test]
fn c13_l18_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 11, 6, 1, 0, 65, 0, 11, 0];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 19
#[test]
fn c14_l19_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 11, 7, 1, 0, 65, 0, 11, 1, 120];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 22
#[test]
fn c15_l22_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0, 42, 2, 0, 26, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 26
#[test]
fn c16_l26_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 14, 1, 12, 0, 67, 0, 0, 0, 0, 65, 0, 56, 2, 0, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 30
#[test]
fn c17_l30_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0, 44, 0, 0, 26, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 34
#[test]
fn c18_l34_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 11, 1, 9, 0, 65, 0, 65, 0, 58, 0, 0, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 38
#[test]
fn c19_l38_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 7, 1, 5, 0, 63, 0, 26, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 42
#[test]
fn c20_l42_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 9, 1, 7, 0, 65, 0, 64, 0, 26, 11];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 48
#[test]
fn c21_l48_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 4, 1, 1, 1, 0];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 52
#[test]
fn c22_l52_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 5, 1, 0, 129, 128, 4];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 56
#[test]
fn c23_l56_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 7, 1, 0, 128, 128, 128, 128, 8];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 60
#[test]
fn c24_l60_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 7, 1, 0, 255, 255, 255, 255, 15];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 64
#[test]
fn c25_l64_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 6, 1, 1, 0, 129, 128, 4];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 68
#[test]
fn c26_l68_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 8, 1, 1, 0, 128, 128, 128, 128, 8];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

// Line 72
#[test]
fn c27_l72_assert_invalid() {
    let wasm_binary = [0, 97, 115, 109, 1, 0, 0, 0, 5, 8, 1, 1, 0, 255, 255, 255, 255, 15];
    let compilation = compile(wasm_binary.to_vec());
    assert!(compilation.is_err(), "WASM should not compile as is invalid");
}

#[test]
fn test_module_7() {
    let result_object = create_module_7();
    // We group the calls together
    start_module_7(&result_object);
    c11_l15_action_invoke(&result_object);
}
