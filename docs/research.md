# Carousel research

Date: 2026-11-06. This file gives the facts that the design uses. Each
fact has a source.

## 1. Use carousels with care

- A home page hero carousel gets about 1% of clicks. Users click slide 1
  for 84–89% of those clicks.
  [Runyon](https://erikrunyon.com/2013/01/carousel-interaction-stats/)
- Users see automatic rotation as an advertisement. They miss content.
  [NN/g](https://www.nngroup.com/articles/auto-forwarding/)
- Carousels help in three cases:
  - Product image galleries. Thumbnails work better than dots.
    [Baymard](https://baymard.com/blog/always-use-thumbnails-additional-images)
  - Rails of related cards, with a part of the next card in view.
    [NN/g](https://www.nngroup.com/articles/mobile-carousels/)
  - Known content, for example testimonials.
    [shouldiuseacarousel.com](https://shouldiuseacarousel.com/)
- GOV.UK and USWDS have no carousel component.
  [GOV.UK](https://design-system.service.gov.uk/components/),
  [USWDS](https://designsystem.digital.gov/components/overview/)

Result: the safe default is a static carousel. Automatic rotation is an
opt-in.

## 2. Accessibility rules

### 2.1 WAI-ARIA APG carousel pattern

Source: [APG carousel](https://www.w3.org/WAI/ARIA/apg/patterns/carousel/).

- The root is a named `<section>` with `aria-roledescription="carousel"`.
  The name does not contain the word "carousel".
- Each slide is `role="group"` with `aria-roledescription="slide"`. Its
  name is a unique label or "N of M". The name does not contain "slide".
- Previous, next and rotation controls are `<button>` elements. They do
  not move focus.
- The rotation control is the first control in DOM order. Its label is
  the action: "Stop automatic slide show" or "Start automatic slide show".
  It has no `aria-pressed`.
- Focus in the carousel stops rotation. Rotation does not start again
  until the user starts it. Mouse hover pauses rotation.
- `prefers-reduced-motion: reduce` starts rotation stopped.
  [APG example JS](https://www.w3.org/WAI/content-assets/wai-aria-practices/patterns/carousel/examples/js/carousel-prev-next.js)
- Do not announce slide changes during rotation.

### 2.2 WCAG 2.2

| Criterion | Rule for a carousel |
|---|---|
| [2.2.2 Pause, Stop, Hide](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html) | Movement longer than 5 s needs a pause control. A pause only while focus or hover stays is not sufficient. |
| [2.1.1 Keyboard](https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html) | All functions work with a keyboard. |
| [2.4.3 Focus Order](https://www.w3.org/WAI/WCAG22/Understanding/focus-order.html) | Hidden slides are not in the tab order. |
| [2.4.11 Focus Not Obscured](https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html) | Focus does not go into a slide that is out of view. |
| [2.5.1 Pointer Gestures](https://www.w3.org/WAI/WCAG22/Understanding/pointer-gestures.html) | A swipe needs a button alternative. |
| [2.5.8 Target Size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | Targets are 24 × 24 CSS px or larger. |
| [1.4.1 Use of Color](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html) | The current marker has a different shape, not only a different color. |
| [2.3.3 Animation from Interactions](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html) | Smooth scroll stops when the user asks for reduced motion. |
| [4.1.2 Name, Role, Value](https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html) | Icon buttons have names. The current marker has a state. |

### 2.3 Hidden slides

- `inert` removes a slide from focus, find-in-page and the accessibility
  tree. It is Baseline since April 2023.
  [MDN](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/inert)
- `aria-hidden="true"` on a slide with a link fails axe
  `aria-hidden-focus`.
  [axe rules](https://github.com/dequelabs/axe-core/blob/develop/doc/rule-descriptions.md)
- Do not make cards in a rail inert. Then Tab and find-in-page reach each
  card. [Chrome](https://developer.chrome.com/blog/accessible-carousel)

### 2.4 Scroll containers

- A scroll container needs keyboard access. Else axe
  `scrollable-region-focusable` fails.
  [Chrome 130](https://developer.chrome.com/blog/keyboard-focusable-scrollers)
- Snap can skip content when a user zooms. Use `mandatory` snap only when
  each slide fits the container.
  [CSSWG #3602](https://github.com/w3c/csswg-drafts/issues/3602)

### 2.5 CSS-only carousels

- `::scroll-button()` and `::scroll-marker` are in Chromium only. Their
  roles are not in a specification. Accessibility experts reported
  problems with them.
  [Soueidan](https://sarasoueidan.com/blog/css-carousels-accessibility),
  [Roselli](https://adrianroselli.com/2025/05/my-request-to-google-on-accessibility.html)
- Do not use them for semantics.

## 3. Implementations

### 3.1 Libraries

| Library | Moves with | No-JS state | Note |
|---|---|---|---|
| Swiper | transform (`cssMode` uses scroll) | content shows | Autoplay at 3 s. No hover pause by default. |
| Splide | transform | content hidden | Good ARIA. Loop mode clones slides. |
| Embla | transform | content shows | Accessibility plugin in v9 only. |
| Glide | transform | content shows | Loop mode clones slides. |
| Flickity | transform | content shows | GPL-3.0. Removes the focus outline. |
| Slick | transform | content hidden | Writes `style` attributes. Removes the focus outline. |
| Bootstrap 5 | transform | content hidden | Uses `display: none` slides. |
| scroll-snap-slider | native scroll | content shows | Small. |

Source: npm and jsDelivr files, read on 2026-11-02.

Lessons:

- Clones for an endless loop make duplicate ids and focus targets.
- A `style` attribute fails a strict CSP.
- Content that is hidden until JS runs fails with JS off.

### 3.2 Native scroll snap

- `scroll-snap-type`, `scroll-snap-align` and `overscroll-behavior` work
  in all main browsers.
  [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/scroll-snap-type)
- The `scrollend` event works in all main browsers since Safari 26.2.
  [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollend_event)
- A `#slide-3` link works without JS. It also scrolls the page.
  [HTML](https://html.spec.whatwg.org/multipage/browsing-the-web.html)
- In right-to-left text, `scrollLeft` is negative. Use logical CSS
  properties. Do not use offset math that assumes left-to-right.
  [CSSOM View](https://drafts.csswg.org/cssom-view/#dom-element-scrollleft)
- In forced colors, the browser replaces background colors. Show the
  current marker with a border.
  [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/forced-colors)

### 3.3 Performance

- Load the first image at once. Load other images lazily.
  [web.dev](https://web.dev/articles/carousel-best-practices)
- Give each image `width` and `height` to stop layout shift.
  [web.dev](https://web.dev/articles/optimize-cls)
- Put all slides in the server HTML. Slides that JS adds make LCP worse.
  [web.dev](https://web.dev/articles/carousel-best-practices)

## 4. Decisions from the research

1. A native scroll-snap track holds all slides. It works with JS off.
2. JS shows the buttons, tracks the current slide and stops a page jump.
3. Markers are links, not tabs. Links work with JS off.
4. No autoplay by default. The opt-in always has a rotation button.
5. No loop by clones. No transition effects. No drag code.
6. No `style` attributes. Classes set the layout.
