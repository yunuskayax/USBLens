//! Platform backend. Windows gets the real implementation; other hosts get
//! sample data so the UI can be built and tested anywhere.
//! `RUSTFLAGS="--cfg usblens_check"` type-checks the Windows code on any host.

#[cfg(any(windows, usblens_check))]
mod win;
#[cfg(any(windows, usblens_check))]
pub use win::*;

#[cfg(not(any(windows, usblens_check)))]
mod mock;
#[cfg(not(any(windows, usblens_check)))]
pub use mock::*;
