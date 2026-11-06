//! Markup tests for the public API. Each test names its acceptance
//! criterion (see `docs/design.md`).

mod common;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::time::Duration;

use autumn_plugin_carousel::{
    CarouselConfig, CarouselText, DEFAULT_ROTATION_INTERVAL, Layout, Markers, PerView, Slide,
    carousel,
};
use autumn_web::form::FieldOptions;
use common::{all, attr, one, render, text, text_slides};

#[test]
fn ac01_root_is_a_named_carousel_section() {
    let html = render(&text_slides(3), &CarouselConfig::new("hero", "Featured news"));
    let root = one(&html, "section#hero");
    assert_eq!(attr(root, "aria-roledescription"), Some("carousel"));
    assert_eq!(attr(root, "aria-label"), Some("Featured news"));
    assert_eq!(attr(root, "aria-labelledby"), None);
    assert!(attr(root, "class").is_some_and(|c| c.contains("autumn-carousel-root")));
    assert!(attr(root, "data-autumn-carousel").is_some());
}

#[test]
fn ac01_labelled_by_replaces_the_label() {
    let config = CarouselConfig::new("hero", "unused").labelled_by("news-heading");
    let html = render(&text_slides(2), &config);
    let root = one(&html, "section#hero");
    assert_eq!(attr(root, "aria-labelledby"), Some("news-heading"));
    assert_eq!(attr(root, "aria-label"), None);
}

#[test]
fn ac02_slides_are_named_groups() {
    let mut slides = text_slides(3);
    slides[1] = Slide::new(maud::html! { p { "Sale" } }).label("Spring sale");
    let html = render(&slides, &CarouselConfig::new("hero", "Featured"));
    let found = all(&html, ".autumn-carousel-slide");
    assert_eq!(found.len(), 3);
    let names: Vec<_> = found.iter().map(|s| attr(*s, "aria-label")).collect();
    assert_eq!(names, [Some("1 of 3"), Some("Spring sale"), Some("3 of 3")]);
    for slide in found {
        assert_eq!(attr(slide, "role"), Some("group"));
        assert_eq!(attr(slide, "aria-roledescription"), Some("slide"));
    }
}

#[test]
fn ac03_ids_derive_from_the_root_id_and_do_not_repeat() {
    let config = CarouselConfig::new("deals", "Deals")
        .markers(Markers::Numbers)
        .auto_rotate();
    let html = render(&text_slides(4), &config);
    let track = one(&html, ".autumn-carousel-track");
    assert_eq!(attr(track, "id"), Some("deals-track"));
    let ids: Vec<_> = all(&html, "[id]")
        .into_iter()
        .filter_map(|e| attr(e, "id"))
        .collect();
    let unique: BTreeSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "repeated id in {ids:?}");
    for n in 1..=4 {
        one(&html, &format!("#deals-slide-{n}"));
    }
}

#[test]
fn ac03_track_is_a_named_focusable_group() {
    let html = render(&text_slides(2), &CarouselConfig::new("hero", "Featured"));
    let track = one(&html, ".autumn-carousel-track");
    assert_eq!(attr(track, "role"), Some("group"));
    assert_eq!(attr(track, "aria-label"), Some("Slides"));
    assert_eq!(attr(track, "tabindex"), Some("0"));
}

#[test]
fn ac04_buttons_are_hidden_named_and_control_the_track() {
    let html = render(&text_slides(3), &CarouselConfig::new("hero", "Featured"));
    for (class, name) in [
        ("autumn-carousel-prev", "Previous slide"),
        ("autumn-carousel-next", "Next slide"),
    ] {
        let button = one(&html, &format!("button.{class}"));
        assert_eq!(attr(button, "type"), Some("button"));
        assert!(attr(button, "hidden").is_some(), "{class} must start hidden");
        assert_eq!(attr(button, "aria-controls"), Some("hero-track"));
        assert_eq!(text(button), name);
    }
}

#[test]
fn ac05_rotation_button_comes_first_and_only_with_rotation() {
    let html = render(&text_slides(3), &CarouselConfig::new("hero", "Featured"));
    assert!(all(&html, ".autumn-carousel-rotate").is_empty());
    assert_eq!(
        attr(one(&html, "section#hero"), "data-autumn-carousel-rotate"),
        None
    );

    let config = CarouselConfig::new("hero", "Featured").auto_rotate();
    let html = render(&text_slides(3), &config);
    let buttons = all(&html, "button");
    assert_eq!(
        attr(buttons[0], "class"),
        Some("autumn-carousel-rotate"),
        "rotation must be the first control"
    );
    let rotate = buttons[0];
    assert!(attr(rotate, "hidden").is_some());
    assert_eq!(text(rotate), "Stop automatic slide show");
    assert_eq!(
        attr(rotate, "data-start-label"),
        Some("Start automatic slide show")
    );
    assert_eq!(
        attr(rotate, "data-stop-label"),
        Some("Stop automatic slide show")
    );
    assert_eq!(attr(rotate, "aria-pressed"), None);
    assert_eq!(
        attr(one(&html, "section#hero"), "data-autumn-carousel-rotate"),
        Some("6000")
    );
}

#[test]
fn ac05_rotation_interval_has_a_floor() {
    let config =
        CarouselConfig::new("hero", "Featured").rotation_interval(Duration::from_millis(200));
    assert_eq!(config.rotation(), Some(Duration::from_secs(1)));
    let config = CarouselConfig::new("hero", "Featured").rotation_interval(Duration::from_secs(9));
    let html = render(&text_slides(2), &config);
    assert_eq!(
        attr(one(&html, "section#hero"), "data-autumn-carousel-rotate"),
        Some("9000")
    );
    assert_eq!(
        CarouselConfig::new("a", "b").auto_rotate().rotation(),
        Some(DEFAULT_ROTATION_INTERVAL)
    );
}

#[test]
fn ac06_markers_link_to_slides_and_mark_the_start() {
    let html = render(&text_slides(3), &CarouselConfig::new("hero", "Featured"));
    let group = one(&html, ".autumn-carousel-markers");
    assert_eq!(attr(group, "role"), Some("group"));
    assert_eq!(attr(group, "aria-label"), Some("Choose a slide"));
    let markers = all(&html, "a.autumn-carousel-marker");
    assert_eq!(markers.len(), 3);
    for (i, marker) in markers.iter().enumerate() {
        assert_eq!(
            attr(*marker, "href"),
            Some(format!("#hero-slide-{}", i + 1).as_str())
        );
        assert_eq!(text(*marker), format!("{} of 3", i + 1));
        let current = attr(*marker, "aria-current");
        assert_eq!(current, (i == 0).then_some("true"));
    }
    assert!(
        attr(markers[0], "class").is_some_and(|c| c.contains("autumn-carousel-marker--dot"))
    );
}

#[test]
fn ac06_default_markers_depend_on_the_layout() {
    assert_eq!(CarouselConfig::new("a", "b").marker_kind(), Markers::Dots);
    let rail = CarouselConfig::new("a", "b").rail(PerView::new(3));
    assert_eq!(rail.marker_kind(), Markers::None);
    assert_eq!(rail.layout(), Layout::Rail(PerView::new(3)));
    let html = render(&text_slides(6), &rail);
    assert!(all(&html, ".autumn-carousel-markers").is_empty());
    let numbers = rail.markers(Markers::Numbers);
    let html = render(&text_slides(6), &numbers);
    let markers = all(&html, "a.autumn-carousel-marker--number");
    assert_eq!(markers.len(), 6);
    let visible = one(&html, "a[href='#a-slide-2'] .autumn-carousel-marker-number");
    assert_eq!(text(visible), "2");
    assert_eq!(
        attr(visible, "aria-hidden"),
        Some("true"),
        "the visible digit must not repeat the hidden name"
    );
}

#[test]
fn ac07_thumbnails_use_the_slide_image_or_the_override() {
    let slides = [
        Slide::image("/a.jpg", "A red chair", 800, 600),
        Slide::image("/b.jpg", "A blue chair", 800, 600).thumbnail("/b-small.jpg"),
        Slide::new(maud::html! { p { "Text" } }).label("Details"),
    ];
    let config = CarouselConfig::new("gallery", "Chair photos").markers(Markers::Thumbnails);
    let html = render(&slides, &config);
    let thumbs = all(&html, "a.autumn-carousel-marker--thumbnail");
    assert_eq!(thumbs.len(), 3);
    let img = |n: usize| {
        thumbs[n]
            .select(&scraper::Selector::parse("img").unwrap_or_else(|e| panic!("{e}")))
            .next()
    };
    assert_eq!(img(0).and_then(|i| attr(i, "src")), Some("/a.jpg"));
    assert_eq!(img(1).and_then(|i| attr(i, "src")), Some("/b-small.jpg"));
    assert!(img(2).is_none(), "a text slide has no thumbnail image");
    assert_eq!(img(0).and_then(|i| attr(i, "alt")), Some(""));
    assert_eq!(img(0).and_then(|i| attr(i, "loading")), Some("lazy"));
    assert_eq!(text(thumbs[0]), "1 of 3");
    assert_eq!(text(thumbs[2]), "Details");
}

#[test]
fn ac08_images_have_size_alt_and_lazy_loading() {
    let slides = [
        Slide::image("/1.jpg", "One", 640, 480),
        Slide::image("/2.jpg", "Two", 640, 480).caption(maud::html! { "Second" }),
        Slide::image("/3.jpg", "Three", 640, 480),
    ];
    let html = render(&slides, &CarouselConfig::new("g", "Photos"));
    let images = all(&html, ".autumn-carousel-slide img");
    assert_eq!(images.len(), 3);
    for img in &images {
        assert_eq!(attr(*img, "width"), Some("640"));
        assert_eq!(attr(*img, "height"), Some("480"));
        assert_eq!(attr(*img, "decoding"), Some("async"));
        assert_eq!(attr(*img, "class"), Some("autumn-carousel-image"));
    }
    assert_eq!(attr(images[0], "alt"), Some("One"));
    assert_eq!(attr(images[0], "loading"), Some("eager"));
    assert_eq!(attr(images[0], "fetchpriority"), None);
    assert_eq!(attr(images[1], "loading"), Some("lazy"));
    assert_eq!(attr(images[2], "loading"), Some("lazy"));
    let caption = one(&html, "#g-slide-2 figure figcaption.autumn-carousel-caption");
    assert_eq!(text(caption), "Second");

    let html = render(&slides, &CarouselConfig::new("g", "Photos").priority().start(1));
    let images = all(&html, ".autumn-carousel-slide img");
    assert_eq!(attr(images[0], "loading"), Some("lazy"));
    assert_eq!(attr(images[1], "loading"), Some("eager"));
    assert_eq!(attr(images[1], "fetchpriority"), Some("high"));
}

#[test]
fn ac09_no_slides_give_no_markup() {
    let html = carousel(&[], &CarouselConfig::new("x", "Empty"), &FieldOptions::new());
    assert_eq!(html.into_string(), "");
}

#[test]
fn ac09_one_slide_has_no_controls() {
    let config = CarouselConfig::new("one", "Single").auto_rotate();
    let html = render(&text_slides(1), &config);
    one(&html, "section#one");
    one(&html, ".autumn-carousel-slide");
    assert!(all(&html, "button").is_empty());
    assert!(all(&html, ".autumn-carousel-bar").is_empty());
    assert!(all(&html, "[aria-live]").is_empty());
    assert_eq!(
        attr(one(&html, "section#one"), "data-autumn-carousel-rotate"),
        None
    );
}

#[test]
fn ac09_many_slides_have_a_live_region() {
    let html = render(&text_slides(2), &CarouselConfig::new("hero", "Featured"));
    let live = one(&html, "[aria-live]");
    assert_eq!(attr(live, "aria-live"), Some("polite"));
    assert_eq!(attr(live, "aria-atomic"), Some("true"));
    assert_eq!(attr(live, "class"), Some("autumn-visually-hidden autumn-carousel-status"));
    assert_eq!(text(live), "");
}

#[test]
fn ac10_start_is_clamped_and_marked() {
    let html = render(&text_slides(3), &CarouselConfig::new("s", "Start").start(1));
    let start = one(&html, ".autumn-carousel-slide--start");
    assert_eq!(attr(start, "id"), Some("s-slide-2"));
    let current = one(&html, "[aria-current='true']");
    assert_eq!(attr(current, "href"), Some("#s-slide-2"));
    assert_eq!(
        attr(one(&html, "section#s"), "data-autumn-carousel-start"),
        Some("1")
    );

    let html = render(&text_slides(3), &CarouselConfig::new("s", "Start").start(99));
    assert_eq!(
        attr(one(&html, ".autumn-carousel-slide--start"), "id"),
        Some("s-slide-3")
    );
}

#[test]
fn ac11_all_text_comes_from_carousel_text() {
    let text_cfg = CarouselText {
        carousel: Cow::Borrowed("karussell"),
        slide: Cow::Borrowed("folie"),
        track: Cow::Borrowed("Folien"),
        markers: Cow::Borrowed("Folie wählen"),
        previous: Cow::Borrowed("Zurück"),
        next: Cow::Borrowed("Weiter"),
        stop: Cow::Borrowed("Anhalten"),
        start: Cow::Borrowed("Abspielen"),
        position: Cow::Borrowed("{n} von {total}"),
    };
    assert_eq!(text_cfg.position_of(2, 5), "2 von 5");
    let config = CarouselConfig::new("k", "Neu")
        .auto_rotate()
        .text(text_cfg);
    let html = render(&text_slides(2), &config);
    let markup = html.html();
    for english in [
        "carousel\"",
        "\"slide\"",
        "Slides",
        "Choose a slide",
        "Previous slide",
        "Next slide",
        "automatic slide show",
        " of ",
    ] {
        assert!(!markup.contains(english), "`{english}` is not translated: {markup}");
    }
    assert_eq!(
        attr(one(&html, "section#k"), "aria-roledescription"),
        Some("karussell")
    );
    assert_eq!(attr(one(&html, "#k-slide-1"), "aria-label"), Some("1 von 2"));
    assert_eq!(
        attr(one(&html, ".autumn-carousel-status"), "data-position"),
        Some("{n} von {total}")
    );
}

#[test]
fn ac11_default_position_text() {
    assert_eq!(CarouselText::default().position_of(3, 7), "3 of 7");
}

#[test]
fn ac12_root_gets_the_app_passthrough() {
    let options = FieldOptions::new()
        .data("test-id", "hero")
        .hx("get", "/slides");
    let html = carousel(
        &text_slides(2),
        &CarouselConfig::new("hero", "Featured"),
        &options,
    )
    .into_string();
    assert!(html.starts_with("<section "), "{html}");
    let root_tag = &html[..html.find('>').unwrap_or(html.len())];
    assert!(root_tag.contains(r#"data-test-id="hero""#), "{root_tag}");
    assert!(root_tag.contains(r#"hx-get="/slides""#), "{root_tag}");
}

#[test]
fn ac12_widget_attributes_win_over_the_passthrough() {
    let options = FieldOptions::new().data("autumn-carousel", "app");
    let html = carousel(
        &text_slides(2),
        &CarouselConfig::new("hero", "Featured"),
        &options,
    )
    .into_string();
    assert_eq!(html.matches("data-autumn-carousel=").count(), 1, "{html}");
    assert!(!html.contains(r#"data-autumn-carousel="app""#), "{html}");
}

#[test]
#[should_panic(expected = "carousel id")]
fn ac13_an_id_with_a_space_panics() {
    let _ = CarouselConfig::new("my carousel", "Bad");
}

#[test]
#[should_panic(expected = "carousel id")]
fn ac13_an_empty_id_panics() {
    let _ = CarouselConfig::new("", "Bad");
}

#[test]
#[should_panic(expected = "carousel id")]
fn ac13_an_id_that_starts_with_a_digit_panics() {
    let _ = CarouselConfig::new("1hero", "Bad");
}

#[test]
fn ac13_valid_ids_do_not_panic() {
    for id in ["a", "hero", "Hero_2", "product-42-gallery"] {
        let _ = CarouselConfig::new(id, "Good");
    }
}

#[test]
fn ac14_markup_is_csp_safe() {
    let config = CarouselConfig::new("all", "Everything")
        .auto_rotate()
        .rewind()
        .markers(Markers::Thumbnails);
    let slides = [
        Slide::image("/a.jpg", "A", 10, 10).caption(maud::html! { "A" }),
        Slide::new(maud::html! { a href="/x" { "Link" } }),
    ];
    let html = render(&slides, &config);
    assert!(all(&html, "script").is_empty());
    assert!(all(&html, "style").is_empty());
    assert!(all(&html, "[style]").is_empty());
    for el in all(&html, "*") {
        for (name, _) in el.value().attrs() {
            assert!(!name.starts_with("on"), "`{name}` on <{}>", el.value().name());
        }
    }
    assert_eq!(
        attr(one(&html, "section#all"), "data-autumn-carousel-rewind"),
        Some("")
    );
}

#[test]
fn rail_layout_sets_the_per_view_class() {
    let html = render(
        &text_slides(5),
        &CarouselConfig::new("r", "Cards").rail(PerView::new(4)),
    );
    let class = attr(one(&html, "section#r"), "class").unwrap_or_default();
    assert!(class.contains("autumn-carousel-root--rail"), "{class}");
    assert!(class.contains("autumn-carousel-root--per-view-4"), "{class}");
    let html = render(&text_slides(5), &CarouselConfig::new("s", "Show"));
    let class = attr(one(&html, "section#s"), "class").unwrap_or_default();
    assert!(class.contains("autumn-carousel-root--slideshow"), "{class}");
    assert!(!class.contains("per-view"), "{class}");
}

#[test]
fn per_view_is_clamped() {
    assert_eq!(PerView::new(0).get(), 1);
    assert_eq!(PerView::new(3).get(), 3);
    assert_eq!(PerView::new(200).get(), PerView::MAX);
}

#[test]
fn controls_come_before_the_track_in_dom_order() {
    let config = CarouselConfig::new("o", "Order").auto_rotate();
    let markup = render(&text_slides(3), &config).html();
    let at = |needle: &str| {
        markup
            .find(needle)
            .unwrap_or_else(|| panic!("`{needle}` is missing: {markup}"))
    };
    assert!(at("autumn-carousel-rotate") < at("autumn-carousel-prev"));
    assert!(at("autumn-carousel-prev") < at("autumn-carousel-markers"));
    assert!(at("autumn-carousel-markers") < at("autumn-carousel-next"));
    assert!(at("autumn-carousel-next") < at("autumn-carousel-track"));
    assert!(at("autumn-carousel-track") < at("autumn-carousel-status"));
}

#[test]
fn slide_content_is_kept() {
    let slides = [Slide::new(maud::html! { h3 { "Title" } a href="/go" { "Go" } })];
    let html = render(&slides, &CarouselConfig::new("c", "Content"));
    assert_eq!(text(one(&html, "#c-slide-1 h3")), "Title");
    assert_eq!(attr(one(&html, "#c-slide-1 a"), "href"), Some("/go"));
}

#[test]
fn text_is_escaped() {
    let slides = [
        Slide::new(maud::html! {}).label("<b>bold</b>"),
        Slide::image("/a.jpg?x=1&y=2", "\"quoted\"", 1, 1),
    ];
    let markup = carousel(
        &slides,
        &CarouselConfig::new("e", "<script>"),
        &FieldOptions::new(),
    )
    .into_string();
    assert!(!markup.contains("<script>"), "{markup}");
    assert!(!markup.contains("<b>"), "{markup}");
    assert!(markup.contains("&lt;script&gt;"), "{markup}");
    assert!(markup.contains("x=1&amp;y=2"), "{markup}");
}
