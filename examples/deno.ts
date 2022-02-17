import { get_devices, open, close } from "../bindings/bindings.ts";

class UsbDevice {
  #inner;
  
  constructor(raw) {
    this.#inner = raw;
  }

  get raw() {
    return this.#inner;
  }

  async open() {
    this.#inner = await open({ device: this.#inner });
  }
}

class WebUsb {
  async getDevices() {
    const { devices } = await get_devices();
    return devices.map(dev => new UsbDevice(dev));
  }
}

const usb = new WebUsb();

const devs = await usb.getDevices();
await devs[0].open();
console.log(devs[0].raw)