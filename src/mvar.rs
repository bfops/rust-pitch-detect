use std;

// TODO: Return any PoisonErrors.

pub struct T<X> {
  full: std::sync::Condvar,
  data: std::sync::Mutex<Option<X>>,
}

unsafe impl<X> Send for T<X> {}
unsafe impl<X> Sync for T<X> {}

pub fn new<X>() -> T<X> {
  T {
    full: std::sync::Condvar::new(),
    data: std::sync::Mutex::new(None),
  }
}

impl<X> T<X> {
  pub fn overwrite(&self, val: X) {
    let mut data = self.data.lock().unwrap();
    *data = Some(val);
    self.full.notify_one();
  }

  pub fn take(&self) -> X {
    let mut data = self.data.lock().unwrap();
    loop {
      data = self.full.wait(data).unwrap();

      let mut r = None;
      std::mem::swap(&mut r, &mut *data);
      match r {
        None => {},
        Some(r) => return r,
      }
    }
  }
}
