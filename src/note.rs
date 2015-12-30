#![allow(unused)]

use std;

fn string_err<Ok, Err: std::fmt::Display>(r: Result<Ok, Err>) -> Result<Ok, String> {
  r.map_err(|err| format!("{}", err))
}

mod scale_step {
  use std;

  pub fn to_str(scale_step: u32) -> &'static str {
    let step = [
      "C",
      "C#/Db",
      "D",
      "D#/Eb",
      "E",
      "F",
      "F#/Gb",
      "G",
      "G#/Ab",
      "A",
      "A#/Bb",
      "B",
    ];
    step[scale_step as usize]
  }

  pub fn of_str(s: &mut std::iter::Peekable<std::str::Chars>) -> Result<i32, String> {
    let diatonic =
      match s.next() {
        Some('C') => 0,
        Some('D') => 2,
        Some('E') => 4,
        Some('F') => 5,
        Some('G') => 7,
        Some('A') => 9,
        Some('B') => 11,
        _ => return Err("Unrecognized note name".to_owned()),
      };

    let accidental =
      match s.peek() {
        Some(&'#') => {
          s.skip(1);
          1
        },
        Some(&'b') => {
          s.skip(1);
          -1
        },
        _ => {
          0
        },
      };

    Ok(diatonic + accidental)
  }
}

#[derive(Clone, Copy)]
pub struct T {
  steps_above_middle_c: i32,
}

impl T {
  pub fn sharp(mut self) -> T {
    self.steps_above_middle_c = self.steps_above_middle_c + 1;
    self
  }

  pub fn flat(mut self) -> T {
    self.steps_above_middle_c = self.steps_above_middle_c - 1;
    self
  }

  pub fn to_string_human(&self) -> String {
    let scale_steps = self.steps_above_middle_c;

    let semitones_as_string = {
      if scale_steps >= 0 {
        format!("{} semitones above middle C", scale_steps)
      } else {
        format!("{} semitones below middle C", -scale_steps)
      }
    };
    debug!("Pitch is {}", semitones_as_string);

    let octave = {
      let scale_steps = if scale_steps >= 0 { scale_steps } else { scale_steps - 12 };
      4 + scale_steps / 12
    };
    let note = {
      let note = scale_steps % 12;
      let note = if note >= 0 { note } else { note + 12 };
      assert!(note >= 0);
      assert!(note < 12);
      note as u32
    };

    let note = scale_step::to_str(note);

    if octave == 4 {
      format!("middle {} ({}{})", note, note, octave)
    } else {
      format!("{}{}", note, octave)
    }
  }

  pub fn to_frequency(&self) -> f64 {
    middle_A() * scale_step().powf((self.steps_above_middle_c - 9) as f64)
  }
}

fn make(octave: i32, step: i32) -> T {
  T {
    steps_above_middle_c: (octave - 4) * 12 + step,
  }
}

pub fn c(octave: i32) -> T { make(octave, 0) }
pub fn d(octave: i32) -> T { make(octave, 2) }
pub fn e(octave: i32) -> T { make(octave, 4) }
pub fn f(octave: i32) -> T { make(octave, 5) }
pub fn g(octave: i32) -> T { make(octave, 7) }
pub fn a(octave: i32) -> T { make(octave, 9) }
pub fn b(octave: i32) -> T { make(octave, 11) }

pub mod middle {
  use super::T;

  pub fn c(octave: i32) -> T { super::c(4) }
  pub fn d(octave: i32) -> T { super::d(4) }
  pub fn e(octave: i32) -> T { super::e(4) }
  pub fn f(octave: i32) -> T { super::f(4) }
  pub fn g(octave: i32) -> T { super::g(4) }
  pub fn a(octave: i32) -> T { super::a(4) }
  pub fn b(octave: i32) -> T { super::b(4) }
}

fn scale_step() -> f64 {
  (2.0 as f64).powf(1.0 / 12.0)
}

fn middle_A() -> f64 {
  440.0
}

pub fn of_frequency(f: f64) -> T {
  let ratio = f / middle_A();
  let steps = ratio.log(scale_step());
  let steps = steps.round() + 9.0;
  T {
    steps_above_middle_c: steps as i32,
  }
}

pub fn from_str(s: &str) -> Result<T, String> {
  let mut s = s.chars().peekable();
  let scale_step = try!(scale_step::of_str(&mut s));
  let s: String = s.collect();
  let octave = try!(string_err(s.parse()));
  Ok(make(octave, scale_step))
}
