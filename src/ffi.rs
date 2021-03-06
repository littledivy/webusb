use deno_bindgen::deno_bindgen;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::Direction;
use crate::UsbControlTransferParameters;
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

#[deno_bindgen]
pub struct Devices {
  devices: Vec<UsbDevice>,
}

#[deno_bindgen]
pub struct Device {
  device: UsbDevice,
}

#[deno_bindgen]
pub struct FfiDirection {
  inner: Direction,
}

#[deno_bindgen(non_blocking)]
pub fn get_devices() -> Devices {
  let ctx = crate::Context::init().unwrap();
  let devices = ctx.devices().unwrap();
  Devices { devices }
}

macro_rules! wrap_ffi_method {
  ($method: ident) => {
    #[deno_bindgen]
    pub fn $method(mut device: Device) -> Device {
      device.device.$method().unwrap();
      device
    }
  };
}

wrap_ffi_method!(open);
wrap_ffi_method!(close);
wrap_ffi_method!(reset);

#[deno_bindgen]
pub fn transfer_out(mut device: Device, endpoint_number: u8, data: &[u8]) {
  device.device.transfer_out(endpoint_number, data).unwrap();
}

#[deno_bindgen]
pub fn transfer_in(
  mut device: Device,
  endpoint_number: u8,
  size: usize,
) -> *const u8 {
  let data = device.device.transfer_in(endpoint_number, size).unwrap();
  let ptr = data.as_ptr();
  // TODO: deallocate from JS
  std::mem::forget(data);
  ptr
}

#[deno_bindgen]
pub fn clear_halt(
  mut device: Device,
  direction: FfiDirection,
  endpoint_number: u8,
) -> Device {
  device
    .device
    .clear_halt(direction.inner, endpoint_number)
    .unwrap();
  device
}

#[deno_bindgen]
pub struct FfiUsbControlTransferParameters {
  inner: UsbControlTransferParameters,
}

#[deno_bindgen]
pub fn control_transfer_out(
  mut device: Device,
  setup: FfiUsbControlTransferParameters,
  data: &[u8],
) -> usize {
  device
    .device
    .control_transfer_out(setup.inner, data)
    .unwrap()
}

#[deno_bindgen]
pub fn control_transfer_in(
  mut device: Device,
  setup: FfiUsbControlTransferParameters,
  length: usize,
) -> *const u8 {
  let data = device
    .device
    .control_transfer_in(setup.inner, length)
    .unwrap();
  let ptr = data.as_ptr();
  // TODO: deallocate from JS
  std::mem::forget(data);
  ptr
}

#[deno_bindgen]
fn select_alternate_interface(
  mut device: Device,
  interface_number: u8,
  alternate_setting: u8,
) -> Device {
  device
    .device
    .select_alternate_interface(interface_number, alternate_setting)
    .unwrap();
  device
}

#[deno_bindgen]
fn release_interface(mut device: Device, interface_number: u8) -> Device {
  device.device.release_interface(interface_number).unwrap();
  device
}

#[deno_bindgen]
fn claim_interface(mut device: Device, interface_number: u8) -> Device {
  device.device.claim_interface(interface_number).unwrap();
  device
}

#[deno_bindgen]
fn select_configuration(mut device: Device, configuration_value: u8) -> Device {
  device
    .device
    .select_configuration(configuration_value)
    .unwrap();
  device
}
