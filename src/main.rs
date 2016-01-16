use std;
use env_logger;
use time;
use portaudio;
use rgsl;
#[cfg(feature="gnuplot")]
use gnuplot;

use thread;
use note;
use mvar;

fn consume<T>(_: T) {}

fn to_fft(mut samples: Vec<f64>) -> Result<Vec<f64>, String> {
  let len = samples.len();
  let r = rgsl::fft::real_radix2::transform(&mut samples, 1, len);
  if r != rgsl::Value::Success {
    return Err(format!("rgsl returned {:?}", r));
  }

  Ok(samples)
}

fn of_fft(mut fft: Vec<f64>) -> Result<Vec<f64>, String> {
  let len = fft.len();
  let r = rgsl::fft::real_radix2::inverse(&mut fft, 1, len);
  if r != rgsl::Value::Success {
    return Err(format!("rgsl returned {:?}", r));
  }

  Ok(fft)
}

fn string_err<Ok, Err: std::fmt::Display>(r: Result<Ok, Err>) -> Result<Ok, String> {
  r.map_err(|err| format!("{}", err))
}

fn sine_wave(note: note::T, sample_frequency: f64, start: u32, end: u32) -> Vec<f32> {
  let mut buf = Vec::new();

  let f = note.to_frequency();

  for t in start .. end {
    let t = t as f64 / sample_frequency;
    let sample = (t * f * 2.0 * std::f64::consts::PI).sin();
    let amplitude = 1.0;
    buf.push((sample * amplitude) as f32);
  }

  buf
}

fn with_play_channel<T, F>(sample_rate: f64, buf_size: u32, f: F) -> Result<T, String> where
  F: FnOnce(&mut portaudio::pa::Stream<f32, f32>) -> Result<T, String>,
{
  let mut stream: portaudio::pa::Stream<f32, f32> = portaudio::pa::Stream::new();
  try!(string_err(
    stream.open_default_blocking(
      sample_rate,
      buf_size,
      0,
      1,
      portaudio::pa::SampleFormat::Float32,
    )));
  try!(string_err(stream.start()));

  let r = try!(f(&mut stream));

  try!(string_err(stream.stop()));

  Ok(r)
}

fn play_note(note: note::T, secs: u64) -> Result<(), String> {
  info!("Writing..");

  let sample_rate = 44100 as f64;
  let buf_size = 1 << 10;

  try!(with_play_channel(
    sample_rate,
    buf_size,
    |stream| {
      let start_time = time::precise_time_ns();
      let mut i = 0;
      while time::precise_time_ns() <= start_time + secs*1_000_000_000 {
        let buf = sine_wave(note, sample_rate, i*buf_size, (i+1)*buf_size);
        assert!(buf.len() == buf_size as usize);
        try!(string_err(stream.write(buf, buf_size)));

        i = i + 1;
      }

      Ok(())
    }
  ));

  Ok(())
}

fn record(sample_frequency: f64, delta_t_ns: u64) -> Result<Vec<f64>, String> {
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
  while time::precise_time_ns() <= start_time + delta_t_ns || (buf.len() & (buf.len() - 1)) != 0 {
    let new = try!(string_err(stream.read(buf_size)));
    assert!(new.len () == buf_size as usize);
    buf.extend(new.into_iter().map(|f| f as f64));
  }

  try!(string_err(stream.stop()));

  Ok(buf)
}

fn detect_frequency(samples: Vec<f64>, timestep: f64) -> Result<Option<f64>, String> {
  let num_samples = samples.len();
  let samples = try!(to_fft(samples));

  let buckets: Vec<_> =
    (0 .. 1 + num_samples / 2)
    .filter_map(|i| {
      let real = samples[i];
      let imag =
        if i % num_samples / 2 == 0 {
          0.0
        } else {
          samples[num_samples - i]
        };

      let noise_threshold = 0.0;
      let x = real*real + imag*imag - noise_threshold;
      if x > noise_threshold {
        let f = i as f64 / num_samples as f64 / timestep;
        Some((f, x))
      } else {
        None
      }
    })
    .collect();
  consume(samples);

  if buckets.is_empty() {
    return Ok(None)
  }

  #[cfg(feature="gnuplot")]
  {
    let x: Vec<_> = buckets.iter().map(|&(f, _)| f).collect();
    let y: Vec<_> = buckets.iter().map(|&(_, y)| y).collect();
    let mut fg = gnuplot::Figure::new();
    fg
      .axes2d()
      .boxes(
        &x,
        &y,
        &[],
      );
    fg.echo_to_file("/tmp/gnuplot.txt");
  }

  let (max_f, max_val) =
    buckets.iter()
      .fold((0.0, 0.0), |(max_f, max_val), &(f, amp)| {
        let val = amp;
        if val > max_val {
          (f, val)
        } else {
          (max_f, max_val)
        }
      });

  debug!("Max index is {}", max_f);
  debug!("Max value is {}", max_val);

  Ok(Some(max_f))
}

fn harmony(fft: &[f64], semitones_up: f64) -> Vec<f64> {
  let num_samples = fft.len();

  let shift_factor: f64 = (2.0 as f64).powf(-semitones_up / 12.0);

  let mut shifted_fft: Vec<_> = std::iter::repeat(0.0).take(num_samples).collect();
  for i in 1..(num_samples / 2) {
    let source_i = (i as f64 * shift_factor).round() as usize;
    shifted_fft[i] = shifted_fft[i] + fft[source_i];
    shifted_fft[num_samples - i] = shifted_fft[num_samples - i] + fft[num_samples - source_i];
  }

  // This might be broken for nonzero shifts?
  shifted_fft[0] = fft[0];
  shifted_fft[num_samples / 2] = fft[num_samples / 2];

  shifted_fft
}

fn detect_pitch_main() -> Result<(), String> {
  let sample_frequency = 44100.0;
  let samples = mvar::new();

  let _record_thread =
    unsafe {
      thread::scoped(|| {
        loop {
          let new_samples = record(sample_frequency, 100_000_000).unwrap();
          info!("Collected {} samples", new_samples.len());
          samples.overwrite(new_samples);
        }
      })
    };

  let _detect_thread =
    unsafe {
      thread::scoped(|| {
        info!("Detecting pitch..");

        loop {
          let samples = samples.take();
          match detect_frequency(samples, 1.0 / sample_frequency).unwrap() {
            None => {},
            Some(f) => {
              debug!("Estimated frequency is {}", f);
              let note = note::of_frequency(f);
              println!("Estimated note is {}", note.to_string_human());
            },
          }
        }
      })
    };

  Ok(())
}

fn harmony_main(semitones: Vec<f64>) -> Result<(), String> {
  let sample_frequency = 44100.0;

  let samples = record(sample_frequency, 1_000_000_000).unwrap();
  info!("Collected {} samples", samples.len());

  let len = samples.len();
  let original_fft = try!(to_fft(samples));
  let mut fft = original_fft.clone();

  for &semitones in &semitones {
    let harmony = harmony(&original_fft, semitones);
    for i in 0 .. len {
      fft[i] += harmony[i];
    }
  }

  let fft = fft.into_iter().map(|f| f / semitones.len() as f64).collect();

  let samples = try!(of_fft(fft));

  let mut samples = samples.into_iter().map(|f| f as f32);

  let sample_rate = 44100 as f64;
  let buf_size = 1 << 10;

  try!(with_play_channel(
    sample_rate,
    buf_size,
    |stream| {
      loop {
        let mut snip = Vec::new();
        for _ in 0..buf_size {
          match samples.next() {
            None => break,
            Some(x) => snip.push(x),
          }
        }

        if snip.is_empty() {
          break
        }

        let len = snip.len();
        try!(string_err(stream.write(snip, len as u32)));
      }

      Ok(())
    }
  ));

  Ok(())
}

fn errorful_main() -> Result<(), String> {
  try!(string_err(env_logger::init()));
  try!(string_err(portaudio::pa::initialize()));

  let matches =
    clap_app!(pitchdetect =>
      (@subcommand play =>
        (about: "Play a note")
        (@arg note: +required +takes_value --note)
        (@arg time: +required +takes_value --time)
      )
      (@subcommand detect =>
        (about: "Detect pitch from the microphone")
      )
      (@subcommand harmony =>
        (about: "Add harmony to a recorded clip")
      )
    ).get_matches();

  if let Some(matches) = matches.subcommand_matches("play") {
    let note = try!(note::from_str(matches.value_of("note").unwrap()));
    let duration = try!(string_err(matches.value_of("time").unwrap().parse()));
    try!(play_note(note, duration));
  } else if let Some(_matches) = matches.subcommand_matches("detect") {
    try!(detect_pitch_main());
  } else if let Some(_harmony) = matches.subcommand_matches("harmony") {
    //let semitones = vec!(4, 7, 12);
    let semitones = vec!(4.0);
    try!(harmony_main(semitones));
  }

  Ok(())
}

#[main]
fn main() {
  errorful_main().unwrap();
}
