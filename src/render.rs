//! The carousel markup.

use autumn_web::form::FieldOptions;
use maud::{Markup, html};

use crate::config::CarouselConfig;
use crate::slide::Slide;

/// Render a carousel.
#[must_use]
pub fn carousel(
    slides: &[Slide<'_>],
    config: &CarouselConfig<'_>,
    options: &FieldOptions<'_>,
) -> Markup {
    let _ = (slides, config, options);
    html! {}
}
