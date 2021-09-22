use crate::Direction;
use crate::Result;
use crate::UsbControlTransferParameters;

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
    setup: UsbControlTransferParameters,
    length: usize,
  ) -> Result<Vec<u8>>;
  fn control_transfer_out(
    &mut self,
    setup: UsbControlTransferParameters,
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

/// Describes a Usb backend. This lets target native, WASM and even *nothing*
#[async_trait::async_trait]
pub trait Backend {
  type Device;

  /// Initializes the backend.
  fn init() -> Result<Self>
  where
    Self: Sized;

  /// List all devices.
  async fn devices(&self) -> Result<Vec<Self::Device>>;
}
