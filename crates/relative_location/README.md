# cloud_terrastodon_relative_location

A helper for converting std::panic::Location to relative paths with displayed.

When crossing crate boundaries calling a function annotated with 

```rust
#[track_caller]
```

the value returned by 

```rust
std::panic::Location::caller()
```

uses an absolute path.

This crate strips the path segments that are also present in the path of this crate at build time.

This means it probably won't work outside of building Cloud Terrastodon 

ㄟ( ▔, ▔ )ㄏ