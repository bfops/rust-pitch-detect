use std;
use time;
use env_logger;
use portaudio;
use rgsl;

fn scale_step() -> f64 {
  (2.0 as f64).powf(1.0 / 12.0)
}

fn scale_steps(f: f64) -> i32 {
  let ratio = f / 440.0;
  let steps = ratio.log(scale_step());
  steps.round() as i32
}

fn string_err<Ok, Err: std::fmt::Display>(r: Result<Ok, Err>) -> Result<Ok, String> {
  r.map_err(|err| format!("{}", err))
}

fn sine_wave(sample_frequency: f32, start: u32, end: u32) -> Vec<f32> {
  let mut buf = Vec::new();

  let semitones_above_a = 1;

  let f = (440.0 * scale_step().powf(semitones_above_a as f64)) as f32;

  for t in start .. end {
    let t = t as f32 / sample_frequency;
    let sample = (t * f * 2.0 * std::f32::consts::PI).sin();
    let amplitude = 1.0;
    buf.push(sample * amplitude);
  }

  buf
}

fn play_pitch() -> Result<(), String> {
  info!("Writing..");

  let f = 44100;
  let buf_size = 1 << 10;

  let mut stream: portaudio::pa::Stream<f32, f32> = portaudio::pa::Stream::new();
  try!(string_err(
    stream.open_default_blocking(
      f as f64,
      buf_size,
      0,
      1,
      portaudio::pa::SampleFormat::Float32,
    )));
  try!(string_err(stream.start()));

  let start_time = time::precise_time_ns();
  let mut i = 0;
  while time::precise_time_ns() <= start_time + 4_000_000_000 {
    let buf = sine_wave(f as f32, i*buf_size, (i+1)*buf_size);
    assert!(buf.len() == buf_size as usize);
    try!(string_err(stream.write(buf, buf_size)));

    i = i + 1;
  }

  try!(string_err(stream.stop()));

  Ok(())
}

fn display_note(note: u32) -> &'static str {
  let notes = [
    "A",
    "A#/Bb",
    "B",
    "C",
    "C#/Db",
    "D",
    "D#/Eb",
    "E",
    "F",
    "F#/Gb",
    "G",
    "G#/Ab",
  ];
  notes[note as usize]
}

fn record(sample_frequency: f64) -> Result<Vec<f64>, String> {
  let buf_size = 1 << 10;
  assert!(buf_size & (buf_size - 1) == 0);

  info!("Reading..");

  let mut stream: portaudio::pa::Stream<f32, f32> = portaudio::pa::Stream::new();
  try!(string_err(
    stream.open_default_blocking(
      sample_frequency,
      buf_size,
      1,
      0,
      portaudio::pa::SampleFormat::Float32,
    )));
  try!(string_err(stream.start()));

  let mut buf = Vec::new();
  let start_time = time::precise_time_ns();
  while time::precise_time_ns() <= start_time + 2_000_000_000 || (buf.len() & (buf.len() - 1)) != 0 {
    let new = try!(string_err(stream.read(buf_size)));
    assert!(new.len () == buf_size as usize);
    buf.extend(new.into_iter().map(|f| f as f64));
  }

  try!(string_err(stream.stop()));

  Ok(buf)
}

fn detect_frequency(mut samples: Vec<f64>, timestep: f64) -> Result<f64, String> {
  let len = samples.len();
  let r = rgsl::fft::real_radix2::transform(&mut samples, 1, len);
  if r != rgsl::Value::Success {
    return Err(format!("rgsl returned {:?}", r));
  }

  let (max_idx, _max_val) =
    samples.iter()
      .enumerate()
      .take(len / 2)
      .fold((0, 0.0), |(max_idx, max_val), (i, val)| {
        if *val > max_val {
          (i, *val)
        } else {
          (max_idx, max_val)
        }
      });

  debug!("Max index is {}", max_idx);
  Ok(max_idx as f64 / samples.len() as f64 / timestep)
}

fn human_readable_frequency(f: f64) -> String {
  let scale_steps = scale_steps(f);

  info!("Estimated frequency is {} ({} semitones above middle A)", f, scale_steps);

  let octave = {
    // Octave is decided by the Cs, so make scale_steps relative to C4.
    let scale_steps = scale_steps + 9;
    4 + scale_steps / 12
  };
  let note = {
    let note = scale_steps % 12;
    let note = if note >= 0 { note } else { note + 12 };
    assert!(note >= 0);
    assert!(note < 12);
    note as u32
  };

  let note = display_note(note);

  if octave == 4 {
    format!("middle {} ({}{})", note, note, octave)
  } else {
    format!("{}{}", note, octave)
  }
}

fn detect_pitch() -> Result<(), String> {
  let sample_frequency = 44100.0;
  let samples = try!(record(sample_frequency));

  info!("Collected {} samples", samples.len());

  info!("Detecting pitch..");

  let f = try!(detect_frequency(samples, 1.0 / sample_frequency));

  println!("Estimated note is {}", human_readable_frequency(f));

  Ok(())
}

fn errorful_main() -> Result<(), String> {
  try!(string_err(env_logger::init()));
  try!(string_err(portaudio::pa::initialize()));

  try!(detect_pitch());

  Ok(())
}

#[main]
fn main() {
  errorful_main().unwrap();
}
