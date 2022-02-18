// Auto-generated with deno_bindgen
import { CachePolicy, prepare } from "https://deno.land/x/plug@0.5.1/plug.ts"
function encode(v: string | Uint8Array): Uint8Array {
  if (typeof v !== "string") return v
  return new TextEncoder().encode(v)
}
function decode(v: Uint8Array): string {
  return new TextDecoder().decode(v)
}
function readPointer(v: any): Uint8Array {
  const ptr = new Deno.UnsafePointerView(v as Deno.UnsafePointer)
  const lengthBe = new Uint8Array(4)
  const view = new DataView(lengthBe.buffer)
  ptr.copyInto(lengthBe, 0)
  const buf = new Uint8Array(view.getUint32(0))
  ptr.copyInto(buf, 4)
  return buf
}
const opts = {
  name: "webusb",
  url:
    (new URL(
      "https://github.com/littledivy/webusb/releases/download/0.4.0",
      import.meta.url,
    )).toString(),
  policy: undefined,
}
const _lib = await prepare(opts, {
  claim_interface: {
    parameters: ["pointer", "usize", "u8"],
    result: "pointer",
    nonblocking: false,
  },
  clear_halt: {
    parameters: ["pointer", "usize", "pointer", "usize", "u8"],
    result: "pointer",
    nonblocking: false,
  },
  close: {
    parameters: ["pointer", "usize"],
    result: "pointer",
    nonblocking: false,
  },
  control_transfer_in: {
    parameters: ["pointer", "usize", "pointer", "usize", "usize"],
    result: "pointer",
    nonblocking: false,
  },
  control_transfer_out: {
    parameters: ["pointer", "usize", "pointer", "usize", "pointer", "usize"],
    result: "usize",
    nonblocking: false,
  },
  get_devices: { parameters: [], result: "pointer", nonblocking: true },
  open: {
    parameters: ["pointer", "usize"],
    result: "pointer",
    nonblocking: false,
  },
  release_interface: {
    parameters: ["pointer", "usize", "u8"],
    result: "pointer",
    nonblocking: false,
  },
  reset: {
    parameters: ["pointer", "usize"],
    result: "pointer",
    nonblocking: false,
  },
  select_alternate_interface: {
    parameters: ["pointer", "usize", "u8", "u8"],
    result: "pointer",
    nonblocking: false,
  },
  select_configuration: {
    parameters: ["pointer", "usize", "u8"],
    result: "pointer",
    nonblocking: false,
  },
  transfer_in: {
    parameters: ["pointer", "usize", "u8", "usize"],
    result: "pointer",
    nonblocking: false,
  },
  transfer_out: {
    parameters: ["pointer", "usize", "u8", "pointer", "usize"],
    result: "void",
    nonblocking: false,
  },
})
export type UsbEndpointType =
  | "bulk"
  | "interrupt"
  | "isochronous"
  | "control"
export type Devices = {
  devices: Array<UsbDevice>
}
export type UsbAlternateInterface = {
  alternateSetting: number
  interfaceClass: number
  interfaceSubclass: number
  interfaceProtocol: number
  interfaceName: string | undefined | null
  endpoints: Array<UsbEndpoint>
}
export type Direction =
  | "in"
  | "out"
export type UsbEndpoint = {
  endpointNumber: number
  direction: Direction
  type: UsbEndpointType
  packetSize: number
}
/**
 * Represents a UsbDevice.
 * Only way you can obtain one is through `Context::devices`
 * https://wicg.github.io/webusb/#device-usage
 */
export type UsbDevice = {
  /**
   * List of configurations supported by the device.
   * Populated from the configuration descriptor.
   * `configurations.len()` SHALL be equal to the
   * bNumConfigurations field of the device descriptor.
   */
  configurations: Array<UsbConfiguration>
  /**
   * Represents the currently selected configuration.
   * One of the elements of `self.configurations`.
   * None, if the device is not configured.
   */
  configuration: UsbConfiguration | undefined | null
  /**
   * bDeviceClass value of the device descriptor.
   */
  deviceClass: number
  /**
   * bDeviceSubClass value of the device descriptor.
   */
  deviceSubclass: number
  /**
   * bDeviceProtocol value of the device descriptor.
   */
  deviceProtocol: number
  /**
   * The major version declared by bcdDevice field
   * such that bcdDevice 0xJJMN represents major version JJ.
   */
  deviceVersionMajor: number
  /**
   * The minor version declared by bcdDevice field
   * such that bcdDevice 0xJJMN represents minor version M.
   */
  deviceVersionMinor: number
  /**
   * The subminor version declared by bcdDevice field
   * such that bcdDevice 0xJJMN represents subminor version N.
   */
  deviceVersionSubminor: number
  /**
   * Optional property of the string descriptor.
   * Indexed by the iManufacturer field of device descriptor.
   */
  manufacturerName: string | undefined | null
  /**
   * idProduct field of the device descriptor.
   */
  productId: number
  /**
   * Optional property of the string descriptor.
   * Indexed by the iProduct field of device descriptor.
   */
  productName: string | undefined | null
  /**
   * Optional property of the string descriptor.
   * None, if the iSerialNumber field of device descriptor
   * is 0.
   */
  serialNumber: string | undefined | null
  /**
   * The major version declared by bcdUSB field
   * such that bcdUSB 0xJJMN represents major version JJ.
   */
  usbVersionMajor: number
  /**
   * The minor version declared by bcdUSB field
   * such that bcdUSB 0xJJMN represents minor version M.
   */
  usbVersionMinor: number
  /**
   * The subminor version declared by bcdUSB field
   * such that bcdUSB 0xJJMN represents subminor version N.
   */
  usbVersionSubminor: number
  /**
   * idVendor field of the device descriptor.
   * https://wicg.github.io/webusb/#vendor-id
   */
  vendorId: number
  /**
   * If true, the underlying device handle is owned by this object.
   */
  opened: boolean
  /**
   * WEBUSB_URL value of the WebUSB Platform Capability Descriptor.
   */
  url: string | undefined | null
  /**
   * Resource ID associated with this Device instance.
   */
  rid: number
  device: Device<Context>
  deviceHandle: DeviceHandle<Context> | undefined | null
}
export type UsbRequestType =
  | "standard"
  | "class"
  | "vendor"
export type UsbInterface = {
  interfaceNumber: number
  alternate: UsbAlternateInterface
  alternates: Array<UsbAlternateInterface>
  claimed: boolean
}
export type FfiDirection = {
  inner: Direction
}
export type UsbRecipient =
  | "device"
  | "interface"
  | "endpoint"
  | "other"
export type UsbConfiguration = {
  configurationName: string | undefined | null
  configurationValue: number
  interfaces: Array<UsbInterface>
}
export type Device = {
  device: UsbDevice
}
export type FfiUsbControlTransferParameters = {
  inner: UsbControlTransferParameters
}
export type UsbControlTransferParameters = {
  requestType: UsbRequestType
  recipient: UsbRecipient
  request: number
  value: number
  index: number
}
export function claim_interface(a0: Device, a1: number) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.claim_interface(a0_buf, a0_buf.byteLength, a1)
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function clear_halt(a0: Device, a1: FfiDirection, a2: number) {
  const a0_buf = encode(JSON.stringify(a0))
  const a1_buf = encode(JSON.stringify(a1))
  let rawResult = _lib.symbols.clear_halt(
    a0_buf,
    a0_buf.byteLength,
    a1_buf,
    a1_buf.byteLength,
    a2,
  )
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function close(a0: Device) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.close(a0_buf, a0_buf.byteLength)
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function control_transfer_in(
  a0: Device,
  a1: FfiUsbControlTransferParameters,
  a2: number,
) {
  const a0_buf = encode(JSON.stringify(a0))
  const a1_buf = encode(JSON.stringify(a1))
  let rawResult = _lib.symbols.control_transfer_in(
    a0_buf,
    a0_buf.byteLength,
    a1_buf,
    a1_buf.byteLength,
    a2,
  )
  const result = readPointer(rawResult)
  return result
}
export function control_transfer_out(
  a0: Device,
  a1: FfiUsbControlTransferParameters,
  a2: Uint8Array,
) {
  const a0_buf = encode(JSON.stringify(a0))
  const a1_buf = encode(JSON.stringify(a1))
  const a2_buf = encode(a2)
  let rawResult = _lib.symbols.control_transfer_out(
    a0_buf,
    a0_buf.byteLength,
    a1_buf,
    a1_buf.byteLength,
    a2_buf,
    a2_buf.byteLength,
  )
  const result = rawResult
  return result
}
export function get_devices() {
  let rawResult = _lib.symbols.get_devices()
  const result = rawResult.then(readPointer)
  return result.then(r => JSON.parse(decode(r))) as Promise<Devices>
}
export function open(a0: Device) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.open(a0_buf, a0_buf.byteLength)
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function release_interface(a0: Device, a1: number) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.release_interface(a0_buf, a0_buf.byteLength, a1)
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function reset(a0: Device) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.reset(a0_buf, a0_buf.byteLength)
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function select_alternate_interface(a0: Device, a1: number, a2: number) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.select_alternate_interface(
    a0_buf,
    a0_buf.byteLength,
    a1,
    a2,
  )
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function select_configuration(a0: Device, a1: number) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.select_configuration(
    a0_buf,
    a0_buf.byteLength,
    a1,
  )
  const result = readPointer(rawResult)
  return JSON.parse(decode(result)) as Device
}
export function transfer_in(a0: Device, a1: number, a2: number) {
  const a0_buf = encode(JSON.stringify(a0))
  let rawResult = _lib.symbols.transfer_in(a0_buf, a0_buf.byteLength, a1, a2)
  const result = readPointer(rawResult)
  return result
}
export function transfer_out(a0: Device, a1: number, a2: Uint8Array) {
  const a0_buf = encode(JSON.stringify(a0))
  const a2_buf = encode(a2)
  let rawResult = _lib.symbols.transfer_out(
    a0_buf,
    a0_buf.byteLength,
    a1,
    a2_buf,
    a2_buf.byteLength,
  )
  const result = rawResult
  return result
}
