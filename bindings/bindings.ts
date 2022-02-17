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
  url: (new URL("../target/debug", import.meta.url)).toString(),
  policy: CachePolicy.NONE,
}
const _lib = await prepare(opts, {})
export type UsbInterface = {
  interfaceNumber: number
  alternate: UsbAlternateInterface
  alternates: Array<UsbAlternateInterface>
  claimed: boolean
}
export type Direction =
  | "in"
  | "out"
export type UsbRecipient =
  | "device"
  | "interface"
  | "endpoint"
  | "other"
export type UsbEndpoint = {
  endpointNumber: number
  direction: Direction
  type: UsbEndpointType
  packetSize: number
}
export type UsbControlTransferParameters = {
  requestType: UsbRequestType
  recipient: UsbRecipient
  request: number
  value: number
  index: number
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
  rid: isize
  device: Device<Context>
  device: UsbDevice
  deviceHandle: DeviceHandle<Context> | undefined | null
}
export type UsbConfiguration = {
  configurationName: string | undefined | null
  configurationValue: number
  interfaces: Array<UsbInterface>
}
export type UsbAlternateInterface = {
  alternateSetting: number
  interfaceClass: number
  interfaceSubclass: number
  interfaceProtocol: number
  interfaceName: string | undefined | null
  endpoints: Array<UsbEndpoint>
}
export type UsbRequestType =
  | "standard"
  | "class"
  | "vendor"
export type UsbEndpointType =
  | "bulk"
  | "interrupt"
  | "isochronous"
  | "control"
