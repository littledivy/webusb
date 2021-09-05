pub const DEVICE_CAPABILITY_DESCRIPTOR_TYPE: u8 = 0x10;
pub const PLATFORM_DEV_CAPABILITY_TYPE: u8 = 0x05;
pub const GET_URL_REQUEST: u16 = 0x02;
/// Little-endian encoding of {3408b638-09a9-47a0-8bfd-a0768815b665}.
pub const WEB_USB_CAPABILITY_UUID: &[u8; 16] = &[
  0x38, 0xB6, 0x08, 0x34, 0xA9, 0x09, 0xA0, 0x47, 0x8B, 0xFD, 0xA0, 0x76, 0x88,
  0x15, 0xB6, 0x65,
];
pub const BOS_DESCRIPTOR_TYPE: u16 = 0x0F;
pub const DESCRIPTOR_TYPE: u8 = 0x03;
pub const DESCRIPTOR_MIN_LENGTH: u8 = 3;
