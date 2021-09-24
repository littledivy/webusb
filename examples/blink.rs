use webusb::Context;
use webusb::Direction;
use webusb::Result;
use webusb::UsbControlTransferParameters;
use webusb::UsbRecipient;
use webusb::UsbRequestType;

use std::io::Read;
use std::io::Write;

const ARDUINO_CONTROL_INIT: UsbControlTransferParameters =
  UsbControlTransferParameters {
    request_type: UsbRequestType::Class,
    recipient: UsbRecipient::Interface,
    request: 0x22,
    value: 0x01,
    index: 2,
  };

const ARDUINO_CONTROL_BYE: UsbControlTransferParameters =
  UsbControlTransferParameters {
    request_type: UsbRequestType::Class,
    recipient: UsbRecipient::Interface,
    request: 0x22,
    value: 0x00,
    index: 2,
  };

#[tokio::main]
async fn main() -> Result<()> {
  let context = Context::init()?;
  let devices = context.devices().await?;

  let mut device = devices
    .into_iter()
    .find(|d| d.vendor_id == 0x2341 && d.product_id == 0x8036)
    .expect("Device not found.");
  device.open().await?;

  device.claim_interface(2).await?;
  device.select_alternate_interface(2, 0).await?;

  device
    .control_transfer_out(ARDUINO_CONTROL_INIT, &[])
    .await?;

  loop {
    let input: Option<u8> = std::io::stdin()
      .bytes()
      .next()
      .and_then(|result| result.ok());

    match input {
      Some(b'H') => {
        device.transfer_out(4, b"H").await?;
        device.clear_halt(Direction::Out, 4).await?;
      }
      Some(b'L') => {
        device.transfer_out(4, b"L").await?;
        device.clear_halt(Direction::Out, 4).await?;
      }
      Some(b'Q') => break,
      _ => {}
    }
  }

  device
    .control_transfer_out(ARDUINO_CONTROL_BYE, &[])
    .await?;
  device.close().await?;
  Ok(())
}
