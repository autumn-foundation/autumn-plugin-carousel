//! Test helpers: parse the carousel markup.

#![allow(dead_code)]

use autumn_web::form::FieldOptions;
use autumn_plugin_carousel::{CarouselConfig, Slide, carousel};
use scraper::{ElementRef, Html, Selector};

/// Render and parse a carousel.
pub fn render(slides: &[Slide<'_>], config: &CarouselConfig<'_>) -> Html {
    Html::parse_fragment(&carousel(slides, config, &FieldOptions::new()).into_string())
}

/// `count` text slides.
pub fn text_slides(count: usize) -> Vec<Slide<'static>> {
    (0..count)
        .map(|i| Slide::new(maud::html! { p { "Slide body " (i) } }))
        .collect()
}

/// All elements that match `css`.
pub fn all<'a>(html: &'a Html, css: &str) -> Vec<ElementRef<'a>> {
    let selector = Selector::parse(css).unwrap_or_else(|e| panic!("bad selector {css}: {e}"));
    html.select(&selector).collect()
}

/// The one element that matches `css`.
pub fn one<'a>(html: &'a Html, css: &str) -> ElementRef<'a> {
    let found = all(html, css);
    assert_eq!(found.len(), 1, "expected one `{css}`, got {}: {}", found.len(), html.html());
    found[0]
}

/// An attribute value of `el`.
pub fn attr<'a>(el: ElementRef<'a>, name: &str) -> Option<&'a str> {
    el.value().attr(name)
}

/// The text of `el`, trimmed.
pub fn text(el: ElementRef<'_>) -> String {
    el.text().collect::<String>().trim().to_owned()
}
