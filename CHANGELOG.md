# Changelog

## [Unreleased]

### Added

- The `carousel` widget: a slideshow or a rail on a native scroll-snap
  track. It works with JavaScript off and under a strict CSP.
- `CarouselConfig` (with `new` and `try_new`), `Slide`, `CarouselText`,
  `PerView`, `Markers`, `Layout` and `InvalidCarouselId`.
- Markers: dots, numbers, thumbnails or none.
- Image slides with size, lazy loading, captions and thumbnails.
- Opt-in automatic rotation with a rotation button. Focus stops it. Hover
  and a hidden page pause it. Reduced motion starts it stopped. The
  interval is clamped to 1 s and 2^31 − 1 ms.
- A hidden live region speaks "N of M" after a user move. A rail speaks
  the cards in view.
- The `autumn-carousel:change` event, one time for each move.
- `CarouselPlugin`, `pack()`, `head()` and five stories.
