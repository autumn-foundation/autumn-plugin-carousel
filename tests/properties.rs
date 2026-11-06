//! Property tests: the markup rules hold for each slide count, start
//! index, layout and marker kind.

mod common;

use std::collections::BTreeSet;

use autumn_plugin_carousel::{CarouselConfig, Markers, PerView};
use common::{all, attr, render, text_slides};
use proptest::prelude::*;

fn markers() -> impl Strategy<Value = Markers> {
    prop_oneof![
        Just(Markers::None),
        Just(Markers::Dots),
        Just(Markers::Numbers),
        Just(Markers::Thumbnails),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn ac14_markup_rules_hold(
        count in 0usize..12,
        start in 0usize..20,
        rail in proptest::option::of(0u8..9),
        kind in markers(),
        rotate in any::<bool>(),
        rewind in any::<bool>(),
    ) {
        let mut config = CarouselConfig::new("p", "Property").start(start).markers(kind);
        if let Some(per_view) = rail {
            config = config.rail(PerView::new(per_view));
        }
        if rotate {
            config = config.auto_rotate();
        }
        if rewind {
            config = config.rewind();
        }
        let html = render(&text_slides(count), &config);

        // No slides: no markup.
        if count == 0 {
            prop_assert!(all(&html, "*").iter().all(|e| e.value().name() == "html"));
            return Ok(());
        }

        // One root. One slide element for each slide, in order.
        prop_assert_eq!(all(&html, "section.autumn-carousel-root").len(), 1);
        let slides = all(&html, ".autumn-carousel-slide");
        prop_assert_eq!(slides.len(), count);
        for (i, slide) in slides.iter().enumerate() {
            let id = format!("p--slide-{}", i + 1);
            let name = format!("{} of {count}", i + 1);
            prop_assert_eq!(attr(*slide, "id"), Some(id.as_str()));
            prop_assert_eq!(attr(*slide, "aria-label"), Some(name.as_str()));
        }

        // Exactly one start slide, at the clamped index.
        let starts = all(&html, ".autumn-carousel-slide--start");
        prop_assert_eq!(starts.len(), 1);
        let want = format!("p--slide-{}", start.min(count - 1) + 1);
        prop_assert_eq!(attr(starts[0], "id"), Some(want.as_str()));

        // No id repeats.
        let ids: Vec<_> = all(&html, "[id]").into_iter().filter_map(|e| attr(e, "id")).collect();
        prop_assert_eq!(ids.len(), ids.iter().collect::<BTreeSet<_>>().len());

        // Controls only with two or more slides.
        let buttons = all(&html, "button");
        if count == 1 {
            prop_assert!(buttons.is_empty());
            prop_assert!(all(&html, ".autumn-carousel-marker").is_empty());
        } else {
            let want_buttons = if rotate { 3 } else { 2 };
            prop_assert_eq!(buttons.len(), want_buttons);
            prop_assert!(buttons.iter().all(|b| attr(*b, "hidden").is_some()));
            let markers = all(&html, "a.autumn-carousel-marker");
            if kind == Markers::None {
                prop_assert!(markers.is_empty());
            } else {
                prop_assert_eq!(markers.len(), count);
                let current = all(&html, "[aria-current='true']");
                prop_assert_eq!(current.len(), 1);
                let href = format!("#p--slide-{}", start.min(count - 1) + 1);
                prop_assert_eq!(attr(current[0], "href"), Some(href.as_str()));
            }
        }

        // No style attribute anywhere.
        prop_assert!(all(&html, "[style]").is_empty());
    }
}
