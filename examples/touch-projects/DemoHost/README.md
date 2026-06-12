# DemoHost touch project

Host (SDL) defaults for the OxivGL hall lighting CAN demo.

- Same UI strings as [`Demo`](../Demo/) (`hall_config.json`)
- SocketCAN on **`vcan0`** (`can_config.json`) — set via `TOUCH_PROJECT=DemoHost` in
  `examples/oxivgl-host/.cargo/config.toml`

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set vcan0 up
cd examples/oxivgl-host
cargo run --bin oxivgl_touch_can
```
