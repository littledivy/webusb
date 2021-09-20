use crate::Direction;
use crate::Result;
use crate::USBControlTransferParameters;

pub trait WebUsbDevice {
  fn open(&mut self) -> Result<()>;
  fn close(&mut self) -> Result<()>;
  fn select_configuration(&mut self, configuration_value: u8) -> Result<()>;
  fn claim_interface(&mut self, interface_number: u8) -> Result<()>;
  fn release_interface(&mut self, interface_number: u8) -> Result<()>;
  fn select_alternate_interface(
    &mut self,
    interface_number: u8,
    alternate_setting: u8,
  ) -> Result<()>;
  fn control_transfer_in(
    &mut self,
    setup: USBControlTransferParameters,
    length: usize,
  ) -> Result<Vec<u8>>;
  fn control_transfer_out(
    &mut self,
    setup: USBControlTransferParameters,
    data: &[u8],
  ) -> Result<usize>;
  fn clear_halt(
    &mut self,
    direction: Direction,
    endpoint_number: u8,
  ) -> Result<()>;
  fn transfer_in(
    &mut self,
    endpoint_number: u8,
    length: usize,
  ) -> Result<Vec<u8>>;
  fn transfer_out(&mut self, endpoint_number: u8, data: &[u8])
    -> Result<usize>;
  fn isochronous_transfer_in(&mut self) {
    unimplemented!()
  }
  fn isochronous_transfer_out(&mut self) {
    unimplemented!()
  }
  fn reset(&mut self) -> Result<()>;
}

/// Describes a USB backend. This lets target native, WASM and even *nothing*
pub trait Backend {
  /// Initializes the backend.
  fn init() -> Result<Self>
  where
    Self: Sized;

  /// List all devices.
  fn devices<C: WebUsbDevice>(&self) -> Result<C>;
}
