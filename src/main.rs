use std;
use time;
use env_logger;
use portaudio;

fn string_err<Ok, Err: std::fmt::Display>(r: Result<Ok, Err>) -> Result<Ok, String> {
  r.map_err(|err| format!("{}", err))
}

fn sine_wav(f: f32, start: u32, end: u32) -> Vec<f32> {
  let mut buf = Vec::new();

  for t in start .. end {
    let t = t as f32 / f;
    let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
    let amplitude = 1.0;
    buf.push(sample * amplitude);
  }

  buf
}

fn errorful_main() -> Result<(), String> {
  try!(string_err(env_logger::init()));

  try!(string_err(portaudio::pa::initialize()));

  let f = 44100;
  let buf_size = 1 << 10;

  info!("Reading..");

  let mut stream: portaudio::pa::Stream<f32, f32> = portaudio::pa::Stream::new();
  try!(string_err(
    stream.open_default_blocking(
      f as f64,
      buf_size,
      1,
      0,
      portaudio::pa::SampleFormat::Float32,
    )));
  try!(string_err(stream.start()));

  let mut buf = std::collections::vec_deque::VecDeque::new();
  let start_time = time::precise_time_ns();
  while time::precise_time_ns() <= start_time + 2_000_000_000 {
    let new = try!(string_err(stream.read(buf_size)));
    assert!(new.len () == buf_size as usize);
    buf.push_back(new);
  }

  try!(string_err(stream.stop()));

  info!("Writing..");

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

  while let Some(buf) = buf.pop_front() {
    assert!(buf.len() == buf_size as usize);
    try!(string_err(stream.write(buf, buf_size)));
  }

  try!(string_err(stream.stop()));

  Ok(())
}

#[main]
fn main() {
  errorful_main().unwrap();
}
