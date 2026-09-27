# NPU test assets

## `mexican_marigold_96x96.rgb`

Raw `RGB8`, 96x96, 27 648 bytes — the fixed input tensor used by the
`npu_mobilenet` example (and by `embedded-nn-tflite`'s `stai_golden` example
when producing the reference logits). The model under test is
MobileNetV1-0.25 trained on five flower classes
(`daisy, dandelion, rose, sunflower, tulip`), so a flower makes a meaningful
in-distribution test input.

| | |
| :--- | :--- |
| Source | Wikimedia Commons, `File:Mexican marigold.JPG` |
| URL | `https://commons.wikimedia.org/wiki/Special:FilePath/Mexican_marigold.JPG?width=320` |
| License | Public domain (`LicenseShortName: Public domain`, `License: pd`) |
| Retrieved | 2026-09-27 |
| sha256 | `b9ac6a9ef04df8f01d05a5b7d17d0a8b3629a05b780bcde763787e4ffbd946e5` |

Conversion (330x247 JPEG -> raw RGB8, centre-cropped to square, then resized):

```text
# decode + centre-crop to 330x330 + resize to 96x96 (triangle filter) -> raw RGB8
```

The Commons `ImageDescription` metadata field for this file is empty; the
subject is taken from the file title.

### Why the raw form, and not a PNG

The ATON NPU block has no image decoder: the compiled network's input tensor
must already be `uint8` NHWC in memory (`1x96x96x3`, scale 1/127.5,
zero-point 127), which is exactly what `mexican_marigold_96x96.rgb` holds. It
is embedded into the example with `include_bytes!`, so the bytes fed to the NPU
are byte-identical to the ones the reference was computed from.

Note that `embedded-nn-tflite` rewrites `UINT8` tensors to `INT8` by
subtracting 128 from values *and* zero-points, so the host interpreter is fed
`pixel - 128`. That is arithmetically the same computation
(`(v - 128) - (zp - 128) == v - zp`), which is why the stored file stays raw
uint8.
