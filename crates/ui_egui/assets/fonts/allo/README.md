# Allo 1.0.1

The original, unmodified `AlloVariable.ttf` font is bundled under the SIL Open
Font License 1.1. The accompanying [OFL.txt](OFL.txt)
retains the complete copyright notices and license.

Source: [Josh Puckett's Allo](https://github.com/joshpuckett/allo), release 1.0.1.
Copyright 2026 Josh Puckett; copyright 2016 The Inter Project Authors.

The proportional interface font and missing-symbol fallback use one shared
`FontData` with `FontTweak::coords`: weight (`wght`) 400, Text optical size (`opsz`)
14, and Friendly roundness (`ROND`) 50. Monospace retains egui's existing primary
font. Branding selects weight 600 and optical size 17 through `RichText::variation`
while keeping explicit white text; it does not embed another face or copy the font
bytes. Other UI text can select any supported weight, optical size, or roundness
through per-run variations.

The original variable font is 1,261,452 bytes, compared with 199,716 bytes for the
previous Regular and SemiBold static faces. This trades about 1 MiB of embedded
font data for all three axes and continuous weights. No static faces are retained.
The available axes are weight 100–900, optical size 9–32, and roundness 0–100.

CT uses egui 0.36.2, which includes variable-font support and the glyph-cache fixes
introduced in 0.35. The implementation follows the
[versioned `FontTweak` API](https://docs.rs/epaint/0.36.2/epaint/text/struct.FontTweak.html)
and [per-run `RichText` variations](https://docs.rs/egui/0.36.2/egui/struct.RichText.html#method.variation).

Keep `eframe`, `egui_extras`, and `emath` on one compatible egui release, alongside
the matching `egui-toast` release. `emath::Rect` crosses the GUI/config boundary.
teamy-tracing's optional egui viewer shares that egui version and retains live
typed reports through its capture buffer. Preserve the current glow renderer
explicitly when newer eframe defaults select a different backend.
