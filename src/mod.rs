#![feature(main)]
#![feature(stmt_expr_attributes)]
#![feature(plugin)]

#![plugin(clippy)]
#![allow(non_snake_case)]
#![allow(let_and_return)]
#![deny(warnings)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate thread_scoped;
extern crate time;
#[macro_use]
extern crate clap;
extern crate portaudio;
extern crate rgsl;
#[cfg(feature="gnuplot")]
extern crate gnuplot;

mod mvar;
mod note;
mod main;

mod thread {
  pub use std::thread::*;
  pub use thread_scoped::*;
}
