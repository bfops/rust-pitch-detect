use std;
use time;
use env_logger;
use portaudio;
use rgsl;

fn scale_step() -> f64 {
  (2.0 as f64).powf(1.0 / 12.0)
}

fn scale_steps(f: f64) -> u32 {
  let ratio = f / 440.0;
  let steps = ratio.log(scale_step());
  steps.round() as u32
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

fn detect_pitch() -> Result<(), String> {
  let f = 44100;
  let buf_size = 1 << 10;
  assert!(buf_size & (buf_size - 1) == 0);

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

  let mut buf = Vec::new();
  let start_time = time::precise_time_ns();
  while time::precise_time_ns() <= start_time + 2_000_000_000 || (buf.len() & (buf.len() - 1)) != 0 {
    let new = try!(string_err(stream.read(buf_size)));
    assert!(new.len () == buf_size as usize);
    buf.extend(new.into_iter().map(|f| f as f64));
  }

  try!(string_err(stream.stop()));

  info!("Collected {} samples", buf.len());

  info!("Pitch detecting..");

  let len = buf.len();
  let r = rgsl::fft::real_radix2::transform(&mut buf, 1, len);
  if r != rgsl::Value::Success {
    return Err(format!("rgsl returned {:?}", r));
  }

  let (max_idx, _max_val) =
    buf.iter()
      .enumerate()
      .take(len / 2)
      .fold((0, 0.0), |(max_idx, max_val), (i, val)| {
        if *val > max_val {
          (i, *val)
        } else {
          (max_idx, max_val)
        }
      });

  info!("Max index is {}", max_idx);
  let f = max_idx as f64 / buf.len() as f64 * f as f64;
  println!("Estimated frequency is {}", f);
  println!("{} semitones away from middle A", scale_steps(f));

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
