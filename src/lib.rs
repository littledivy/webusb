use serde::Deserialize;
use serde::Serialize;

use rusb::UsbContext;

use core::convert::TryFrom;

pub use rusb;

pub mod constants;
mod descriptors;

use crate::constants::BOS_DESCRIPTOR_TYPE;
use crate::constants::GET_URL_REQUEST;
use crate::descriptors::parse_bos;
use crate::descriptors::parse_webusb_url;

#[non_exhaustive]
pub enum Error {
  Usb(rusb::Error),
  NotFound,
  InvalidState,
}

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsbConfiguration {
  // Index of String Descriptor describing this configuration.
  configuration_name: Option<String>,
  // The configuration number (bConfigurationValue)
  // https://www.beyondlogic.org/usbnutshell/usb5.shtml#ConfigurationDescriptors
  configuration_value: u8,
  interfaces: Vec<UsbInterface>,
}

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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsbInterface {
  interface_number: u8,
  alternate: UsbAlternateInterface,
  alternates: Vec<UsbAlternateInterface>,
  claimed: bool,
}

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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UsbEndpointType {
  Bulk,
  Interrupt,
  Isochronous,
  Control,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
  In,
  Out,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsbEndpoint {
  endpoint_number: u8,
  direction: Direction,
  // TODO(@littledivy): Get rid of reserved `type` key somehow?
  r#type: UsbEndpointType,
  packet_size: u16,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsbAlternateInterface {
  alternate_setting: u8,
  interface_class: u8,
  interface_subclass: u8,
  interface_protocol: u8,
  interface_name: Option<String>,
  endpoints: Vec<UsbEndpoint>,
}

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

/// Represents a WebUSB UsbDevice.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
  pub configurations: Vec<UsbConfiguration>,
  pub configuration: Option<UsbConfiguration>,
  pub device_class: u8,
  pub device_subclass: u8,
  pub device_protocol: u8,
  pub device_version_major: u8,
  pub device_version_minor: u8,
  pub device_version_subminor: u8,
  pub manufacturer_name: Option<String>,
  pub product_id: u16,
  pub product_name: Option<String>,
  pub serial_number: Option<String>,
  pub usb_version_major: u8,
  pub usb_version_minor: u8,
  pub usb_version_subminor: u8,
  pub vendor_id: u16,
  pub opened: bool,
  /// The `WEBUSB_URL` value. Present in devices with the WebUSB Platform Capability Descriptor.
  #[serde(skip)]
  pub url: Option<String>,
  #[serde(skip)]
  device: rusb::Device<rusb::Context>,
  #[serde(skip)]
  device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

impl UsbDevice {
  pub fn open(&mut self) -> Result<()> {
    // 3. device is already open?
    if self.opened {
      return Ok(());
    }

    // 4.
    let handle = self.device.open()?;
    self.device_handle = Some(handle);

    // 5.
    self.opened = true;
    Ok(())
  }

  pub fn close(&mut self) -> Result<()> {
    // 3. device is already closed?
    if !self.opened {
      return Ok(());
    }

    match &self.device_handle {
      Some(handle_ref) => {
        // 5-6.
        // release claimed interfaces, close device and release handle
        drop(handle_ref);
      }
      None => unreachable!(),
    };

    self.device_handle = None;

    // 7.
    self.opened = false;
    Ok(())
  }

  /// `configuration_value` is the bConfigurationValue of the device configuration.
  pub fn select_configuration(
    &mut self,
    configuration_value: u8,
  ) -> Result<()> {
    // 3.
    let configuration = match self
      .configurations
      .iter()
      .find(|c| c.configuration_value == configuration_value)
    {
      Some(_) => self.device.config_descriptor(configuration_value)?,
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
    self.configuration = Some(UsbConfiguration::from(configuration, &handle)?);

    Ok(())
  }

  pub fn claim_interface(&mut self, interface_number: u8) -> Result<()> {
    // 2.
    let mut interface = match self.configurations.iter_mut().find_map(|c| {
      c.interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
    }) {
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

    Ok(())
  }

  pub fn release_interface(&mut self, interface_number: u8) -> Result<()> {
    // 3.
    let mut interface = match self.configurations.iter_mut().find_map(|c| {
      c.interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
    }) {
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

    Ok(())
  }

  pub fn select_alternate_interface(
    &mut self,
    interface_number: u8,
    alternate_setting: u8,
  ) -> Result<()> {
    // 3.
    let mut interface = match self.configurations.iter_mut().find_map(|c| {
      c.interfaces
        .iter_mut()
        .find(|i| i.interface_number == interface_number)
    }) {
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

    // 7.
    return Ok(());
  }

  pub fn control_transfer_in(
    &mut self,
    setup: USBControlTransferParameters,
    length: usize,
  ) -> Result<Vec<u8>> {
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
          USBRequestType::Standard => rusb::RequestType::Standard,
          USBRequestType::Class => rusb::RequestType::Class,
          USBRequestType::Vendor => rusb::RequestType::Vendor,
        };

        let recipient = match setup.recipient {
          USBRecipient::Device => rusb::Recipient::Device,
          USBRecipient::Interface => rusb::Recipient::Interface,
          USBRecipient::Endpoint => rusb::Recipient::Endpoint,
          USBRecipient::Other => rusb::Recipient::Other,
        };

        let req_type = rusb::request_type(rusb::Direction::In, req, recipient);

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
    // a USBInTransferResult.
    let result = &buffer[0..bytes_transferred];

    // 10-11. TODO: Will need to handle `read_control` Err

    // 13.
    Ok(result.to_vec())
  }

  pub fn control_transfer_out(
    &mut self,
    setup: USBControlTransferParameters,
    data: &[u8],
  ) -> Result<usize> {
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
          USBRequestType::Standard => rusb::RequestType::Standard,
          USBRequestType::Class => rusb::RequestType::Class,
          USBRequestType::Vendor => rusb::RequestType::Vendor,
        };

        let recipient = match setup.recipient {
          USBRecipient::Device => rusb::Recipient::Device,
          USBRecipient::Interface => rusb::Recipient::Interface,
          USBRecipient::Endpoint => rusb::Recipient::Endpoint,
          USBRecipient::Other => rusb::Recipient::Other,
        };

        let req_type = rusb::request_type(rusb::Direction::Out, req, recipient);

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

  // https://wicg.github.io/webusb/#check-the-validity-of-the-control-transfer-parameters
  fn validate_control_setup(
    &mut self,
    setup: &USBControlTransferParameters,
  ) -> Result<()> {
    // 3.
    if let Some(configuration) = &self.configuration {
      match setup.recipient {
        // 4.
        USBRecipient::Interface => {
          // 4.1
          let interface_number: u8 = setup.index as u8 & 0xFF;

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
        USBRecipient::Endpoint => {
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

  pub fn clear_halt(
    &mut self,
    direction: Direction,
    endpoint_number: u8,
  ) -> Result<()> {
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
        const EP_DIR_IN: u8 = 0x80;
        const EP_DIR_OUT: u8 = 0x0;

        let mut endpoint = endpoint_number;

        match direction {
          Direction::In => endpoint |= EP_DIR_IN,
          Direction::Out => endpoint |= EP_DIR_OUT,
        };

        handle_ref.clear_halt(endpoint)?
      }
      None => unreachable!(),
    };

    Ok(())
  }

  pub fn transfer_in(&mut self) {}
  pub fn transfer_out(&mut self) {}

  pub fn isochronous_transfer_in(&mut self) {
    unimplemented!()
  }

  pub fn isochronous_transfer_out(&mut self) {
    unimplemented!()
  }

  pub fn reset(&mut self) -> Result<()> {
    // 3.
    if !self.opened {
      return Err(Error::InvalidState);
    }

    // 4-6.
    match self.device_handle {
      Some(ref mut handle_ref) => handle_ref.reset()?,
      None => unreachable!(),
    };

    Ok(())
  }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
enum USBRequestType {
  Standard,
  Class,
  Vendor,
}

#[derive(Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum USBRecipient {
  Device,
  Interface,
  Endpoint,
  Other,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct USBControlTransferParameters {
  request_type: USBRequestType,
  recipient: USBRecipient,
  request: u8,
  value: u16,
  index: u16,
}

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

/// Method to determine the transfer type from the device's
/// configuration descriptor and an endpoint address.
pub fn transfer_type(
  cnf: rusb::ConfigDescriptor,
  addr: u8,
) -> Option<rusb::TransferType> {
  let interfaces = cnf.interfaces();
  for interface in interfaces {
    for descriptor in interface.descriptors() {
      let endpoint_desc = descriptor
        .endpoint_descriptors()
        .find(|s| s.address() == addr);
      if let Some(endpoint_desc) = endpoint_desc {
        return Some(endpoint_desc.transfer_type());
      }
    }
  }
  None
}

/// A WebUSB Context. Provides APIs for device enumaration.
pub struct Context(rusb::Context);

impl Context {
  pub fn new() -> Result<Self> {
    let ctx = rusb::Context::new()?;
    Ok(Self(ctx))
  }

  pub fn devices(&self) -> Result<Vec<UsbDevice>> {
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
      .collect::<Result<Vec<UsbDevice>>>()?;
    Ok(usb_devices)
  }
}
