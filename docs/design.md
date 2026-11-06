# Carousel widget pack: design

Status: accepted. Date: 2026-11-06. Research: [research.md](research.md).

## 1. Goal

Give Autumn apps one carousel widget. The widget is accessible, works with
JavaScript off and works under a strict CSP. It obeys the Autumn widget
contract (`docs/guide/widget-packs.md` in the Autumn repository).

## 2. Brainstorming

We wrote all ideas first. We did not judge them during this step.

1. Slideshow: one slide in view, previous and next buttons.
2. Rail: many cards in view, with a part of the next card in view.
3. Product gallery with thumbnails.
4. Dots, numbers or thumbnails as markers.
5. Automatic rotation with a pause button.
6. Endless loop.
7. Fade, cube and 3D transitions.
8. Swipe and mouse drag.
9. Native scroll snap as the engine.
10. A JS transform engine.
11. CSS-only carousel with `::scroll-button` and `::scroll-marker`.
12. Markers as tabs (APG tabbed carousel).
13. Markers as links to `#slide` fragments.
14. Server paging with htmx for long rails.
15. A live region that tells the slide number.
16. Lazy images and a high fetch priority for the first image.
17. Start at slide N.
18. Captions for image slides.
19. Translatable text.
20. Right-to-left support.
21. Forced colors support.
22. Pause when the page is hidden.
23. An event when the slide changes.
24. Keyboard shortcuts on the whole page.
25. Slides per view that change with the screen width.

## 3. Reverse brainstorming

Question: "How can we make the worst carousel?" Each answer gives a rule.

| Way to fail | Rule that prevents it |
|---|---|
| Rotate automatically, fast, with no stop button. | No rotation by default. The opt-in always shows a rotation button. The interval is 6 s by default. |
| Start rotation again after the user stops it. | A stop is permanent until the user starts rotation. |
| Hide all content until JS runs. | All slides are in the server HTML and show with JS off. |
| Show buttons that do nothing without JS. | The server writes the buttons with `hidden`. JS removes `hidden`. |
| Clone slides for a loop. | No clones. `rewind` jumps to the start. |
| Let Tab go into slides that are out of view. | In a slideshow, JS makes slides out of view `inert`. |
| Make cards in a rail unreachable. | Rail cards are never `inert`. |
| Name each slide "slide". | The slide name is a label or "N of M". |
| Tiny dots. | Each marker target is 24 × 24 px or larger. Buttons are 44 × 44 px. |
| Show the current dot by color only. | The current dot is wider and filled. |
| Move focus when the slide changes. | Buttons do not move focus. Rotation does not move focus. |
| Scroll the page during rotation. | JS scrolls only the track. |
| Announce each automatic change. | The live region speaks only after a user action. |
| Use `style` attributes. | Classes set the layout. |
| Hard-code English. | `CarouselText` holds all text. |
| Break in right-to-left text. | Logical CSS. JS uses edge deltas, not `scrollLeft` signs. |
| Lose the current dot in forced colors. | The current marker uses the `Highlight` system color. A thumbnail also gets an outline. |
| Write arrow-key code. | No key code. The browser scrolls the focused track. |
| Use a CSS-only carousel. | Do not use `::scroll-button` or `::scroll-marker`. |
| Load all images at once. | Images after the start slide load lazily. |
| Shift the layout when images load. | `Slide::image` needs a width and a height. |

## 4. Six thinking hats

**White hat (facts).**
Hero carousels get few clicks. Galleries and rails help users. APG and
WCAG 2.2 give the rules in research §2. Scroll snap works in all main
browsers. `inert` is Baseline. Autumn packs cannot use `style` attributes,
inline scripts or raw colors. The keyboard core owns arrow-key code.

**Red hat (feelings).**
Users dislike motion that they did not start. Developers want one
function call that gives a good result. Reviewers fear a carousel that
fails an audit.

**Black hat (risks).**

- Scroll snap can skip content at high zoom. We use `mandatory` only in
  the slideshow, where each slide fills the track. The rail uses
  `proximity`.
- A no-JS marker link scrolls the page too. This is acceptable: the user
  asked for that slide.
- With JS off, `aria-current` stays on the start slide. The core tabs
  widget has the same limit. We document it.
- A focusable track adds one tab stop. It is necessary for keyboard
  scroll with JS off.
- Controls come before the slides in the DOM but show below them. APG
  and Smashing accept this order. We document it.

**Yellow hat (benefits).**
The browser gives touch, trackpad, momentum and right-to-left scroll. The
pack needs no code for them. The JS is small. All content shows with JS off. htmx swaps work
through `autumn:init`.

**Green hat (new ideas).**

- `scroll-initial-target` shows the start slide in new Chromium with JS
  off.
- One `autumn-carousel:change` event lets apps record slide changes.
- Thumbnails come from the image slides. No second list is necessary.

**Blue hat (process).**
Decisions are in §5. Each acceptance criterion in §9 has a test. Work goes
RED, GREEN, REFACTOR. A multi-angle review follows. §10 maps each
criterion to its evidence.

## 5. Decisions

| # | Decision | Reason |
|---|---|---|
| D1 | Native scroll-snap track. No JS transform engine. | Works with JS off. Free touch and RTL. Research §3.2. [ADR 0001](adr/0001-native-scroll-snap-track.md). |
| D2 | Two layouts: `Slideshow` (default) and `Rail(PerView)`. | Different evidence for each use. Research §1. |
| D3 | Markers are links in a named group. Kinds: `None`, `Dots`, `Numbers`, `Thumbnails`. | Links work with JS off. Tabs need hidden panels. Research §2.5. |
| D4 | Rotation is an opt-in. Default interval 6 s. Interval floor 1 s. | Research §1, §2.1. |
| D5 | No clones, no transitions, no drag code, no key code. | Research §3.1. Widget contract. |
| D6 | JS makes slideshow slides out of view `inert`. Rail slides stay active. | Research §2.3. |
| D7 | A hidden live region speaks "N of M" after a user action only. | APG. Research §2.1. |
| D8 | Verus is not used. | The Rust side only writes markup. It has no state machine. Property tests check the markup rules. The JS rotation states are in §7.3. |
| D9 | Server paging for long rails is out of scope. | The app can put the core `pagination` widget under a rail. An htmx swap starts the new carousel through `autumn:init`. |

## 6. Rust API

```rust
pub fn carousel(slides: &[Slide<'_>], config: &CarouselConfig<'_>, options: &FieldOptions<'_>) -> Markup;

CarouselConfig::new(id, label)       // id: [A-Za-z][A-Za-z0-9_-]*
    .labelled_by(heading_id)         // use a visible heading as the name
    .rail(PerView::new(3))           // default: slideshow
    .markers(Markers::Thumbnails)    // default: Dots (slideshow), None (rail)
    .auto_rotate()                   // 6 s
    .rotation_interval(Duration)     // turns rotation on; floor 1 s
    .rewind()                        // wrap at the ends
    .start(2)                        // clamped to the last slide
    .priority()                      // start image: eager, fetchpriority=high
    .text(CarouselText { .. });      // all visible and spoken text

Slide::new(markup).label("Spring sale")
Slide::image(src, alt, width, height).caption(markup).thumbnail(src)

CarouselPlugin::new()                // installs the pack
pack() -> WidgetPack                 // stories and assets
```

## 7. Behavior

### 7.1 Markup

```text
section#ID.autumn-carousel-root  aria-roledescription=carousel  aria-label
  div.autumn-carousel-bar
    button.autumn-carousel-rotate  hidden          (rotation only)
    button.autumn-carousel-prev    hidden
    div.autumn-carousel-markers    role=group aria-label   (markers only)
      a.autumn-carousel-marker href=#ID--slide-N [aria-current=true]
    button.autumn-carousel-next    hidden
  div#ID--track.autumn-carousel-track  role=group aria-label tabindex=0
    div#ID--slide-N.autumn-carousel-slide  role=group aria-roledescription=slide aria-label
  div.autumn-visually-hidden  aria-live=polite aria-atomic=true
```

One slide: no bar, no live region. No slides: no markup.

### 7.2 JavaScript

- Set-up runs in an `autumn:init` listener, one time per carousel.
- It removes `hidden` from the buttons.
- Scroll events update the current slide, the markers, the disabled
  buttons and `inert`.
- A step starts from the requested slide, not from the scroll position.
  Thus quick clicks do not lose a step.
- When a slide that holds focus becomes `inert`, focus goes to the track.
- A scroll that the pack did not start is a user move.
- Previous and next move one slide (slideshow) or one view (rail). At an
  end, the button has `aria-disabled="true"`, unless `rewind` is set.
- A marker click scrolls the track. It does not change the URL.
- JS scrolls with `scrollTo` to an absolute position. A relative
  `scrollBy` stops at the first snap point. See
  [ADR 0001](adr/0001-native-scroll-snap-track.md).
- After a user move, when the scroll stops, the live region speaks "N of
  M". A rail speaks the cards in view: "4 to 6 of 8".
- Each move sends one `autumn-carousel:change` with `detail.index`, when
  the scroll stops.

### 7.3 Rotation states

| State | Timer | Button text | Enters when |
|---|---|---|---|
| `playing` | runs | Stop | set-up (no reduced motion); user presses Start |
| `paused` | stops | Stop | page is hidden; mouse is on the carousel (not after a user start); focus is in the carousel (after a user start). Only from `playing`. |
| `stopped` | stops | Start | reduced motion at set-up; user presses Stop; focus enters the carousel (not the rotation button); user moves a slide |

Rules:

- `paused` goes back to `playing` when the cause ends.
- Focus on the rotation button does not stop rotation. Else a mouse click
  on Stop gives focus (stop), then a click (start), and rotation goes on.
- A scroll of the track that the pack did not start is a user move. A
  vertical wheel or a Tab key is not.
- After the user presses Start, focus in the carousel pauses rotation
  until focus goes out. Rotation does not move content away from the
  user. Hover does not pause it, as in the APG examples. Only the button
  and a user move stop it.
- Rotation wraps to the first slide. It does not speak.
- A carousel that leaves the DOM stops its timer and shows the Start
  label.

## 8. Tests

| Kind | Where | Runs in |
|---|---|---|
| Public API tests | `tests/markup.rs` | `cargo test` |
| Property tests | `tests/properties.rs` | `cargo test` |
| Static conformance | `tests/conformance.rs` | `cargo test` |
| Browser behavior and conformance | `tests/browser.rs` | `cargo test --features system-tests -- --ignored` |

## 9. Acceptance criteria

### Markup and API

- AC-01: The root is a `<section>` with `aria-roledescription` and a name
  from `aria-label` or `aria-labelledby`.
- AC-02: Each slide is `role="group"` with `aria-roledescription` and a
  name: its label, else "N of M".
- AC-03: IDs are `ID--track` and `ID--slide-N`. No ID repeats, also
  across two carousels.
- AC-04: The buttons have `hidden`, `type="button"`, a text name and
  `aria-controls` to the track.
- AC-05: The rotation button comes first in DOM order. It exists only
  with rotation.
- AC-06: Markers are links to the slide fragments. The start marker has
  `aria-current="true"`. The group has a name.
- AC-07: Thumbnails use the slide image or `Slide::thumbnail`. The link
  name is the slide name.
- AC-08: Image slides have `width`, `height` and `alt`. The start image
  is eager. The others are lazy. `priority()` adds `fetchpriority="high"`.
- AC-09: No slides give no markup. One slide gives no bar and no live
  region.
- AC-10: `start` is clamped. The start slide has the start class.
- AC-11: All text comes from `CarouselText`.
- AC-12: The root gets the app `data-*` and `hx-*` passthrough.
- AC-13: `try_new` returns `InvalidCarouselId` for an invalid ID. `new`
  panics with the same message.
- AC-14: No `style` attribute, no inline script and no `on*` attribute.

### Pack and plugin

- AC-15: `pack()` passes the static widget conformance checks.
- AC-16: Each public widget variant has a story. Slugs start with
  `carousel-`.
- AC-17: `CarouselPlugin` installs the pack. It serves `carousel.css` and
  `carousel.js`. It declares `autumn-web` 0.10.

### Browser

- AC-18: The pack passes the axe, console and keyboard browser checks.
- AC-19: With JS off, all slides show, the buttons stay hidden and a
  marker link shows its slide.
- AC-20: Next and previous move the track, update `aria-current` and
  write "N of M". They do not move focus.
- AC-21: Without `rewind`, the buttons are `aria-disabled` at the ends.
  With `rewind`, they wrap.
- AC-22: A marker click shows its slide and does not change the URL.
- AC-23: A native scroll updates the current marker.
- AC-24: In a slideshow, slides out of view are `inert`. In a rail, no
  slide is `inert`.
- AC-25: Rotation moves the slides and does not move focus or speak.
- AC-26: Focus in the carousel stops rotation. The button then says
  "Start". Start makes it play again.
- AC-27: Hover pauses rotation. Leaving resumes it.
- AC-28: Reduced motion starts rotation stopped.
- AC-29: A carousel swapped in by htmx starts through `autumn:init`.
- AC-30: In right-to-left text, next moves to the next slide.
- AC-31: `autumn-carousel:change` fires one time for each move, with the
  new index.
- AC-32: A removed carousel stops its timer.

### Quality

- AC-33: `cargo fmt`, `cargo clippy` (pedantic, nursery) and `cargo doc`
  pass with no warning.
- AC-34: CI runs format, lint, tests, browser tests and coverage.
- AC-35: The README and docs use short ASD-STE100 sentences.
- AC-36: Line coverage of the Rust code is 90% or more.

### Added after the review

- AC-37: Quick clicks step from the requested slide.
- AC-38: Focus does not go to `<body>` when its slide becomes `inert`.
  After a user start, focus in the carousel pauses rotation.
- AC-39: A rail makes its last cards current and speaks the cards in view.
- AC-40: A user scroll of the track stops rotation. A vertical wheel does
  not.
- AC-41: The rotation interval is clamped to 2^31 − 1 ms.
- AC-42: A number marker keeps its digit in its name (WCAG 2.5.3).
- AC-43: A hidden page pauses rotation.
- AC-44: The start slide shows with JS on, also when the carousel shows
  late. A deep link wins over the start slide.
- AC-45: The carousel sets up without `autumn-widgets.js`.
- AC-46: The track has room for the focus ring of slide content.

## 10. Evidence

See [acceptance.md](acceptance.md).
