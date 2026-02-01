# Recent Changes

## 0.3.0

- `riri-mod-tools`: 
  - Add automatic bindgen between C# classes and Rust structs, designed to call Reloaded API interfaces
  from Rust.
  - Pointer and reference types in function parameters are treated as nint in C# to prevent unknown type errors
  - Write a README describing the crate's usage
- `riri-mod-tools-rt`: 
  - Implement ProcessInfo for Linux and OS-agnostic page protection function (`PageProtection::change_protection`)
  - Add callback for Reloaded Logger API's OnWrite/OnWriteLine events

## 0.2.7

- `riri-mod-runtime`: Updated build script and added CI support to automatically create and publish Reloaded mod on release
- `riri-mod-tools`: Added NuGet update support into mod config