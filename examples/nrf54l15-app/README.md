# nrf54l15-app examples

Examples for the application core of the nRF54L15.

## sQSPI firmware blob license

The `sqspi` example uses the **sQSPI soft peripheral**, a QSPI/SPI controller
implemented in firmware that runs on the nRF54L15's FLPR coprocessor. The
firmware blob `src/bin/sqspi_firmware.bin` is redistributed from Nordic
Semiconductor's nRF Connect SDK (nrfxlib) and is **not** covered by embassy's
license:

```
Copyright (c) 2025 Nordic Semiconductor ASA

SPDX-License-Identifier: LicenseRef-Nordic-5-Clause
```

See the [Nordic 5-Clause License](https://github.com/nrfconnect/sdk-nrfxlib/blob/main/LICENSE)
for the terms. All other files in this crate are licensed under embassy's usual
terms (MIT or Apache-2.0).
