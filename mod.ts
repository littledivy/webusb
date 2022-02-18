import {
  claim_interface,
  clear_halt,
  close,
  control_transfer_in,
  control_transfer_out,
  Direction,
  get_devices,
  open,
  release_interface,
  reset,
  select_alternate_interface,
  select_configuration,
  transfer_in,
  transfer_out,
  UsbControlTransferParameters,
} from "./bindings/bindings.ts";

class UsbDevice {
  #inner;

  constructor(raw) {
    this.#inner = { device: raw };
  }

  get raw() {
    return this.#inner.device;
  }

  get configurations() {
    return this.#inner.device.configurations;
  }
  get configuration() {
    return this.#inner.device.configuration;
  }

  get deviceClass() {
    return this.#inner.device.deviceClass;
  }

  get deviceSubclass() {
    return this.#inner.device.deviceSubclass;
  }

  get deviceProtocol() {
    return this.#inner.device.deviceProtocol;
  }

  get deviceVersionMajor() {
    return this.#inner.device.deviceVersionMajor;
  }

  get deviceVersionMinor() {
    return this.#inner.device.deviceVersionMinor;
  }

  get deviceVersionSubminor() {
    return this.#inner.device.deviceVersionSubminor;
  }

  get manufacturerName() {
    return this.#inner.device.manufacturerName;
  }

  get productId() {
    return this.#inner.device.productId;
  }

  get productName() {
    return this.#inner.device.productName;
  }

  get serialNumber() {
    return this.#inner.device.serialNumber;
  }

  get usbVersionMajor() {
    return this.#inner.device.usbVersionMajor;
  }

  get usbVersionMinor() {
    return this.#inner.device.usbVersionMinor;
  }

  get usbVersionSubminor() {
    return this.#inner.device.usbVersionSubminor;
  }

  get vendorId() {
    return this.#inner.device.vendorId;
  }

  get opened() {
    return this.#inner.opened;
  }

  get url() {
    return this.#inner.url;
  }

  async open() {
    this.#inner = await open(this.#inner);
  }

  async reset() {
    this.#inner = await reset(this.#inner);
  }

  async close() {
    this.#inner = await close(this.#inner);
  }

  async transferIn(endpointNumber: number, length: number) {
    const pointer = await transfer_in(
      this.#inner,
      endpointNumber,
      length,
    );
    const view = new Deno.UnsafePointerView(pointer);
    const u8 = new Uint8Array(length);
    view.copyInto(u8);
    return u8;
  }

  async transferOut(endpointNumber: number, data: Uint8Array) {
    await transfer_out(this.#inner, endpointNumber, data);
  }

  async controlTransferIn(
    setup: UsbControlTransferParameters,
    length: number,
  ): Promise<Uint8Array> {
    const ptr = await control_transfer_in(
      this.#inner,
      { inner: setup },
    );
    const view = new Deno.UnsafePointerView(ptr);
    const u8 = new Uint8Array(length);
    view.copyInto(u8);
    return u8;
  }

  controlTransferOut(
    setup: UsbControlTransferParameters,
    data?: Uint8Array,
  ): Promise<number> {
    return control_transfer_out(
      this.#inner,
      { inner: setup },
      data || new Uint8Array(),
    );
  }

  async clearHalt(
    direction: Direction,
    endpointNumber: number,
  ) {
    await clear_halt(
      this.#inner,
      { inner: direction },
      endpointNumber,
    );
  }

  async selectAlternateInterface(
    interfaceNumber: number,
    alternateSetting: number,
  ) {
    this.#inner = await select_alternate_interface(
      this.#inner,
      interfaceNumber,
      alternateSetting,
    );
  }

  async selectConfiguration(configurationValue: number) {
    this.#inner = await select_configuration(this.#inner, configurationValue);
  }

  async releaseInterface(
    interfaceNumber: number,
  ) {
    this.#inner = await release_interface(
      this.#inner,
      interfaceNumber,
    );
  }

  async claimInterface(
    interfaceNumber: number,
  ) {
    this.#inner = await claim_interface(
      this.#inner,
      interfaceNumber,
    );
  }
}

class Usb {
  async getDevices() {
    const { devices } = await get_devices();
    return devices.map((dev) => new UsbDevice(dev));
  }
}

const usb = new Usb();
navigator.usb = usb;
export default usb;
