# Diff Gate visual system

## Direction

**Dithered / halftone print system.** Diff Gate should feel like a reviewer’s marked-up engineering proof: ink, paper, strong editorial hierarchy, and a visible trail of decisions. The visual metaphor is a change-control desk, not a dashboard. Cards are clipped notes and evidence strips; cyan is a highlighter, coral signals escalation, and dot fields show areas needing attention.

## Tokens

- Paper `#F6F1E6`; ink `#17212B`; muted ink `#51606A`; rule `#BDC5BF`
- Signal cyan `#007F8B` with white ink for actions; review yellow `#F7C948`; escalation coral `#C94C3B`; approved green `#1E6F50`
- Night treatment: `#111923` paper, `#ECF0E9` ink, cyan `#47C5D0` for labels, and dark cyan `#006F7A` behind white action text.
- Display and body: self-hosted system sans stack (no network font request). A mono system stack carries paths, commands, and evidence. This avoids external font traffic and keeps code dense.
- Spacing follows an 8px scale. Wide screens use asymmetric 7/5 columns; small screens stack packet context before checks.

## Interaction and motion

Packets use a stamped, 180ms opacity/translate reveal when opened. A small non-looping scan line passes through unresolved findings once. With `prefers-reduced-motion`, these appear immediately and the scan is removed. Every state uses words and an icon in addition to colour.

GitHub-imported packets show the reviewed head revision in the packet header. Refreshing that revision returns the packet to the same evidence-first hold state, so a changed pull request cannot inherit a prior approval.

## Asset plan and provenance

The landing hero uses an original abstract “change-control desk” illustration: repository file cards, test receipts, and an approval stamp in a cyan/coral/yellow halftone print world. It contains no readable text, logos, brands, or people. Generated with the factory image model using `/opt/fleet/lib/gen-image.sh` on 2026-08-28; original product artwork. The 1200×630 social crop is derived from this scene. The runtime UI uses authored CSS dot screens and SVG marks.

### Prompt sheet

Subject: an engineering change-control desk with source file sheets, evidence receipt, and review stamp. World: tactile editorial screenprint. Materials: recycled cream paper, newsprint ink, coarse cyan and coral halftone dots. Light: flat studio scan. Lens: isometric editorial still life. Palette: ink navy, cream, signal cyan, safety yellow, coral red. Avoid: readable text, watermarks, logos, brand marks, people, gradients, photorealistic glass, generic dashboards.
