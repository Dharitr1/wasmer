var N = null;var sourcesIndex = {};
sourcesIndex["generate_emscripten_tests"] = {"name":"","files":["emtests.rs","lib.rs"]};
sourcesIndex["generate_wasi_tests"] = {"name":"","files":["lib.rs","set_up_toolchain.rs","util.rs","wasi_version.rs","wasitests.rs"]};
sourcesIndex["kernel_net"] = {"name":"","files":["lib.rs"]};
sourcesIndex["kwasmd"] = {"name":"","files":["kwasmd.rs"]};
sourcesIndex["parallel"] = {"name":"","files":["main.rs"]};
sourcesIndex["parallel_guest"] = {"name":"","files":["main.rs"]};
sourcesIndex["plugin_for_example"] = {"name":"","files":["main.rs"]};
sourcesIndex["wasmer"] = {"name":"","files":["wasmer.rs"]};
sourcesIndex["wasmer_bin"] = {"name":"","files":["lib.rs","update.rs","utils.rs","webassembly.rs"]};
sourcesIndex["wasmer_clif_backend"] = {"name":"","dirs":[{"name":"signal","files":["mod.rs","unix.rs"]}],"files":["cache.rs","code.rs","lib.rs","libcalls.rs","module.rs","relocation.rs","resolver.rs","trampoline.rs"]};
sourcesIndex["wasmer_dev_utils"] = {"name":"","files":["file_descriptor.rs","lib.rs","stdio.rs"]};
sourcesIndex["wasmer_emscripten"] = {"name":"","dirs":[{"name":"env","dirs":[{"name":"unix","files":["mod.rs"]}],"files":["mod.rs"]},{"name":"io","files":["mod.rs","unix.rs"]},{"name":"syscalls","files":["mod.rs","unix.rs"]}],"files":["bitwise.rs","emscripten_target.rs","errno.rs","exception.rs","exec.rs","exit.rs","inet.rs","jmp.rs","lib.rs","libc.rs","linking.rs","lock.rs","macros.rs","math.rs","memory.rs","process.rs","pthread.rs","ptr.rs","signal.rs","storage.rs","time.rs","ucontext.rs","unistd.rs","utils.rs","varargs.rs"]};
sourcesIndex["wasmer_interface_types"] = {"name":"","dirs":[{"name":"decoders","files":["binary.rs","mod.rs","wat.rs"]},{"name":"encoders","files":["binary.rs","mod.rs","wat.rs"]},{"name":"interpreter","dirs":[{"name":"instructions","files":["argument_get.rs","call_core.rs","mod.rs","numbers.rs","records.rs","strings.rs"]},{"name":"wasm","dirs":[{"name":"serde","files":["de.rs","mod.rs","ser.rs"]}],"files":["mod.rs","structures.rs","values.rs"]}],"files":["mod.rs","stack.rs"]}],"files":["ast.rs","errors.rs","lib.rs","macros.rs","vec1.rs"]};
sourcesIndex["wasmer_kernel_loader"] = {"name":"","files":["lib.rs","service.rs"]};
sourcesIndex["wasmer_llvm_backend"] = {"name":"","dirs":[{"name":"platform","files":["common.rs","mod.rs","unix.rs"]}],"files":["backend.rs","code.rs","intrinsics.rs","lib.rs","read_info.rs","stackmap.rs","state.rs","structs.rs","trampolines.rs"]};
sourcesIndex["wasmer_middleware_common"] = {"name":"","files":["block_trace.rs","call_trace.rs","lib.rs","metering.rs"]};
sourcesIndex["wasmer_runtime"] = {"name":"","files":["cache.rs","lib.rs"]};
sourcesIndex["wasmer_runtime_c_api"] = {"name":"","dirs":[{"name":"import","files":["mod.rs","wasi.rs"]}],"files":["error.rs","export.rs","global.rs","instance.rs","lib.rs","memory.rs","module.rs","table.rs","trampoline.rs","value.rs"]};
sourcesIndex["wasmer_runtime_core"] = {"name":"","dirs":[{"name":"memory","files":["dynamic.rs","mod.rs","ptr.rs","static_.rs","view.rs"]},{"name":"structures","files":["boxed.rs","map.rs","mod.rs","slice.rs"]},{"name":"sys","dirs":[{"name":"unix","files":["memory.rs","mod.rs"]}],"files":["mod.rs"]},{"name":"table","files":["anyfunc.rs","mod.rs"]}],"files":["backend.rs","backing.rs","cache.rs","codegen.rs","error.rs","export.rs","fault.rs","global.rs","import.rs","instance.rs","jit_debug.rs","lib.rs","loader.rs","macros.rs","module.rs","parse.rs","sig_registry.rs","state.rs","tiering.rs","trampoline_x64.rs","typed_func.rs","types.rs","units.rs","vm.rs","vmcalls.rs"]};
sourcesIndex["wasmer_singlepass_backend"] = {"name":"","files":["codegen_x64.rs","emitter_x64.rs","lib.rs","machine.rs"]};
sourcesIndex["wasmer_wasi"] = {"name":"","dirs":[{"name":"state","files":["builder.rs","mod.rs","types.rs"]},{"name":"syscalls","dirs":[{"name":"legacy","files":["mod.rs","snapshot0.rs"]},{"name":"unix","files":["mod.rs"]}],"files":["mod.rs","types.rs"]}],"files":["lib.rs","macros.rs","ptr.rs","utils.rs"]};
sourcesIndex["wasmer_wasi_experimental_io_devices"] = {"name":"","files":["lib.rs","util.rs"]};
sourcesIndex["wasmer_win_exception_handler"] = {"name":"","files":["lib.rs"]};
createSourceSidebar();
