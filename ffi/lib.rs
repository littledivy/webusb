use webusb::UsbDevice as Device;

macro_rules! c_ffi {
  ($module:ident, fn $name:ident($($arg:ident: $arg_type:ty),*) -> Result<$ret_type:ty, ()> { $($body:tt)* }) => {
    #[no_mangle]
    pub extern "C" fn $name($module: *mut Device, $($arg: $arg_type),*) -> $ret_type {
      let $module = unsafe { &mut *$module };
      let res: Result<$ret_type, ()> = (|| { $($body)* })();
      match res {
        Ok(v) => v,
        Err(_) => Default::default(),
      }
    }
  };
}

c_ffi!(
  device,
  fn webusb_open_device() -> Result<(), ()> {
    device.open().map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_close_device() -> Result<(), ()> {
    device.close().map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_reset_device() -> Result<(), ()> {
    device.reset().map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_transfer_out(
    endpoint: u8,
    data: *const u8,
    length: u32
  ) -> Result<usize, ()> {
    let data = unsafe { std::slice::from_raw_parts(data, length as usize) };
    device.transfer_out(endpoint, data).map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_transfer_in(
    endpoint: u8,
    size: u32,
    out: *mut *mut u8
  ) -> Result<(), ()> {
    let mut buf = device
      .transfer_in(endpoint, size as usize)
      .map_err(|_| ())?;
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    unsafe { *out = ptr };
    Ok(())
  }
);

c_ffi!(
  device,
  fn webusb_free_buffer(buf: *mut u8, size: u32) -> Result<(), ()> {
    let _ = unsafe { Vec::from_raw_parts(buf, size as usize, size as usize) };
    Ok(())
  }
);

c_ffi!(
  device,
  fn webusb_clear_halt(direction: u8, endpoint: u8) -> Result<(), ()> {
    let direction = match direction {
      0 => webusb::Direction::Out,
      1 => webusb::Direction::In,
      _ => return Err(()),
    };
    device.clear_halt(direction, endpoint).map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_select_alternate_interface(
    interface: u8,
    alternate: u8
  ) -> Result<(), ()> {
    device
      .select_alternate_interface(interface, alternate)
      .map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_claim_interface(interface: u8) -> Result<(), ()> {
    device.claim_interface(interface).map_err(|_| ())
  }
);

c_ffi!(
  device,
  fn webusb_release_interface(interface: u8) -> Result<(), ()> {
    device.release_interface(interface).map_err(|_| ())
  }
);

c_ffi!(device, fn weusb_select_configuration(configuration: u8) -> Result<(), ()> {
  device.select_configuration(configuration).map_err(|_| ())
});

c_ffi!(device, 
  fn webusb_control_transfer_out(
    request_type: webusb::UsbRequestType,
    recipient: webusb::UsbRecipient,
    request: u8,
    value: u16,
    index: u16,
    data: *const u8,
    length: u32
  ) -> Result<u32, ()> {
    let data = unsafe { std::slice::from_raw_parts(data, length as usize) };
    let setup = webusb::UsbControlTransferParameters {
      request_type,
      recipient,
      request,
      value,
      index,
    };
    device
      .control_transfer_out(setup, data)
      .map_err(|_| ())
      .map(|v| v as u32)
});

c_ffi!(device, fn webusb_control_transfer_in(
    request_type: webusb::UsbRequestType,
    recipient: webusb::UsbRecipient,
    request: u8,
    value: u16,
    index: u16,
    length: u32,
    out: *mut *mut u8
 ) -> Result<(), ()> {
    let setup = webusb::UsbControlTransferParameters {
      request_type,
      recipient,
      request,
      value,
      index,
    };
    let mut buf = device
      .control_transfer_in(setup, length as usize)
      .map_err(|_| ())?;
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    unsafe { *out = ptr };
    Ok(())
});
