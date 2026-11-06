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
    let Some(plan) = Plan::new(slides, config) else {
        return html! {};
    };
    let class = root_class(config.layout);
    let start_index = plan.start.to_string();
    let rotate_ms = config
        .rotation
        .filter(|_| plan.rotation)
        .map(|interval| interval.as_millis().to_string());

    let mut attrs: Vec<(&str, &str)> = vec![
        ("id", config.id),
        ("class", &class),
        ("aria-roledescription", &config.text.carousel),
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

    let body = html! {
        @if plan.many {
            (bar(&plan))
        }
        (track(&plan))
        @if plan.many {
            div class="autumn-visually-hidden autumn-carousel-status" aria-live="polite"
                aria-atomic="true" data-position=(plan.config.text.position) {}
        }
    };
    element("section", &attrs, options, body)
}

/// The values that each part of the markup needs.
struct Plan<'p, 'a> {
    slides: &'p [Slide<'a>],
    config: &'p CarouselConfig<'a>,
    /// The start slide index, clamped.
    start: usize,
    /// Two or more slides: the carousel has controls.
    many: bool,
    /// Rotation is on and the carousel has controls.
    rotation: bool,
    /// The markers, or `None` for one slide.
    markers: Markers,
    /// The accessible name of each slide.
    names: Vec<String>,
    track_id: String,
}

impl<'p, 'a> Plan<'p, 'a> {
    /// `None` when there are no slides.
    fn new(slides: &'p [Slide<'a>], config: &'p CarouselConfig<'a>) -> Option<Self> {
        let last = slides.len().checked_sub(1)?;
        let many = last > 0;
        let names = slides
            .iter()
            .enumerate()
            .map(|(i, slide)| {
                slide.label.map_or_else(
                    || config.text.position_of(i + 1, slides.len()),
                    str::to_owned,
                )
            })
            .collect();
        Some(Self {
            slides,
            config,
            start: config.start.min(last),
            many,
            rotation: many && config.rotation.is_some(),
            markers: if many {
                config.marker_kind()
            } else {
                Markers::None
            },
            names,
            track_id: format!("{}-track", config.id),
        })
    }
}

/// The controls: rotation, previous, markers and next, in DOM order.
fn bar(plan: &Plan<'_, '_>) -> Markup {
    let text = &plan.config.text;
    html! {
        div class="autumn-carousel-bar" {
            @if plan.rotation {
                button type="button" class="autumn-carousel-rotate" aria-controls=(plan.track_id)
                    data-start-label=(text.start) data-stop-label=(text.stop) hidden {
                    (icon(Icon::Pause))
                    (icon(Icon::Play))
                    span class="autumn-visually-hidden autumn-carousel-label" { (text.stop) }
                }
            }
            (step_button("autumn-carousel-prev", Icon::Previous, &text.previous, &plan.track_id))
            @if plan.markers != Markers::None {
                div class="autumn-carousel-markers" role="group" aria-label=(text.markers) {
                    @for (i, slide) in plan.slides.iter().enumerate() {
                        (marker(plan, i, slide))
                    }
                }
            }
            (step_button("autumn-carousel-next", Icon::Next, &text.next, &plan.track_id))
        }
    }
}

/// A previous or next button. JS removes `hidden`.
fn step_button(class: &str, icon_kind: Icon, name: &str, track_id: &str) -> Markup {
    html! {
        button type="button" class=(class) aria-controls=(track_id) hidden {
            (icon(icon_kind))
            span class="autumn-visually-hidden" { (name) }
        }
    }
}

/// The track and its slides.
fn track(plan: &Plan<'_, '_>) -> Markup {
    let config = plan.config;
    html! {
        div id=(plan.track_id) class="autumn-carousel-track" role="group"
            aria-label=(config.text.track) tabindex="0" {
            @for (i, slide) in plan.slides.iter().enumerate() {
                @let start = i == plan.start;
                div id=(slide_id(config.id, i))
                    class=(if start { "autumn-carousel-slide autumn-carousel-slide--start" } else { "autumn-carousel-slide" })
                    role="group" aria-roledescription=(config.text.slide)
                    aria-label=(plan.names[i]) {
                    (slide_content(slide, start, config.priority))
                }
            }
        }
    }
}

/// The id of slide `i` (from 0). The number in the id starts at 1.
fn slide_id(id: &str, i: usize) -> String {
    format!("{id}-slide-{}", i + 1)
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

/// One marker link to slide `i` (from 0).
fn marker(plan: &Plan<'_, '_>, i: usize, slide: &Slide<'_>) -> Markup {
    let kind = plan.markers;
    let href = format!("#{}", slide_id(plan.config.id, i));
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
        a class=(class) href=(href) aria-current=[(i == plan.start).then_some("true")] {
            @if kind == Markers::Numbers {
                span class="autumn-carousel-marker-number" aria-hidden="true" { (i + 1) }
            }
            @if kind == Markers::Thumbnails {
                @if let Some((src, (width, height))) = thumbnail {
                    img class="autumn-carousel-thumbnail" src=(src) alt="" width=[width]
                        height=[height] loading="lazy" decoding="async";
                }
            }
            span class="autumn-visually-hidden" { (plan.names[i]) }
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
