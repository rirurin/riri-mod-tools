# Recent Changes

## 0.3.1

- `riri-mod-tools`:
  - Added the `riri_mod_loading_fn` attribute which can be attached onto a function with an `IModConfig` parameter to get
  called when Reloaded-II loads another mod.

- `riri-mod-tools-rt`:
  - Added the `Array<T>` system object to interop with C# arrays.
  - Added most of the missing methods from `IModConfig` in C# interop.

## 0.3.0

- `riri-mod-tools`: 
  - Add automatic bindgen between C# classes and Rust structs, designed to call Reloaded API interfaces
  from Rust.
  - Pointer and reference types in function parameters are treated as nint in C# to prevent unknown type errors
  - Write a README describing the crate's usage
- `riri-mod-tools-rt`: 
  - Implement ProcessInfo for Linux and OS-agnostic page protection function (`PageProtection::change_protection`)
  - Add callback for Reloaded Logger API's OnWrite/OnWriteLine events