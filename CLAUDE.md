# CLAUDE.md: autumn-plugin-carousel

A carousel widget pack for Autumn (`autumn-web` 0.10).

## Layout

| Path | Content |
|---|---|
| `src/config.rs` | `CarouselConfig`, `CarouselText`, `PerView`, `Markers`, `Layout`. |
| `src/slide.rs` | `Slide`. |
| `src/render.rs` | `carousel()`: the server markup. |
| `src/stories.rs` | One story for each variant. Slugs start with `carousel-`. |
| `src/carousel.css`, `src/carousel.js` | The pack assets. |
| `tests/markup.rs` | Markup tests. Each test name starts with its AC number. |
| `tests/properties.rs` | Property tests of the markup rules. |
| `tests/conformance.rs` | Static widget conformance and plugin wiring. |
| `tests/browser.rs` | Chromium tests. `#[ignore]`d, behind `system-tests`. |
| `docs/` | Research, design (with the ACs), ADRs, AC evidence. |

## Commands

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features system-tests --test browser -- --ignored --test-threads=2
cargo llvm-cov --fail-under-lines 90 --ignore-filename-regex 'tests/'
```

## Rules

- Obey the Autumn widget contract (`docs/guide/widget-packs.md` in Autumn):
  - Classes start with `autumn-carousel-`, or are core classes.
  - CSS reads only public tokens and sets only `--autumn-carousel-*`. No raw colors.
  - No `style` attribute, no inline script, no `on*` attribute.
  - No arrow-key code in JS. Set-up runs on `autumn:init`.
- The markup works with JavaScript off. Buttons start `hidden`.
- No automatic rotation by default.
- Work RED, GREEN, REFACTOR. A new behavior gets a test first.
- A new acceptance criterion goes in `docs/design.md` §9 and in
  `docs/acceptance.md`.
- Docs and comments use short ASD-STE100 sentences.
