pub(crate) mod client;
pub(crate) mod controller;
pub(crate) mod domain;
pub(crate) mod owner;
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
pub(crate) mod peer;
pub(crate) mod persistence;
pub(crate) mod presentation;
pub(crate) mod protocol;
pub(crate) mod replay;
pub(crate) mod runtime;
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
pub(crate) mod transport;

#[cfg(test)]
#[path = "../t158_persistent_runtime_adversarial_tests.rs"]
mod t158_persistent_runtime_adversarial_tests;

#[cfg(test)]
#[path = "../t159_persistent_runtime_performance_tests.rs"]
mod t159_persistent_runtime_performance_tests;
