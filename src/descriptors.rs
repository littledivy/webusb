use crate::constants::*;

macro_rules! assert_return {
  ($e: expr) => {
    if $e {
      return None;
    }
  };
}

// Based on Chromium implementation https://source.chromium.org/chromium/chromium/src/+/main:services/device/usb/webusb_descriptors.cc;l=133;
// https://wicg.github.io/webusb/#webusb-platform-capability-descriptor
pub(crate) fn parse_bos(bytes: &[u8]) -> Option<(u8, u8)> {
  // Too short
  assert_return!(bytes.len() < 5);

  let total_length = bytes[2] + (bytes[3].wrapping_shl(8));

  // Validate BOS header
  // bLength
  assert_return!(bytes[0] != 5);
  // bDescriptorType
  assert_return!(bytes[1] != BOS_DESCRIPTOR_TYPE as u8);
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
    assert_return!(bytes[1] != DEVICE_CAPABILITY_DESCRIPTOR_TYPE);

    // bDevCapabilityType
    if bytes[2] != PLATFORM_DEV_CAPABILITY_TYPE {
      continue;
    }

    // atleast 20 bytes
    assert_return!(length < 20);

    // PlatformCapabilityUUID
    if &bytes[4..20] != WEB_USB_CAPABILITY_UUID {
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

// http://wicg.github.io/webusb/#dfn-url-descriptor
pub(crate) fn parse_webusb_url(bytes: &[u8]) -> Option<String> {
  assert_return!(bytes.len() < DESCRIPTOR_MIN_LENGTH as usize);

  let length = bytes[0];
  assert_return!(length < DESCRIPTOR_MIN_LENGTH);
  assert_return!(length as usize > bytes.len());
  assert_return!(bytes[1] != DESCRIPTOR_TYPE);

  let mut url = match bytes[2] {
    0 => String::from("http://"),
    1 => String::from("https://"),
    _ => return None,
  };

  url.push_str(&String::from_utf8_lossy(&bytes[3..length as usize]));
  Some(url)
}

#[cfg(test)]
mod tests {
  use crate::descriptors::parse_bos;
  use crate::descriptors::parse_webusb_url;

  #[test]
  fn test_parse_bos() {
    assert_eq!(
      parse_bos(&[
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
      ]),
      Some((0x42, 0x01))
    );
  }

  #[test]
  fn test_parse_url_descriptor() {
    assert_eq!(
      parse_webusb_url(&[
        0x19, 0x03, 0x01, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c',
        b'o', b'm', b'/', b'i', b'n', b'd', b'e', b'x', b'.', b'h', b't', b'm',
        b'l',
      ]),
      Some("https://example.com/index.html".to_string())
    );
  }

  // TODO(@littledivy): Import more tests from https://source.chromium.org/chromium/chromium/src/+/main:services/device/usb/webusb_descriptors_unittest.cc
}
