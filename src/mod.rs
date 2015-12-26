#![feature(main)]
#![feature(plugin)]

#![plugin(clippy)]

extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate portaudio;
extern crate rgsl;

mod main;
