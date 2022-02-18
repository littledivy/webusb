//! # webusb
//!
//! Implementation of the [WebUSB API specification](https://wicg.github.io/webusb/) in
//! Rust.
//!
//! ## Design
//!
//! The crate is designed to be as close to the WebUSB specification as possible.
//! There are two "backends" available, Native and WASM.
//!
//! The native backend (`libusb`) supports parsing webusb descriptors. The wasm backend will
//! make use of the runtime's WebUSB implementation.
//!
//! see [usbd-webusb](https://github.com/redpfire/usbd-webusb) for WebUSB compatible firmware
//! for the device.
//!
//! ## Usage
//!
//! See [webusb/examples](https://github.com/littledivy/webusb/tree/main/examples) for usage examples.
//!

#[cfg(feature = "serde_derive")]
use serde::Deserialize;
#[cfg(feature = "serde_derive")]
use serde::Serialize;

#[cfg(feature = "libusb")]
use rusb::UsbContext;

use core::convert::TryFrom;

#[cfg(feature = "libusb")]
pub use rusb;

#[cfg(feature = "wasm")]
pub use web_sys;

pub mod constants;
mod descriptors;

use crate::constants::BOS_DESCRIPTOR_TYPE;
use crate::constants::GET_URL_REQUEST;
use crate::descriptors::parse_bos;
use crate::descriptors::parse_webusb_url;

const EP_DIR_IN: u8 = 0x80;
const EP_DIR_OUT: u8 = 0x0;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
  #[cfg(feature = "libusb")]
  Usb(rusb::Error),
  NotFound,
  InvalidState,
  InvalidAccess,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "libusb")]
impl From<rusb::Error> for Error {
  fn from(err: rusb::Error) -> Self {
    Self::Usb(err)
  }
}

impl<T> From<Option<T>> for Error {
  fn from(_: Option<T>) -> Self {
    Self::NotFound
  }
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbConfiguration {
  // Index of String Descriptor describing this configuration.
  configuration_name: Option<String>,
  // The configuration number (bConfigurationValue)
  // https://www.beyondlogic.org/usbnutshell/usb5.shtml#ConfigurationDescriptors
  configuration_value: u8,
  interfaces: Vec<UsbInterface>,
}

#[cfg(feature = "wasm")]
impl From<web_sys::UsbConfiguration> for UsbConfiguration {
  fn from(config: web_sys::UsbConfiguration) -> Self {
    let interfaces = {
      let array = config.interfaces().to_vec();
      array
        .into_iter()
        .map(|itf| UsbInterface::from(web_sys::UsbInterface::from(itf)))
        .collect()
    };

    Self {
      configuration_name: config.configuration_name(),
      configuration_value: config.configuration_value(),
      interfaces,
    }
  }
}

#[cfg(feature = "libusb")]
impl UsbConfiguration {
  pub fn from(
    config_descriptor: rusb::ConfigDescriptor,
    handle: &rusb::DeviceHandle<rusb::Context>,
  ) -> Result<Self> {
    Ok(UsbConfiguration {
      configuration_name: match config_descriptor.description_string_index() {
        None => None,
        Some(idx) => Some(handle.read_string_descriptor_ascii(idx)?),
      },
      configuration_value: config_descriptor.number(),
      interfaces: config_descriptor
        .interfaces()
        .map(|i| UsbInterface::from(i, &handle))
        .collect::<Vec<UsbInterface>>(),
    })
  }
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbInterface {
  interface_number: u8,
  alternate: UsbAlternateInterface,
  alternates: Vec<UsbAlternateInterface>,
  claimed: bool,
}

#[cfg(feature = "wasm")]
impl From<web_sys::UsbInterface> for UsbInterface {
  fn from(interface: web_sys::UsbInterface) -> Self {
    let alternates = {
      let array = interface.alternates().to_vec();
      array
        .into_iter()
        .map(|ep| {
          UsbAlternateInterface::from(web_sys::UsbAlternateInterface::from(ep))
        })
        .collect()
    };
    Self {
      interface_number: interface.interface_number(),
      alternate: UsbAlternateInterface::from(interface.alternate()),
      alternates,
      claimed: interface.claimed(),
    }
  }
}

#[cfg(feature = "libusb")]
impl UsbInterface {
  pub fn from(
    i: rusb::Interface,
    handle: &rusb::DeviceHandle<rusb::Context>,
  ) -> Self {
    UsbInterface {
      interface_number: i.number(),
      claimed: false,
      // By default, the alternate setting is for the interface with
      // bAlternateSetting equal to 0.
      alternate: {
        // TODO: don't panic
        let interface =
          i.descriptors().find(|d| d.setting_number() == 0).unwrap();
        UsbAlternateInterface::from(interface, &handle)
      },
      alternates: i
        .descriptors()
        .map(|interface| UsbAlternateInterface::from(interface, &handle))
        .collect(),
    }
  }
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub enum UsbEndpointType {
  Bulk,
  Interrupt,
  Isochronous,
  Control,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "lowercase")
)]
pub enum Direction {
  In,
  Out,
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbEndpoint {
  endpoint_number: u8,
  direction: Direction,
  // TODO(@littledivy): Get rid of reserved `type` key somehow?
  r#type: UsbEndpointType,
  packet_size: u16,
}

#[cfg(feature = "wasm")]
impl From<web_sys::UsbEndpoint> for UsbEndpoint {
  fn from(ep: web_sys::UsbEndpoint) -> Self {
    Self {
      endpoint_number: ep.endpoint_number(),
      direction: match ep.direction() {
        web_sys::UsbDirection::In => Direction::In,
        web_sys::UsbDirection::Out => Direction::Out,
        _ => unreachable!(),
      },
      r#type: match ep.type_() {
        web_sys::UsbEndpointType::Bulk => UsbEndpointType::Bulk,
        web_sys::UsbEndpointType::Interrupt => UsbEndpointType::Interrupt,
        web_sys::UsbEndpointType::Isochronous => UsbEndpointType::Isochronous,
        _ => unreachable!(),
      },
      packet_size: ep.packet_size() as u16,
    }
  }
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbAlternateInterface {
  alternate_setting: u8,
  interface_class: u8,
  interface_subclass: u8,
  interface_protocol: u8,
  interface_name: Option<String>,
  endpoints: Vec<UsbEndpoint>,
}

#[cfg(feature = "wasm")]
impl From<web_sys::UsbAlternateInterface> for UsbAlternateInterface {
  fn from(interface: web_sys::UsbAlternateInterface) -> Self {
    let endpoints = {
      let array = interface.endpoints().to_vec();
      array
        .into_iter()
        .map(|ep| UsbEndpoint::from(web_sys::UsbEndpoint::from(ep)))
        .collect()
    };

    Self {
      alternate_setting: interface.alternate_setting(),
      interface_class: interface.interface_class(),
      interface_subclass: interface.interface_subclass(),
      interface_protocol: interface.interface_protocol(),
      interface_name: interface.interface_name(),
      endpoints,
    }
  }
}

#[cfg(feature = "libusb")]
impl UsbAlternateInterface {
  pub fn from(
    d: rusb::InterfaceDescriptor,
    handle: &rusb::DeviceHandle<rusb::Context>,
  ) -> Self {
    UsbAlternateInterface {
      alternate_setting: d.setting_number(),
      interface_class: d.class_code(),
      interface_subclass: d.sub_class_code(),
      interface_protocol: d.protocol_code(),
      interface_name: d
        .description_string_index()
        .map(|idx| handle.read_string_descriptor_ascii(idx).unwrap()),
      endpoints: d
        .endpoint_descriptors()
        .map(|e| UsbEndpoint {
          endpoint_number: e.number(),
          packet_size: e.max_packet_size(),
          direction: match e.direction() {
            rusb::Direction::In => Direction::In,
            rusb::Direction::Out => Direction::Out,
          },
          r#type: match e.transfer_type() {
            rusb::TransferType::Control => UsbEndpointType::Control,
            rusb::TransferType::Isochronous => UsbEndpointType::Isochronous,
            rusb::TransferType::Bulk => UsbEndpointType::Bulk,
            rusb::TransferType::Interrupt => UsbEndpointType::Interrupt,
          },
        })
        .collect(),
    }
  }
}

/// Represents a UsbDevice.
/// Only way you can obtain one is through `Context::devices`
/// https://wicg.github.io/webusb/#device-usage
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbDevice {
  /// List of configurations supported by the device.
  /// Populated from the configuration descriptor.
  /// `configurations.len()` SHALL be equal to the
  /// bNumConfigurations field of the device descriptor.
  pub configurations: Vec<UsbConfiguration>,
  /// Represents the currently selected configuration.
  /// One of the elements of `self.configurations`.
  /// None, if the device is not configured.
  pub configuration: Option<UsbConfiguration>,
  /// bDeviceClass value of the device descriptor.
  pub device_class: u8,
  /// bDeviceSubClass value of the device descriptor.
  pub device_subclass: u8,
  /// bDeviceProtocol value of the device descriptor.
  pub device_protocol: u8,
  /// The major version declared by bcdDevice field
  /// such that bcdDevice 0xJJMN represents major version JJ.
  pub device_version_major: u8,
  /// The minor version declared by bcdDevice field
  /// such that bcdDevice 0xJJMN represents minor version M.
  pub device_version_minor: u8,
  /// The subminor version declared by bcdDevice field
  /// such that bcdDevice 0xJJMN represents subminor version N.
  pub device_version_subminor: u8,
  /// Optional property of the string descriptor.
  /// Indexed by the iManufacturer field of device descriptor.
  pub manufacturer_name: Option<String>,
  /// idProduct field of the device descriptor.
  pub product_id: u16,
  /// Optional property of the string descriptor.
  /// Indexed by the iProduct field of device descriptor.
  pub product_name: Option<String>,
  /// Optional property of the string descriptor.
  /// None, if the iSerialNumber field of device descriptor
  /// is 0.
  pub serial_number: Option<String>,
  /// The major version declared by bcdUSB field
  /// such that bcdUSB 0xJJMN represents major version JJ.
  pub usb_version_major: u8,
  /// The minor version declared by bcdUSB field
  /// such that bcdUSB 0xJJMN represents minor version M.
  pub usb_version_minor: u8,
  /// The subminor version declared by bcdUSB field
  /// such that bcdUSB 0xJJMN represents subminor version N.
  pub usb_version_subminor: u8,
  /// idVendor field of the device descriptor.
  /// https://wicg.github.io/webusb/#vendor-id
  pub vendor_id: u16,
  /// If true, the underlying device handle is owned by this object.
  pub opened: bool,

  /// WEBUSB_URL value of the WebUSB Platform Capability Descriptor.
  #[cfg_attr(
    feature = "serde_derive",
    doc = "NOTE: Skipped during serde deserialization.",
    serde(skip)
  )]
  pub url: Option<String>,

  #[cfg_attr(feature = "serde_derive", serde(skip))]
  #[cfg(feature = "libusb")]
  device: rusb::Device<rusb::Context>,

  #[cfg_attr(feature = "serde_derive", serde(skip))]
  #[cfg(feature = "wasm")]
  device: web_sys::UsbDevice,

  #[cfg_attr(feature = "serde_derive", serde(skip))]
  #[cfg(feature = "libusb")]
  device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

impl UsbDevice {
  // https://wicg.github.io/webusb/#check-the-validity-of-the-control-transfer-parameters
  fn validate_control_setup(
    &mut self,
    setup: &UsbControlTransferParameters,
  ) -> Result<()> {
    // 3.
    if let Some(configuration) = &self.configuration {
      match setup.recipient {
        // 4.
        UsbRecipient::Interface => {
          // 4.1
          let interface_number: u8 = (setup.index & 0xFF) as u8;

          // 4.2
          let interface = configuration
            .interfaces
            .iter()
            .find(|itf| itf.interface_number == interface_number)
            .ok_or(Error::NotFound)?;

          // 4.3
          if !interface.claimed {
            return Err(Error::InvalidState);
          }
        }
        // 5.
        UsbRecipient::Endpoint => {
          // 5.1
          let endpoint_number = setup.index as u8 & (1 << 4);

          // 5.2
          let direction = match (setup.index >> 8) & 1 {
            1 => Direction::In,
            _ => Direction::Out,
          };

          // 5.3-5.4
          let interface = configuration
            .interfaces
            .iter()
            .find(|itf| {
              itf
                .alternates
                .iter()
                .find(|alt| {
                  alt
                    .endpoints
                    .iter()
                    .find(|endpoint| {
                      endpoint.endpoint_number == endpoint_number
                        && endpoint.direction == direction
                    })
                    .is_some()
                })
                .is_some()
            })
            .ok_or(Error::NotFound)?;
        }
        _ => {}
      }
    }

    Ok(())
  }
}

impl UsbDevice {
  pub async fn isochronous_transfer_in(&mut self) {
    unimplemented!()
  }

  pub async fn isochronous_transfer_out(&mut self) {
    unimplemented!()
  }

  pub async fn open(&mut self) -> Result<()> {
    // 3. device is already open?
    if self.opened {
      return Ok(());
    }

    // 4.
    #[cfg(feature = "libusb")]
    {
      let handle = self.device.open()?;
      self.device_handle = Some(handle);
    }

    #[cfg(feature = "wasm")]
    {
      let fut = self.device.open();
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    }

    // 5.
    self.opened = true;
    Ok(())
  }

  pub async fn close(&mut self) -> Result<()> {
    // 3. device is already closed?
    if !self.opened {
      return Ok(());
    }

    #[cfg(feature = "libusb")]
    {
      match &self.device_handle {
        Some(handle_ref) => {
          // 5-6.
          // release claimed interfaces, close device and release handle
          drop(handle_ref);
        }
        None => unreachable!(),
      };

      self.device_handle = None;
    }

    #[cfg(feature = "wasm")]
    {
      let fut = self.device.close();
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    }

    // 7.
    self.opened = false;
    Ok(())
  }

  /// `configuration_value` is the bConfigurationValue of the device configuration.
  pub async fn select_configuration(
    &mut self,
    configuration_value: u8,
  ) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      let fut = self.device.select_configuration(configuration_value);
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
      // TODO: sync configuration
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      let configuration = match self
        .configurations
        .iter()
        .position(|c| c.configuration_value == configuration_value)
      {
        Some(config_idx) => self.device.config_descriptor(config_idx as u8)?,
        None => return Err(Error::NotFound),
      };

      // 4.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 5-6.
      let handle = match self.device_handle {
        Some(ref mut handle_ref) => {
          // Calls `libusb_set_configuration`
          handle_ref.set_active_configuration(configuration_value)?;
          handle_ref
        }
        None => unreachable!(),
      };

      // 7.
      self.configuration =
        Some(UsbConfiguration::from(configuration, &handle)?);
    }
    Ok(())
  }

  pub async fn claim_interface(&mut self, interface_number: u8) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      let fut = self.device.claim_interface(interface_number);
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
      // TODO: sync configuration
    }

    #[cfg(feature = "libusb")]
    {
      // 2.
      let mut active_configuration =
        self.configuration.as_mut().ok_or(Error::NotFound)?;
      let mut interface = match active_configuration
        .interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
      {
        Some(mut i) => i,
        None => return Err(Error::NotFound),
      };

      // 3.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 4.
      if interface.claimed {
        return Ok(());
      }

      // 5.
      match self.device_handle {
        Some(ref mut handle_ref) => {
          handle_ref.claim_interface(interface.interface_number)?;
        }
        None => unreachable!(),
      };

      // 6.
      interface.claimed = true;
    }

    Ok(())
  }

  pub async fn release_interface(
    &mut self,
    interface_number: u8,
  ) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      let fut = self.device.release_interface(interface_number);
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      let mut active_configuration =
        self.configuration.as_mut().ok_or(Error::NotFound)?;
      let mut interface = match active_configuration
        .interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
      {
        Some(mut i) => i,
        None => return Err(Error::NotFound),
      };

      // 4.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 5.
      if !interface.claimed {
        return Ok(());
      }

      // 5.
      match self.device_handle {
        Some(ref mut handle_ref) => {
          handle_ref.release_interface(interface.interface_number)?;
        }
        None => unreachable!(),
      };

      // 6.
      interface.claimed = false;
    }

    Ok(())
  }

  pub async fn select_alternate_interface(
    &mut self,
    interface_number: u8,
    alternate_setting: u8,
  ) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      let fut = self
        .device
        .select_alternate_interface(interface_number, alternate_setting);
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      let mut active_configuration =
        self.configuration.as_mut().ok_or(Error::NotFound)?;
      let mut interface = match active_configuration
        .interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
      {
        Some(mut i) => i,
        None => return Err(Error::NotFound),
      };

      // 4.
      if !self.opened || !interface.claimed {
        return Err(Error::InvalidState);
      }

      // 5-6.
      match self.device_handle {
        Some(ref mut handle_ref) => {
          handle_ref.set_alternate_setting(
            interface.interface_number,
            alternate_setting,
          )?;
        }
        None => unreachable!(),
      };
    }
    // 7.
    return Ok(());
  }

  pub async fn control_transfer_in(
    &mut self,
    setup: UsbControlTransferParameters,
    length: usize,
  ) -> Result<Vec<u8>> {
    #[cfg(feature = "wasm")]
    {
      // TODO
      return Ok(vec![0; length]);
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 4.
      self.validate_control_setup(&setup)?;

      // 5.
      let mut buffer = vec![0u8; length];

      // 6-7.
      let bytes_transferred = match self.device_handle {
        Some(ref mut handle_ref) => {
          let req = match setup.request_type {
            UsbRequestType::Standard => rusb::RequestType::Standard,
            UsbRequestType::Class => rusb::RequestType::Class,
            UsbRequestType::Vendor => rusb::RequestType::Vendor,
          };

          let recipient = match setup.recipient {
            UsbRecipient::Device => rusb::Recipient::Device,
            UsbRecipient::Interface => rusb::Recipient::Interface,
            UsbRecipient::Endpoint => rusb::Recipient::Endpoint,
            UsbRecipient::Other => rusb::Recipient::Other,
          };

          let req_type =
            rusb::request_type(rusb::Direction::In, req, recipient);

          handle_ref.read_control(
            req_type,
            setup.request,
            setup.value,
            setup.index,
            &mut buffer,
            std::time::Duration::new(0, 0),
          )?
        }
        None => unreachable!(),
      };

      // 8-9.
      // Returns the buffer containing first bytes_transferred instead of returning
      // a UsbInTransferResult.
      let result = &buffer[0..bytes_transferred];

      // 10-11. TODO: Will need to handle `read_control` Err

      // 13.
      Ok(result.to_vec())
    }
  }

  pub async fn control_transfer_out(
    &mut self,
    setup: UsbControlTransferParameters,
    data: &[u8],
  ) -> Result<usize> {
    #[cfg(feature = "wasm")]
    {
      // TODO
      return Ok(0);
    }

    #[cfg(feature = "libusb")]
    {
      // 2.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 3.
      self.validate_control_setup(&setup)?;

      // 4-8.
      let bytes_written = match self.device_handle {
        Some(ref mut handle_ref) => {
          let req = match setup.request_type {
            UsbRequestType::Standard => rusb::RequestType::Standard,
            UsbRequestType::Class => rusb::RequestType::Class,
            UsbRequestType::Vendor => rusb::RequestType::Vendor,
          };

          let recipient = match setup.recipient {
            UsbRecipient::Device => rusb::Recipient::Device,
            UsbRecipient::Interface => rusb::Recipient::Interface,
            UsbRecipient::Endpoint => rusb::Recipient::Endpoint,
            UsbRecipient::Other => rusb::Recipient::Other,
          };

          let req_type =
            rusb::request_type(rusb::Direction::Out, req, recipient);

          handle_ref.write_control(
            req_type,
            setup.request,
            setup.value,
            setup.index,
            data,
            std::time::Duration::new(0, 0),
          )?
        }
        None => unreachable!(),
      };

      // 9.
      Ok(bytes_written)
    }
  }

  pub async fn clear_halt(
    &mut self,
    direction: Direction,
    endpoint_number: u8,
  ) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      // TODO
    }

    #[cfg(feature = "libusb")]
    {
      let active_configuration =
        self.configuration.as_ref().ok_or(Error::NotFound)?;

      // 2.
      let interface = active_configuration
        .interfaces
        .iter()
        .find(|itf| {
          itf
            .alternates
            .iter()
            .find(|alt| {
              alt
                .endpoints
                .iter()
                .find(|endpoint| {
                  endpoint.endpoint_number == endpoint_number
                    && endpoint.direction == direction
                })
                .is_some()
            })
            .is_some()
        })
        .ok_or(Error::NotFound)?;

      // 3.
      if !self.opened || !interface.claimed {
        return Err(Error::InvalidState);
      }

      // 4-5.
      match self.device_handle {
        Some(ref mut handle_ref) => {
          let mut endpoint = endpoint_number;

          match direction {
            Direction::In => endpoint |= EP_DIR_IN,
            Direction::Out => endpoint |= EP_DIR_OUT,
          };

          handle_ref.clear_halt(endpoint)?
        }
        None => unreachable!(),
      };
    }
    Ok(())
  }

  pub async fn transfer_in(
    &mut self,
    endpoint_number: u8,
    length: usize,
  ) -> Result<Vec<u8>> {
    #[cfg(feature = "wasm")]
    {
      // TODO
      return Ok(vec![0; length]);
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      let endpoint = self
        .configuration
        .as_ref()
        .ok_or(Error::NotFound)?
        .interfaces
        .iter()
        .find_map(|itf| {
          itf.alternates.iter().find_map(|alt| {
            alt.endpoints.iter().find(|endpoint| {
              endpoint.endpoint_number == endpoint_number
                && endpoint.direction == Direction::In
            })
          })
        })
        .ok_or(Error::NotFound)?;

      // 4.
      match endpoint.r#type {
        UsbEndpointType::Bulk | UsbEndpointType::Interrupt => {}
        _ => return Err(Error::InvalidAccess),
      }

      // 5.
      // FIXME: Check if interface is claimed
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 6.
      let mut buffer = vec![0u8; length];

      // 7-8.
      let bytes_transferred = match self.device_handle {
        Some(ref mut handle_ref) => {
          let endpoint_addr = EP_DIR_IN | endpoint_number;

          match endpoint.r#type {
            UsbEndpointType::Bulk => handle_ref.read_bulk(
              endpoint_addr,
              &mut buffer,
              std::time::Duration::new(0, 0),
            )?,
            UsbEndpointType::Interrupt => handle_ref.read_interrupt(
              endpoint_addr,
              &mut buffer,
              std::time::Duration::new(0, 0),
            )?,
            _ => unreachable!(),
          }
        }
        None => unreachable!(),
      };

      // 10.
      let result = &buffer[0..bytes_transferred];

      // 11-14. See `control_transfer_in` TODO comment

      // 15.
      Ok(result.to_vec())
    }
  }

  pub async fn transfer_out(
    &mut self,
    endpoint_number: u8,
    data: &[u8],
  ) -> Result<usize> {
    #[cfg(feature = "wasm")]
    {
      // TODO
      return Ok(0);
    }

    #[cfg(feature = "libusb")]
    {
      // 2.
      let endpoint = self
        .configuration
        .as_ref()
        .ok_or(Error::NotFound)?
        .interfaces
        .iter()
        .find_map(|itf| {
          itf.alternates.iter().find_map(|alt| {
            alt.endpoints.iter().find(|endpoint| {
              endpoint.endpoint_number == endpoint_number
                && endpoint.direction == Direction::Out
            })
          })
        })
        .ok_or(Error::NotFound)?;

      // 3.
      match endpoint.r#type {
        UsbEndpointType::Bulk | UsbEndpointType::Interrupt => {}
        _ => return Err(Error::InvalidAccess),
      }

      // 4.
      // FIXME: Check if interface is claimed
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 5.
      let bytes_written = match self.device_handle {
        Some(ref mut handle_ref) => {
          let endpoint_addr = EP_DIR_OUT | endpoint_number;

          match endpoint.r#type {
            UsbEndpointType::Bulk => handle_ref.write_bulk(
              endpoint_addr,
              data,
              std::time::Duration::new(0, 0),
            )?,
            UsbEndpointType::Interrupt => handle_ref.write_interrupt(
              endpoint_addr,
              data,
              std::time::Duration::new(0, 0),
            )?,
            _ => unreachable!(),
          }
        }
        None => unreachable!(),
      };

      Ok(bytes_written)
    }
  }

  pub async fn reset(&mut self) -> Result<()> {
    #[cfg(feature = "wasm")]
    {
      let fut = self.device.reset();
      wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    }

    #[cfg(feature = "libusb")]
    {
      // 3.
      if !self.opened {
        return Err(Error::InvalidState);
      }

      // 4-6.
      match self.device_handle {
        Some(ref mut handle_ref) => handle_ref.reset()?,
        None => unreachable!(),
      };
    }
    Ok(())
  }
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "lowercase")
)]
pub enum UsbRequestType {
  Standard,
  Class,
  Vendor,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "lowercase")
)]
pub enum UsbRecipient {
  Device,
  Interface,
  Endpoint,
  Other,
}

#[derive(Clone)]
#[cfg_attr(
  feature = "serde_derive",
  derive(Serialize, Deserialize),
  serde(rename_all = "camelCase")
)]
pub struct UsbControlTransferParameters {
  pub request_type: UsbRequestType,
  pub recipient: UsbRecipient,
  pub request: u8,
  pub value: u16,
  pub index: u16,
}

#[cfg(feature = "wasm")]
impl TryFrom<web_sys::UsbDevice> for UsbDevice {
  type Error = Error;

  fn try_from(dev: web_sys::UsbDevice) -> Result<UsbDevice> {
    let configurations = {
      let array = dev.configurations().to_vec();
      array
        .into_iter()
        .map(|config| {
          UsbConfiguration::from(web_sys::UsbConfiguration::from(config))
        })
        .collect()
    };

    Ok(UsbDevice {
      configurations,
      configuration: dev.configuration().map(|c| UsbConfiguration::from(c)),
      device_class: dev.device_class(),
      device_subclass: dev.device_subclass(),
      device_protocol: dev.device_protocol(),
      device_version_major: dev.device_version_major(),
      device_version_minor: dev.device_version_minor(),
      device_version_subminor: dev.device_version_subminor(),
      product_id: dev.product_id(),
      usb_version_major: dev.usb_version_major(),
      usb_version_minor: dev.usb_version_minor(),
      usb_version_subminor: dev.usb_version_subminor(),
      vendor_id: dev.vendor_id(),
      manufacturer_name: dev.manufacturer_name(),
      product_name: dev.product_name(),
      serial_number: dev.serial_number(),
      opened: dev.opened(),
      url: None,
      device: dev,
    })
  }
}

#[cfg(feature = "libusb")]
impl TryFrom<rusb::Device<rusb::Context>> for UsbDevice {
  type Error = Error;

  fn try_from(device: rusb::Device<rusb::Context>) -> Result<UsbDevice> {
    let device_descriptor = device.device_descriptor()?;
    let device_class = device_descriptor.class_code();
    let usb_version = device_descriptor.usb_version();

    let config_descriptor = device.active_config_descriptor();
    let handle = device.open()?;
    let read_bos_descriptors = usb_version.0 >= 2 && usb_version.1 >= 1;
    let url = if read_bos_descriptors {
      // Check descriptor.iManufacturer != 0 && descriptor.iProduct != 0 && descriptor.iSerialNumber != 0

      // Read capability descriptor
      let request_type = rusb::request_type(
        rusb::Direction::In,
        rusb::RequestType::Standard,
        rusb::Recipient::Device,
      );
      let kGetDescriptorRequest = 0x06;

      let mut buffer = [0; 5];
      let length = handle.read_control(
        request_type,
        kGetDescriptorRequest,
        BOS_DESCRIPTOR_TYPE << 8,
        0,
        &mut buffer,
        core::time::Duration::new(2, 0),
      )?;
      assert_eq!(length, 5);

      // Read BOS descriptor
      let new_length = buffer[2] | (buffer[3].wrapping_shl(8));
      let mut new_buffer = vec![0; new_length as usize];
      let request_type = rusb::request_type(
        rusb::Direction::In,
        rusb::RequestType::Standard,
        rusb::Recipient::Device,
      );
      handle.read_control(
        request_type,
        kGetDescriptorRequest,
        BOS_DESCRIPTOR_TYPE << 8,
        0,
        &mut new_buffer,
        core::time::Duration::new(2, 0),
      )?;

      // Parse capibility from BOS descriptor
      if let Some((vendor_code, landing_page_id)) = parse_bos(&new_buffer) {
        let mut buffer = [0; 255];
        let request_type = rusb::request_type(
          rusb::Direction::In,
          rusb::RequestType::Vendor,
          rusb::Recipient::Device,
        );

        handle.read_control(
          request_type,
          vendor_code,
          landing_page_id as u16,
          GET_URL_REQUEST,
          &mut buffer,
          core::time::Duration::new(2, 0),
        )?;

        // Parse URL descriptor
        let url = parse_webusb_url(&buffer);
        url
      } else {
        None
      }
    } else {
      None
    };

    let configuration = match config_descriptor {
      Ok(config_descriptor) => {
        UsbConfiguration::from(config_descriptor, &handle).ok()
      }
      Err(_) => None,
    };

    let num_configurations = device_descriptor.num_configurations();
    let mut configurations: Vec<UsbConfiguration> = vec![];
    for idx in 0..num_configurations {
      if let Ok(curr_config_descriptor) = device.config_descriptor(idx) {
        configurations
          .push(UsbConfiguration::from(curr_config_descriptor, &handle)?);
      }
    }

    let device_version = device_descriptor.device_version();
    let manufacturer_name = handle
      .read_manufacturer_string_ascii(&device_descriptor)
      .ok();
    let product_name =
      handle.read_product_string_ascii(&device_descriptor).ok();
    let serial_number = handle
      .read_serial_number_string_ascii(&device_descriptor)
      .ok();

    let usb_device = UsbDevice {
      configurations,
      configuration,
      device_class,
      device_subclass: device_descriptor.sub_class_code(),
      device_protocol: device_descriptor.protocol_code(),
      device_version_major: device_version.major(),
      device_version_minor: device_version.minor(),
      device_version_subminor: device_version.sub_minor(),
      product_id: device_descriptor.product_id(),
      usb_version_major: usb_version.major(),
      usb_version_minor: usb_version.minor(),
      usb_version_subminor: usb_version.sub_minor(),
      vendor_id: device_descriptor.vendor_id(),
      manufacturer_name,
      product_name,
      serial_number,
      opened: false,
      url,
      device,
      device_handle: None,
    };

    // Explicitly close the device.
    drop(handle);

    Ok(usb_device)
  }
}

#[cfg(feature = "wasm")]
pub struct Context(web_sys::Usb);

#[cfg(feature = "wasm")]
impl Context {
  pub fn init() -> Result<Self> {
    let window = web_sys::window().unwrap();
    Ok(Self(window.navigator().usb()))
  }

  pub async fn devices(&self) -> Result<Vec<UsbDevice>> {
    let usb = self.0.clone();

    let fut = usb.get_devices();
    let fut_value = wasm_bindgen_futures::JsFuture::from(fut).await.unwrap();
    let devices_array: js_sys::Array = js_sys::Array::from(&fut_value);

    let devices = devices_array
      .to_vec()
      .into_iter()
      .map(|val| UsbDevice::try_from(web_sys::UsbDevice::from(val)).unwrap())
      .collect();

    Ok(devices)
  }
}

/// A WebUSB Context. Provides APIs for device enumaration.
#[cfg(feature = "libusb")]
pub struct Context(rusb::Context);

#[cfg(feature = "libusb")]
impl Context {
  pub fn init() -> Result<Self> {
    let ctx = rusb::Context::new()?;
    Ok(Self(ctx))
  }

  pub async fn devices(&self) -> Result<Vec<UsbDevice>> {
    let devices = self.0.devices()?;

    let usb_devices: Vec<UsbDevice> = devices
      .iter()
      .filter(|d| {
        // Do not list hubs.
        // TODO(@littledivy): WTF is this code
        d.device_descriptor().is_ok()
          && d.device_descriptor().unwrap().class_code() != 9
      })
      .map(|d| UsbDevice::try_from(d))
      .filter(|d| {
        d.is_ok()
          || d.as_ref().err().unwrap() != &Error::Usb(rusb::Error::Access)
      })
      .map(|d| d.unwrap())
      .collect::<Vec<UsbDevice>>();
    Ok(usb_devices)
  }
}

#[cfg(test)]
mod tests {
  // These tests depends on real hardware.
  // TODO(@littledivy): Document running tests locally.
  use crate::Context;
  use crate::Direction;
  use crate::Error;
  use crate::UsbControlTransferParameters;
  use crate::UsbDevice;
  use crate::UsbRecipient;
  use crate::UsbRequestType;

  use std::sync::Arc;
  use std::sync::Mutex;
  use std::thread;

  // Arduino Leonardo (2341:8036).
  // Make sure you follow the instructions and load this sketch https://github.com/webusb/arduino/blob/gh-pages/demos/console/sketch/sketch.ino
  async fn test_device() -> UsbDevice {
    let ctx = Context::init().unwrap();
    let devices = ctx.devices().await.unwrap();
    let device = devices.into_iter().find(|d| d.vendor_id == 0x2341 && d.product_id == 0x8036).expect("Device not found.\nhelp: ensure you follow the test setup instructions carefully");
    device
  }

  #[tokio::test]
  async fn test_bos() -> crate::Result<()> {
    // Read and Parse BOS the descriptor.
    let mut device = test_device().await;
    assert_eq!(
      device.url,
      Some("https://webusb.github.io/arduino/demos/console".to_string())
    );

    Ok(())
  }

  #[tokio::test]
  async fn test_device_initial_state() -> crate::Result<()> {
    let mut device = test_device().await;

    device.open().await?;
    device.open().await?;

    device.close().await?;
    device.close().await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_device_invalid_state() -> crate::Result<()> {
    let mut device = test_device().await;

    // Without open() should panic.
    device.select_configuration(1).await.unwrap_err();
    device.claim_interface(2).await.unwrap_err();

    device.select_alternate_interface(2, 0).await.unwrap_err();

    device
      .control_transfer_out(
        UsbControlTransferParameters {
          request_type: crate::UsbRequestType::Class,
          recipient: crate::UsbRecipient::Interface,
          request: 0x22,
          value: 0x01,
          index: 2,
        },
        &[],
      )
      .await
      .unwrap_err();

    device.transfer_out(4, b"H").await.unwrap_err();
    device.clear_halt(Direction::Out, 4).await.unwrap_err();
    device.transfer_out(4, b"L").await.unwrap_err();
    device.clear_halt(Direction::Out, 4).await.unwrap_err();

    device
      .control_transfer_out(
        UsbControlTransferParameters {
          request_type: crate::UsbRequestType::Class,
          recipient: crate::UsbRecipient::Interface,
          request: 0x22,
          value: 0x00,
          index: 2,
        },
        &[],
      )
      .await
      .unwrap_err();
    device.release_interface(2).await.unwrap_err();
    device.reset().await.unwrap_err();
    device.close().await?;
    Ok(())
  }

  fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use tokio::runtime;

    let rt = runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();

    rt.block_on(future)
  }

  #[flaky_test::flaky_test]
  fn test_device_blink() {
    block_on(async move {
      async fn test(device: &mut UsbDevice) {
        device.transfer_out(4, b"H").await.unwrap();
        device.clear_halt(Direction::Out, 4).await.unwrap();

        device.transfer_out(4, b"L").await.unwrap();
        device.clear_halt(Direction::Out, 4).await.unwrap();

        let recv = device.transfer_in(5, 64).await.unwrap();
        let mut first_run = false;

        match recv.as_slice() {
          b"Sketch begins.\r\n> " => {
            first_run = true;
          }
          b"H\r\nTurning LED on.\r\n> " => {}
          _ => unreachable!(),
        };
        let recv = device.transfer_in(5, 64).await.unwrap();

        match (first_run, recv.as_slice()) {
          (true, b"H\r\nTurning LED on.\r\n> ")
          | (false, b"L\r\nTurning LED off.\r\n> ") => {}
          _ => unreachable!(),
        };
      }
      let mut device = test_device().await;

      device.open().await.unwrap();

      // A real world application should use `device.configuration.is_none()`.
      match device.select_configuration(1).await {
        Ok(_) => {} // Unreachable in the test runner
        Err(crate::Error::Usb(rusb::Error::Busy))
        | Err(crate::Error::InvalidState) => {}
        _ => unreachable!(),
      }

      // Device might be busy.
      if device.claim_interface(2).await.is_ok() {
        device.select_alternate_interface(2, 0).await.unwrap();

        device
          .control_transfer_out(
            UsbControlTransferParameters {
              request_type: crate::UsbRequestType::Class,
              recipient: crate::UsbRecipient::Interface,
              request: 0x22,
              value: 0x01,
              index: 2,
            },
            &[],
          )
          .await
          .unwrap();
        test(&mut device).await;
        device
          .control_transfer_out(
            UsbControlTransferParameters {
              request_type: crate::UsbRequestType::Class,
              recipient: crate::UsbRecipient::Interface,
              request: 0x22,
              value: 0x00,
              index: 2,
            },
            &[],
          )
          .await
          .unwrap();
      } else {
        test(&mut device).await;
      }
      device.release_interface(2).await.unwrap();
      device.reset().await.unwrap();
      device.close().await.unwrap();
    })
  }

  #[test]
  fn test_device_control() {
    block_on(async move {
      async fn test(device: &mut UsbDevice) {
        let device_descriptor_bytes = device
          .control_transfer_in(
            UsbControlTransferParameters {
              request_type: crate::UsbRequestType::Standard,
              recipient: crate::UsbRecipient::Device,
              // kGetDescriptorRequest
              request: 0x06,
              // kDeviceDescriptorType
              value: 0x01 << 8,
              index: 0,
            },
            // kDeviceDescriptorLength
            18,
          )
          .await
          .unwrap();

        assert_eq!(device_descriptor_bytes.len(), 18);
        assert_eq!(device_descriptor_bytes[0], 18);

        let bcd_usb = u16::from_le_bytes([
          device_descriptor_bytes[2],
          device_descriptor_bytes[3],
        ]);

        assert_eq!((bcd_usb >> 8) as u8, device.usb_version_major);
        assert_eq!(((bcd_usb & 0xf0) >> 4) as u8, device.usb_version_minor);
        assert_eq!((bcd_usb & 0xf) as u8, device.usb_version_subminor);

        assert_eq!(device_descriptor_bytes[4], device.device_class);
        assert_eq!(device_descriptor_bytes[5], device.device_subclass);
        assert_eq!(device_descriptor_bytes[6], device.device_protocol);

        let vendor_id = u16::from_le_bytes([
          device_descriptor_bytes[8],
          device_descriptor_bytes[9],
        ]);

        assert_eq!(vendor_id, device.vendor_id);

        let product_id = u16::from_le_bytes([
          device_descriptor_bytes[10],
          device_descriptor_bytes[11],
        ]);

        assert_eq!(product_id, device.product_id);

        let bcd_device = u16::from_le_bytes([
          device_descriptor_bytes[12],
          device_descriptor_bytes[13],
        ]);

        assert_eq!((bcd_device >> 8) as u8, device.device_version_major);
        assert_eq!(
          ((bcd_device & 0xf0) >> 4) as u8,
          device.device_version_minor
        );
        assert_eq!((bcd_device & 0xf) as u8, device.device_version_subminor);

        assert_eq!(
          device_descriptor_bytes[17],
          device.configurations.len() as u8
        );
      }
      let mut device = test_device().await;

      device.open().await.unwrap();

      // A real world application should use `device.configuration.is_none()`.
      match device.select_configuration(1).await {
        Ok(_) => {} // Unreachable in the test runner
        Err(crate::Error::Usb(rusb::Error::Busy))
        | Err(crate::Error::InvalidState) => {}
        _ => unreachable!(),
      }

      // Device might be busy.
      if device.claim_interface(2).await.is_ok() {
        device.select_alternate_interface(2, 0).await.unwrap();

        device
          .control_transfer_out(
            UsbControlTransferParameters {
              request_type: crate::UsbRequestType::Class,
              recipient: crate::UsbRecipient::Interface,
              request: 0x22,
              value: 0x01,
              index: 2,
            },
            &[],
          )
          .await
          .unwrap();
        test(&mut device).await;
        device
          .control_transfer_out(
            UsbControlTransferParameters {
              request_type: crate::UsbRequestType::Class,
              recipient: crate::UsbRecipient::Interface,
              request: 0x22,
              value: 0x00,
              index: 2,
            },
            &[],
          )
          .await
          .unwrap();
      } else {
        test(&mut device).await;
      }
      device.release_interface(2).await.unwrap();
      device.reset().await.unwrap();
      device.close().await.unwrap();
    })
  }

  #[tokio::test]
  #[should_panic]
  // IMPORTANT! These are meant to fail when the methods are implemented.
  async fn test_unimplemented1() {
    let mut device = test_device().await;
    device.isochronous_transfer_in().await;
  }

  #[tokio::test]
  #[should_panic]
  // IMPORTANT! These are meant to fail when the methods are implemented.
  async fn test_unimplemented2() {
    let mut device = test_device().await;
    device.isochronous_transfer_out().await;
  }

  #[tokio::test]
  async fn test_device_not_found() -> crate::Result<()> {
    let mut device = test_device().await;

    device.open().await?;

    device.select_configuration(255).await.unwrap_err();
    device.claim_interface(255).await.unwrap_err();
    device.release_interface(255).await.unwrap_err();
    device.select_alternate_interface(255, 0).await.unwrap_err();

    device.close().await?;
    Ok(())
  }

  #[tokio::test]
  async fn test_validate_control_setup() {
    let mut device = test_device().await;
    device.open().await.unwrap();

    fn standard_ctrl_req(device: &mut UsbDevice) -> crate::Result<()> {
      device.validate_control_setup(&UsbControlTransferParameters {
        request_type: UsbRequestType::Class,
        recipient: UsbRecipient::Interface,
        request: 0x22,
        value: 0x01,
        index: 2,
      })
    }

    // Interface is not claimed.
    standard_ctrl_req(&mut device).unwrap_err();

    // Interface is claimed and selected.
    device.claim_interface(2).await.unwrap();
    standard_ctrl_req(&mut device).unwrap();
  }

  #[test]
  fn test_error_impl() {
    let nope: Option<()> = None;
    assert_eq!(Error::from(nope), Error::NotFound);
  }
}
