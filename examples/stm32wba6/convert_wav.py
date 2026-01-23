#!/usr/bin/env python3

import os
import struct
import sys
import wave


def convert_wav_to_pcm(input_file: str, output_file: str) -> bool:
    """Convert a WAV file to raw unsigned 16-bit PCM."""

    print(f"Converting {input_file} -> {output_file}")

    try:
        with wave.open(input_file, "rb") as wav_file:
            channels = wav_file.getnchannels()
            sample_width = wav_file.getsampwidth()
            sample_rate = wav_file.getframerate()
            frames = wav_file.getnframes()

            print(
                f"WAV: {channels}ch, {sample_width * 8}bit, {sample_rate}Hz, {frames} frames"
            )

            if sample_width != 2:
                print("Error: only 16-bit WAV files are supported")
                return False

            if sample_rate != 48000:
                print("Warning: SAI example expects 48 kHz PCM")

            data = wav_file.readframes(frames)

        with open(output_file, "wb") as pcm_file:
            if channels == 1:
                for (sample,) in struct.iter_unpack("<h", data):
                    unsigned = (sample + 0x8000) & 0xFFFF
                    pcm_file.write(struct.pack("<H", unsigned))
            else:
                step = channels
                samples = struct.iter_unpack("<h", data)
                index = 0
                for (sample,) in samples:
                    if index % step == 0:
                        unsigned = (sample + 0x8000) & 0xFFFF
                        pcm_file.write(struct.pack("<H", unsigned))
                    index += 1

        print("Done")
        print(f"Size: {os.path.getsize(output_file)} bytes")
        return True
    except Exception as exc:
        print(f"Error: {exc}")
        return False


def main() -> None:
    if len(sys.argv) != 3:
        print("Usage: convert_wav.py input.wav output.pcm")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    if not os.path.exists(input_file):
        print(f"Error: input file '{input_file}' not found")
        sys.exit(1)

    ok = convert_wav_to_pcm(input_file, output_file)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
