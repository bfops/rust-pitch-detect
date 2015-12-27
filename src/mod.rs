#![feature(main)]
#![feature(plugin)]

#![plugin(clippy)]
#![allow(non_snake_case)]

extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate thread_scoped;
extern crate portaudio;
extern crate rgsl;

mod mvar;
mod main;

mod thread {
  pub use std::thread::*;
  pub use thread_scoped::*;
}
