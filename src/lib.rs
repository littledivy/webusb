use serde::Deserialize;
use serde::Serialize;

use rusb::UsbContext;

pub use rusb;

#[non_exhaustive]
pub enum Error {
  Usb(rusb::Error),
  NotFound,
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

#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsbDevice {
  configurations: Vec<UsbConfiguration>,
  configuration: Option<UsbConfiguration>,
  device_class: u8,
  device_subclass: u8,
  device_protocol: u8,
  device_version_major: u8,
  device_version_minor: u8,
  device_version_subminor: u8,
  manufacturer_name: Option<String>,
  product_id: u16,
  product_name: Option<String>,
  serial_number: Option<String>,
  usb_version_major: u8,
  usb_version_minor: u8,
  usb_version_subminor: u8,
  vendor_id: u16,
}

// Method to determine the transfer type from the device's
// configuration descriptor and an endpoint address.
fn transfer_type(
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

const kBosDescriptorType: u16 = 0x0F;

macro_rules! assert_return {
  ($e: expr) => {
    if $e {
      return None;
    }
  };
}

const kDeviceCapabilityDescriptorType: u8 = 0x10;
const kPlatformDevCapabilityType: u8 = 0x05;
const kWebUsbCapabilityUUID: &[u8; 16] = &[
  // Little-endian encoding of {3408b638-09a9-47a0-8bfd-a0768815b665}.
  0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47, 0x8B, 0xFD, 0xA0, 0x76, 0x88,
  0x15, 0xB6, 0x65,
];

// Based on Chromium implementation https://source.chromium.org/chromium/chromium/src/+/main:services/device/usb/webusb_descriptors.cc;l=133;
pub(crate) fn parse_bos(bytes: &[u8]) -> Option<(u8, u8)> {
  // Too short
  assert_return!(bytes.len() < 5);

  let total_length = bytes[2] + (bytes[3].wrapping_shl(8));

  // Validate BOS header
  // bLength
  assert_return!(bytes[0] != 5);
  // bDescriptorType
  assert_return!(bytes[1] != kBosDescriptorType as u8);
  // wTotalLength
  assert_return!(5_u8 > total_length || total_length as usize > bytes.len());

  // bNumDeviceCaps
  let num_device_caps = bytes[4];
  let end = bytes[0] + total_length;

  let mut bytes = &bytes[5..];

  let mut length = 0;
  for i in 0..num_device_caps {
    bytes = &bytes[length..];

    assert_return!(i == end);

    length = bytes[0] as usize;
    // bLength
    assert_return!(length < 3);
    assert_return!(bytes.len() < length);
    // bDescriptorType
    assert_return!(bytes[1] != kDeviceCapabilityDescriptorType);

    // bDevCapabilityType
    if bytes[2] != kPlatformDevCapabilityType {
      continue;
    }

    // atleast 20 bytes
    assert_return!(length < 20);

    // PlatformCapabilityUUID
    if &bytes[4..20] != kWebUsbCapabilityUUID {
      continue;
    }

    // The WebUSB capability descriptor must be at least 22 bytes (to allow for future versions).
    assert_return!(length < 22);

    // bcdVersion
    let version = bytes[20] as u16 + ((bytes[21] as u16) << 8);
    if version < 0x0100 {
      continue;
    }

    // Version 1.0 defines two fields for a total length of 24 bytes.
    assert_return!(length != 24);

    let vendor_code = bytes[22];
    let landing_page_id = bytes[23];

    return Some((vendor_code, landing_page_id));
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
    let mut usbdevices: Vec<UsbDevice> = vec![];
    let devices = self.0.devices()?;
    for device in devices.iter() {
      let device_descriptor = device.device_descriptor()?;
      let device_class = device_descriptor.class_code();
      // Do not list hubs. Ignore them.
      if device_class == 9 {
        continue;
      }

      let usb_version = device_descriptor.usb_version();

      let config_descriptor = device.active_config_descriptor();
      if let Ok(handle) = device.open() {
        let read_bos_descriptors = usb_version.0 >= 2 && usb_version.1 >= 1;
        if read_bos_descriptors {
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
            kBosDescriptorType << 8,
            0,
            &mut buffer,
            core::time::Duration::new(2, 0),
          )?;
          assert_eq!(length, 5);

          // Read BOS descriptor
          let new_length = (buffer[2] | (buffer[3].wrapping_shl(8)));
          let mut new_buffer = vec![0; new_length as usize];
          let request_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Standard,
            rusb::Recipient::Device,
          );
          handle.read_control(
            request_type,
            kGetDescriptorRequest,
            kBosDescriptorType << 8,
            0,
            &mut new_buffer,
            core::time::Duration::new(2, 0),
          )?;

          // Parse capibility from BOS descriptor
          parse_bos(&new_buffer);
        }

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

        let usbdevice = UsbDevice {
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
        };

        // Explicitly close the device.
        drop(handle);

        usbdevices.push(usbdevice);
      }
    }
    Ok(usbdevices)
  }
}

#[cfg(test)]
mod tests {
  use crate::parse_bos;

  const kExampleBosDescriptor: &[u8] = &[
    // BOS descriptor.
    0x05, 0x0F, 0x4C, 0x00, 0x03, // Container ID descriptor.
    0x14, 0x10, 0x04, 0x00, 0x2A, 0xF9, 0xF6, 0xC2, 0x98, 0x10, 0x2B, 0x49,
    0x8E, 0x64, 0xFF, 0x01, 0x0C, 0x7F, 0x94, 0xE1,
    // WebUSB Platform Capability descriptor.
    0x18, 0x10, 0x05, 0x00, 0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47,
    0x8B, 0xFD, 0xA0, 0x76, 0x88, 0x15, 0xB6, 0x65, 0x00, 0x01, 0x42, 0x01,
    // Microsoft OS 2.0 Platform Capability descriptor.
    0x1C, 0x10, 0x05, 0x00, 0xDF, 0x60, 0xDD, 0xD8, 0x89, 0x45, 0xC7, 0x4C,
    0x9C, 0xD2, 0x65, 0x9D, 0x9E, 0x64, 0x8A, 0x9F, 0x00, 0x00, 0x03, 0x06,
    0x00, 0x00, 0x01, 0x00,
  ];

  #[test]
  fn test_parse_bos() {
    assert_eq!(parse_bos(kExampleBosDescriptor), Some((0x42, 0x01)));
  }

  // TODO(@littledivy): Import more tests from https://source.chromium.org/chromium/chromium/src/+/main:services/device/usb/webusb_descriptors_unittest.cc
}
