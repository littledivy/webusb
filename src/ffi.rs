use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::UsbDevice;

pub struct DeviceResource {
  pub device: rusb::Device<rusb::Context>,
  pub device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

impl DeviceResource {
  pub fn new(device: rusb::Device<rusb::Context>) -> Self {
    Self {
      device,
      device_handle: None,
    }
  }
}

pub type Resources = Arc<Mutex<HashMap<i32, Arc<Mutex<DeviceResource>>>>>;

pub static RESOURCES: Lazy<Resources> = Lazy::new(|| {
  let table = HashMap::new();
  Arc::new(Mutex::new(table))
});

pub fn insert_device(rid: i32, device: rusb::Device<rusb::Context>) {
  let mut resources = RESOURCES.lock().unwrap();
  resources.insert(rid, Arc::new(Mutex::new(DeviceResource::new(device))));
}
