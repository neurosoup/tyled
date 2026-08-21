---
name: feedback-user-authors-art
description: User authors pixel-art assets themselves; check assets/ and confirm before generating art
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6eb22acf-6b15-4d9e-891b-d5b1b94c10fa
---

When a task needs an art asset (sprite, glyph/font atlas, tileset), do **not** generate it unprompted. Check `assets/` first and confirm with the user — they author pixel art themselves (they have the pixel-art skills and a strong pixel aesthetic).

**Why:** During the Starting-phase overlay work, I started generating `assets/font.png` via ImageMagick; the user stopped it with "I already made a font.png font atlas in assets." My generation was redundant.

**How to apply:** Write the code against a documented atlas layout (cell size, grid, glyph order), then **inspect the user's actual asset** and sync the code's mapping to it (their `font.png` had digits on row 2 and no `¢`, differing from my assumed order — I had to fix `glyph_index`). Ask before authoring art; offer to generate only as a fallback if they haven't made it. Relates to [[feedback-stage-code-not-delete]] (respect existing work over regenerating).
