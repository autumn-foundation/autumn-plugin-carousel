//! The pack passes the static widget conformance checks (AC-15, AC-16)
//! and the plugin serves it (AC-17).

use autumn_plugin_carousel::{ASSETS, CarouselPlugin, PACK_NAME, pack};
use autumn_web::plugin::Plugin;
use autumn_web::plugin_conformance::widgets::run_widget_conformance;

#[test]
fn ac15_the_pack_passes_static_conformance() {
    let report = run_widget_conformance(&pack());
    assert!(report.passed(), "{}", report.to_text_report());
}

#[test]
fn ac16_each_variant_has_a_story() {
    let pack = pack();
    let slugs: Vec<&str> = pack
        .stories()
        .iter()
        .map(autumn_web::stories::Story::slug)
        .collect();
    assert_eq!(
        slugs,
        [
            "carousel-slideshow",
            "carousel-auto-rotation",
            "carousel-rail",
            "carousel-gallery",
            "carousel-numbers"
        ]
    );
    for story in pack.stories() {
        assert!(story.slug().starts_with("carousel-"));
        let html = story
            .render()
            .map(maud::Markup::into_string)
            .unwrap_or_default();
        assert!(html.contains("autumn-carousel-root"), "{}", story.slug());
    }
}

#[test]
fn ac17_the_bundle_holds_the_css_and_js() {
    assert_eq!(pack().name(), PACK_NAME);
    let paths: Vec<&str> = ASSETS
        .iter()
        .map(autumn_web::assets::PluginAsset::logical_path)
        .collect();
    assert_eq!(paths, ["carousel.css", "carousel.js"]);
    let head = pack().head().into_string();
    assert!(head.contains("<link"), "{head}");
    assert!(head.contains("defer"), "{head}");
    assert!(head.contains("integrity="), "{head}");
}

#[test]
fn ac17_the_plugin_serves_the_assets() {
    let builder = autumn_web::app().plugin(CarouselPlugin::new());
    let routes = builder.plugin_route_infos().unwrap_or_default();
    for file in ["carousel.css", "carousel.js"] {
        assert!(
            routes
                .iter()
                .any(|r| r.path == format!("/static/_plugins/carousel/{file}")),
            "{file} is not served: {routes:?}"
        );
    }
}

#[test]
fn ac17_the_plugin_declares_its_contract() {
    let plugin = CarouselPlugin::new();
    assert_eq!(plugin.name(), "autumn-plugin-carousel");
    let contract = plugin.contract().unwrap_or_else(|| panic!("no contract"));
    let report = format!("{contract:?}");
    assert!(report.contains("autumn-plugin-carousel"), "{report}");
    assert!(report.contains("0.10"), "{report}");
}

#[test]
fn ac17_installing_twice_is_safe() {
    let builder = autumn_web::app()
        .widget_pack(pack())
        .plugin(CarouselPlugin::new());
    assert!(builder.plugin_route_infos().is_ok());
}
