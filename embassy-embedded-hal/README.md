# embassy-embedded-hal

Collection of utilities to use `embedded-hal` and `embedded-storage` traits with Embassy.

- Shared SPI and I2C buses, both blocking and async, with a `SetConfig` trait allowing changing bus configuration (e.g. frequency) between devices on the same bus.
- Async utilities
    - Adapters to convert from blocking to (fake) async.
    - Adapters to insert yields on trait operations.
- Flash utilities
    - Split a flash memory into smaller partitions.
    - Concatenate flash memories together.
    - Simulated in-memory flash.
