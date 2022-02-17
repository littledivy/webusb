use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub struct DeviceResource {
  device: rusb::Device<rusb::Context>,
  device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

pub type Resources = Arc<Mutex<HashMap<i32, Mutex<Mutex<DeviceResource>>>>>;

pub static RESOURCES: Lazy<Resources> = Lazy::new(|| {
  let table = HashMap::new();
  Arc::new(Mutex::new(table));
});
