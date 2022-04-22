import "../mod.ts";

const devices = await navigator.usb.getDevices();
// Arduino Leonardo
let device = devices.find((p) => p.productId == 0x8036);

await device.open();
console.log("Device opened.");

if (device.configuration === null) {
  device.selectConfiguration(1);
}

console.log(`${device.productName} - ${device.serialNumber}`);

await device.claimInterface(2);
await device.selectAlternateInterface(2, 0);
await device.controlTransferOut({
  "requestType": "class",
  "recipient": "interface",
  "request": 0x22,
  "value": 0x01,
  "index": 2,
});

while (true) {
  const action = prompt(">>");
  if (action.toLowerCase() == "exit") break;
  const data = new TextEncoder().encode(action);
  await device.transferOut(4, data);
  console.info("Transfer.");
}

await device.controlTransferOut({
  "requestType": "class",
  "recipient": "interface",
  "request": 0x22,
  "value": 0x00,
  "index": 2,
});

await device.close();
console.log("Bye.");
