relib_interface::include_exports!();
relib_interface::include_imports!();

impl shared::imports::Imports for gen_imports::ModuleImportsImpl {
  fn foo() -> u8 {
    10
  }
}

fn main() {
  let path_to_dylib = if cfg!(target_os = "linux") {
    "target/debug/libmodule.so"
  } else {
    "target/debug/module.dll"
  };

  let module = unsafe {
    relib_host::load_module::<gen_exports::ModuleExports>(path_to_dylib, gen_imports::init_imports)
  };
  let module = module.unwrap_or_else(|e| {
    panic!("module loading failed: {e:#}");
  });

  // main function is unsafe to call (as well as any other module export) because these preconditions are not checked by relib:
  // 1. returned value must be actually `R` at runtime. For example if you called this function with type bool but module returns i32, UB will occur.
  // 2. type of return value must be FFI-safe.
  // 3. returned value must not be a reference-counting pointer (see caveats on main docs page/README).
  let returned_value = unsafe { module.call_main::<()>() };

  // if module panics while executing any export it returns None
  // (panic will be printed by module)
  if returned_value.is_none() {
    println!("module panicked");
  }

  // both imports and exports are unsafe to call since these preconditions are not checked by relib:
  // 1. types of arguments and return value must be FFI-safe
  //    (you can use abi_stable or stabby crate for it, see "abi_stable_usage" example).
  // 2. host and module crates must be compiled with same shared crate code.
  // 3. returned value must not be a reference-counting pointer (see caveats on main docs page/README).
  let bar_value = unsafe { module.exports().bar() }.unwrap();
  dbg!(bar_value);

  // module.unload() is provided when unloading feature of relib_host crate is enabled
  #[cfg(feature = "unloading")]
  {
    println!("unloading feature is enabled, calling module unload");

    module.unload().unwrap_or_else(|e| {
      panic!("module unloading failed: {e:#}");
    });
  }
}
