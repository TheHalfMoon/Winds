fn main() -> std::process::ExitCode {
    winds_control::cli_main()
}

// Preserve the historical `cargo test --bin winds` test surface without
// duplicating production authority. The library source is compiled into the
// binary crate only under `cfg(test)`; release/production builds use only the
// library entry above.
#[cfg(test)]
include!("lib.rs");
