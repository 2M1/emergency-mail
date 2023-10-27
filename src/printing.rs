pub mod com;
pub mod document;
pub mod pdf;
pub mod print_ems;

#[cfg(feature = "xps")]
pub mod xps;

#[cfg(all(feature = "xps", not(target_os = "windows")))]
compile_error!("the xps feature requires windows as a target os!");

#[cfg(test)]
pub mod print_ems_tests;
