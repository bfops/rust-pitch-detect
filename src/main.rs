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

//  info!("Reading..");
//
//  let mut stream: portaudio::pa::Stream<f32, f32> = portaudio::pa::Stream::new();
//  try!(string_err(
//    stream.open_default_blocking(
//      44100.0,
//      64,
//      2,
//      0,
//      portaudio::pa::SampleFormat::Float32,
//    )));
//  try!(string_err(stream.start()));
//  let mut buf = Vec::new();
//  while buf.len() < 1000000 {
//    let mut new = try!(string_err(stream.read(64)));
//    buf.append(&mut new);
//  }
//
//  try!(string_err(stream.stop()));

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

  let start_time = time::precise_time_ns();
  let mut i = 0;
  while time::precise_time_ns() <= start_time + 2_000_000_000 {
    let buf = sine_wav(f as f32, i*buf_size, (i+1)*buf_size);
    assert!(buf.len() == buf_size as usize);
    try!(string_err(stream.write(buf, buf_size)));

    i = i + 1;
  }

  try!(string_err(stream.stop()));

  Ok(())
}

#[main]
fn main() {
  errorful_main().unwrap();
}
