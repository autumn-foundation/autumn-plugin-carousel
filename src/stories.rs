//! The pack stories: one for each carousel variant.

use autumn_web::form::FieldOptions;
use autumn_web::stories::{Story, story};
use maud::{Markup, html};

use crate::{CarouselConfig, Markers, PerView, Slide, carousel};

/// A small SVG image as a `data:` URL. The default CSP allows
/// `img-src 'self' data:`.
const MOUNTAIN: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 640 360'%3E%3Crect width='640' height='360' fill='%23dbeafe'/%3E%3Cpath d='M0 360 220 120l140 160 90-90 190 170z' fill='%231e3a8a'/%3E%3C/svg%3E";
const LAKE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 640 360'%3E%3Crect width='640' height='360' fill='%23ecfccb'/%3E%3Cellipse cx='320' cy='260' rx='260' ry='70' fill='%230e7490'/%3E%3C/svg%3E";
const FOREST: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 640 360'%3E%3Crect width='640' height='360' fill='%23fef3c7'/%3E%3Cpath d='M120 320 200 100l80 220zm200 0 80-240 80 240z' fill='%23166534'/%3E%3C/svg%3E";

fn news_slides() -> Vec<Slide<'static>> {
    [
        ("Faster pages", "Pages now load in half the time."),
        ("Dark mode", "Each widget follows the system color scheme."),
        ("Strict CSP", "No inline script or style is necessary."),
        ("Right to left", "Each widget works in Arabic and Hebrew."),
    ]
    .into_iter()
    .map(|(title, body)| {
        Slide::new(html! {
            h3 { (title) }
            p { (body) }
        })
    })
    .collect()
}

fn slideshow() -> Markup {
    carousel(
        &news_slides(),
        &CarouselConfig::new("story-news", "Release news"),
        &FieldOptions::new(),
    )
}

fn auto_rotation() -> Markup {
    carousel(
        &news_slides(),
        &CarouselConfig::new("story-rotate", "Release news").auto_rotate(),
        &FieldOptions::new(),
    )
}

fn rail() -> Markup {
    let slides: Vec<Slide<'static>> = (1..=8)
        .map(|n| {
            Slide::new(html! {
                h3 { "Product " (n) }
                p { "A short description of product " (n) "." }
                a href={ "#product-" (n) } { "View product " (n) }
            })
        })
        .collect();
    carousel(
        &slides,
        &CarouselConfig::new("story-rail", "Related products").rail(PerView::new(3)),
        &FieldOptions::new(),
    )
}

fn gallery() -> Markup {
    let slides = [
        Slide::image(MOUNTAIN, "A blue mountain under a pale sky", 640, 360)
            .caption(html! { "Mountain" }),
        Slide::image(LAKE, "A teal lake on a green field", 640, 360).caption(html! { "Lake" }),
        Slide::image(FOREST, "Two green trees on a yellow field", 640, 360)
            .caption(html! { "Forest" }),
    ];
    carousel(
        &slides,
        &CarouselConfig::new("story-gallery", "Trip photos")
            .markers(Markers::Thumbnails)
            .priority(),
        &FieldOptions::new(),
    )
}

fn numbers() -> Markup {
    carousel(
        &news_slides(),
        &CarouselConfig::new("story-numbers", "Release news")
            .markers(Markers::Numbers)
            .rewind(),
        &FieldOptions::new(),
    )
}

/// All stories, in gallery order.
pub fn all() -> [Story; 5] {
    [
        story! { "Carousel", "Carousel slideshow", { slideshow() } },
        story! { "Carousel", "Carousel auto rotation", { auto_rotation() } },
        story! { "Carousel", "Carousel rail", { rail() } },
        story! { "Carousel", "Carousel gallery", { gallery() } },
        story! { "Carousel", "Carousel numbers", { numbers() } },
    ]
}
