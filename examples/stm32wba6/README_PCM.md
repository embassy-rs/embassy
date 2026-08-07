# SD to SAI PCM Streaming

This example streams audio from SD card to SAI using raw PCM files.

## File format

Raw PCM files (.pcm) must be:
- Unsigned 16-bit little-endian
- 48 kHz
- Mono

## Convert WAV to PCM

Use the Python helper:

```bash
python3 convert_wav.py input.wav output.pcm
```

If your WAV is not 48 kHz, resample it first (for example with ffmpeg):

```bash
ffmpeg -i input.wav -ar 48000 -ac 1 temp.wav
```

Then convert `temp.wav` to PCM with the script above.

## Usage

1. Copy a `.pcm` or `.wav` file to the root of the SD card.
2. Flash and run:

```bash
cargo run --bin sdmmc_sai --release
```

The example plays the first `.pcm` or `.wav` file found in the SD card root.
