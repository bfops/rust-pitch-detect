use std;
use time;
use env_logger;
use portaudio;
use rgsl;

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
  info!("Estimated frequency is {}", max_idx as f32 / buf.len() as f32 * f as f32);

  Ok(())
}

#[main]
fn main() {
  errorful_main().unwrap();
}
