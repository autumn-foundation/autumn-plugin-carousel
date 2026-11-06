# ADR 0001: A native scroll-snap track

Status: accepted. Date: 2026-11-06.

## Context

A carousel can move its slides in three ways:

1. A JS transform engine (Swiper, Splide, Embla, Slick).
2. CSS-only carousel features (`::scroll-button`, `::scroll-marker`).
3. A native scroll container with scroll snap, and a small script.

Autumn widgets must work with JavaScript off. They must pass a strict CSP:
no `style` attribute and no inline script. Pack JS has no arrow-key code.

## Decision

Use a native scroll-snap track (option 3).

- The server writes all slides into the track.
- The markers are links to the slides.
- The script shows the buttons, tracks the current slide and runs the
  opt-in rotation.
- The script scrolls with `scrollTo` to an absolute position. A relative
  `scrollBy` stops at the first snap point when `scroll-snap-stop` is
  `always`.

## Reasons

- With JS off, all content shows, and touch, trackpad and keyboard scroll
  work.
- The browser gives momentum, snap and right-to-left support.
- A transform engine writes `style` attributes or hides content until JS
  runs.
- CSS-only carousels work in Chromium only, and experts found faults in
  their accessibility.

## Consequences

- No endless loop by clones. `rewind` jumps to the start.
- No custom transition effects.
- The track is one tab stop, so that keyboard users can scroll it with
  JS off.
- See [research §3](../research.md#3-implementations) for the data.
