#![feature(main)]
#![feature(plugin)]

#![plugin(clippy)]
#![allow(non_snake_case)]

extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate portaudio;
extern crate rgsl;

mod main;
