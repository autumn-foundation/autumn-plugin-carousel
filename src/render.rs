//! The carousel markup.

use autumn_web::form::FieldOptions;
use autumn_web::widget_pack::element;
use maud::{Markup, html};

use crate::config::{CarouselConfig, Layout, Markers};
use crate::slide::{Content, Slide};

/// Render a carousel.
///
/// The markup works with JavaScript off: all slides are in a track that
/// scrolls, and the markers are links to the slides. With `carousel.js`,
/// the previous, next and rotation buttons show, and slides out of view
/// in a slideshow become `inert`.
///
/// The root `<section>` gets the app `data-*` and `hx-*` passthrough from
/// `options`. No slides give no markup. One slide gives no controls.
///
/// ```rust
/// use autumn_plugin_carousel::{CarouselConfig, Slide, carousel};
/// use autumn_web::form::FieldOptions;
/// use maud::html;
///
/// let slides = [
///     Slide::new(html! { h3 { "Fast" } p { "Pages load in 50 ms." } }),
///     Slide::new(html! { h3 { "Safe" } p { "A strict CSP is the default." } }),
/// ];
/// let markup = carousel(&slides, &CarouselConfig::new("tour", "Product tour"), &FieldOptions::new());
/// let html = markup.into_string();
/// assert!(html.contains(r#"aria-roledescription="carousel""#));
/// assert!(html.contains(r#"aria-label="2 of 2""#));
/// ```
#[must_use]
pub fn carousel(
    slides: &[Slide<'_>],
    config: &CarouselConfig<'_>,
    options: &FieldOptions<'_>,
) -> Markup {
    let total = slides.len();
    if total == 0 {
        return html! {};
    }
    let id = config.id;
    let start = config.start.min(total - 1);
    let many = total > 1;
    let rotation = config.rotation.filter(|_| many);
    let rotate_ms = rotation.map(|interval| interval.as_millis().to_string());
    let start_index = start.to_string();
    let class = root_class(config.layout);
    let text = &config.text;

    let mut attrs: Vec<(&str, &str)> = vec![
        ("id", id),
        ("class", &class),
        ("aria-roledescription", &text.carousel),
    ];
    match config.labelled_by {
        Some(heading) => attrs.push(("aria-labelledby", heading)),
        None => attrs.push(("aria-label", config.label)),
    }
    attrs.push(("data-autumn-carousel", ""));
    attrs.push(("data-autumn-carousel-start", &start_index));
    if let Some(ms) = &rotate_ms {
        attrs.push(("data-autumn-carousel-rotate", ms));
    }
    if config.rewind {
        attrs.push(("data-autumn-carousel-rewind", ""));
    }

    let names: Vec<String> = slides
        .iter()
        .enumerate()
        .map(|(i, slide)| {
            slide
                .label
                .map_or_else(|| text.position_of(i + 1, total), str::to_owned)
        })
        .collect();
    let track_id = format!("{id}-track");
    let markers = if many {
        config.marker_kind()
    } else {
        Markers::None
    };

    let body = html! {
        @if many {
            div class="autumn-carousel-bar" {
                @if rotation.is_some() {
                    button type="button" class="autumn-carousel-rotate" aria-controls=(track_id)
                        data-start-label=(text.start) data-stop-label=(text.stop) hidden {
                        (icon(Icon::Pause))
                        (icon(Icon::Play))
                        span class="autumn-visually-hidden autumn-carousel-label" { (text.stop) }
                    }
                }
                button type="button" class="autumn-carousel-prev" aria-controls=(track_id) hidden {
                    (icon(Icon::Previous))
                    span class="autumn-visually-hidden" { (text.previous) }
                }
                @if markers != Markers::None {
                    div class="autumn-carousel-markers" role="group" aria-label=(text.markers) {
                        @for (i, slide) in slides.iter().enumerate() {
                            (marker(id, markers, i, i == start, &names[i], slide))
                        }
                    }
                }
                button type="button" class="autumn-carousel-next" aria-controls=(track_id) hidden {
                    (icon(Icon::Next))
                    span class="autumn-visually-hidden" { (text.next) }
                }
            }
        }
        div id=(track_id) class="autumn-carousel-track" role="group" aria-label=(text.track)
            tabindex="0" {
            @for (i, slide) in slides.iter().enumerate() {
                div id=(format!("{id}-slide-{}", i + 1))
                    class=(if i == start { "autumn-carousel-slide autumn-carousel-slide--start" } else { "autumn-carousel-slide" })
                    role="group" aria-roledescription=(text.slide) aria-label=(names[i]) {
                    (slide_content(slide, i == start, config.priority))
                }
            }
        }
        @if many {
            div class="autumn-visually-hidden autumn-carousel-status" aria-live="polite"
                aria-atomic="true" data-position=(text.position) {}
        }
    };
    element("section", &attrs, options, body)
}

/// The root classes for `layout`.
fn root_class(layout: Layout) -> String {
    match layout {
        Layout::Slideshow => "autumn-carousel-root autumn-carousel-root--slideshow".to_owned(),
        Layout::Rail(per_view) => format!(
            "autumn-carousel-root autumn-carousel-root--rail autumn-carousel-root--per-view-{}",
            per_view.get()
        ),
    }
}

/// One marker link. `n` is the slide index from 0.
fn marker(
    id: &str,
    kind: Markers,
    n: usize,
    current: bool,
    name: &str,
    slide: &Slide<'_>,
) -> Markup {
    let href = format!("#{id}-slide-{}", n + 1);
    let class = match kind {
        Markers::Numbers => "autumn-carousel-marker autumn-carousel-marker--number",
        Markers::Thumbnails => "autumn-carousel-marker autumn-carousel-marker--thumbnail",
        Markers::Dots | Markers::None => "autumn-carousel-marker autumn-carousel-marker--dot",
    };
    let thumbnail = match (&slide.content, slide.thumbnail) {
        (_, Some(src)) => Some((src, slide.size())),
        (Content::Image { src, .. }, None) => Some((*src, slide.size())),
        (Content::Markup(_), None) => None,
    };
    html! {
        a class=(class) href=(href) aria-current=[current.then_some("true")] {
            @if kind == Markers::Numbers {
                span class="autumn-carousel-marker-number" aria-hidden="true" { (n + 1) }
            }
            @if kind == Markers::Thumbnails {
                @if let Some((src, (width, height))) = thumbnail {
                    img class="autumn-carousel-thumbnail" src=(src) alt="" width=[width]
                        height=[height] loading="lazy" decoding="async";
                }
            }
            span class="autumn-visually-hidden" { (name) }
        }
    }
}

/// The content of one slide.
fn slide_content(slide: &Slide<'_>, start: bool, priority: bool) -> Markup {
    match &slide.content {
        Content::Markup(markup) => markup.clone(),
        Content::Image {
            src,
            alt,
            width,
            height,
            caption,
        } => {
            let image = html! {
                img class="autumn-carousel-image" src=(src) alt=(alt) width=(width)
                    height=(height) loading=(if start { "eager" } else { "lazy" })
                    fetchpriority=[(start && priority).then_some("high")] decoding="async";
            };
            match caption {
                Some(caption) => html! {
                    figure class="autumn-carousel-figure" {
                        (image)
                        figcaption class="autumn-carousel-caption" { (caption) }
                    }
                },
                None => image,
            }
        }
    }
}

/// The icons. They are decoration: each button also has text.
#[derive(Clone, Copy)]
enum Icon {
    Previous,
    Next,
    Pause,
    Play,
}

fn icon(icon: Icon) -> Markup {
    let (class, path) = match icon {
        Icon::Previous => (
            "autumn-carousel-icon",
            "M15.4 5.4 14 4l-8 8 8 8 1.4-1.4L8.8 12z",
        ),
        Icon::Next => (
            "autumn-carousel-icon",
            "M8.6 5.4 10 4l8 8-8 8-1.4-1.4 6.6-6.6z",
        ),
        Icon::Pause => (
            "autumn-carousel-icon autumn-carousel-icon-pause",
            "M7 5h3v14H7zm7 0h3v14h-3z",
        ),
        Icon::Play => (
            "autumn-carousel-icon autumn-carousel-icon-play",
            "M8 5v14l11-7z",
        ),
    };
    html! {
        svg class=(class) viewBox="0 0 24 24" aria-hidden="true" focusable="false" {
            path d=(path) {}
        }
    }
}
