#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod Constants;
pub mod bettingTrait;
pub mod contract;
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;
pub use contract::*;

#[cfg(test)]
mod test;
