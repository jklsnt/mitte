//! mitte
//!
//! Facilities for UDP-based secure message transfer.
//!
//! (c) 2021 Houjun Liu and Huxley Marvit.
//! Submitted as the final project to CS240

// Error handling
pub mod error;

// Agent information: i.e. most of the library
mod agent;

// We publish every public function in agents
pub use agent::*;

// As most of the actual code is in [`agent`], we will
// leverage this module to write usage examples and
// unit tests.

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        println!("hewo");
    }
}

