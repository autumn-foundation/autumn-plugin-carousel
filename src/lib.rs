//! An accessible carousel widget pack for Autumn.
//!
//! The carousel is a native scroll-snap track. All slides are in the server
//! HTML. With JavaScript off, the track scrolls and the markers are links.
//! With `carousel.js`, previous, next and rotation buttons show.
//!
//! # Install
//!
//! ```rust
//! use autumn_plugin_carousel::CarouselPlugin;
//!
//! let app = autumn_web::app().plugin(CarouselPlugin::new());
//! # let _ = app;
//! ```
//!
//! Then link the assets in the layout, after the core widget assets:
//!
//! ```rust
//! # let _ = maud::html! {
//! head {
//!     link rel="stylesheet" href=(autumn_web::ui::WIDGETS_CSS_PATH);
//!     script src="/static/js/autumn-widgets.js" defer {}
//!     (autumn_plugin_carousel::head())
//! }
//! # };
//! ```
//!
//! # Render
//!
//! ```rust
//! use autumn_plugin_carousel::{CarouselConfig, Slide, carousel};
//! use autumn_web::form::FieldOptions;
//! use maud::html;
//!
//! let slides = [
//!     Slide::image("/img/chair-front.jpg", "A red chair, front view", 800, 600),
//!     Slide::image("/img/chair-side.jpg", "A red chair, side view", 800, 600),
//! ];
//! let config = CarouselConfig::new("chair", "Chair photos");
//! let markup = carousel(&slides, &config, &FieldOptions::new());
//! # assert!(markup.into_string().contains("chair--slide-2"));
//! ```
//!
//! The repository file `docs/design.md` gives the markup, the behavior
//! and the acceptance criteria.

mod config;
mod render;
mod slide;
mod stories;

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::plugin_contract::PluginContract;
use autumn_web::widget_pack::WidgetPack;

pub use config::{
    CarouselConfig, CarouselText, DEFAULT_ROTATION_INTERVAL, InvalidCarouselId, Layout,
    MAX_ROTATION_INTERVAL, MIN_ROTATION_INTERVAL, Markers, PerView,
};
pub use render::carousel;
pub use slide::Slide;

/// The pack name. Classes start with `autumn-carousel-`.
pub const PACK_NAME: &str = "carousel";

/// The `autumn-web` versions that this crate supports.
pub const AUTUMN_WEB_RANGE: &str = "0.10";

/// The pack CSS and JS.
pub static ASSETS: PluginAssets = PluginAssets::from_files(
    PACK_NAME,
    &[
        ("carousel.css", include_bytes!("carousel.css")),
        ("carousel.js", include_bytes!("carousel.js")),
    ],
);

/// The pack: its stories and its assets.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new(PACK_NAME)
        .extend(stories::all())
        .assets(&ASSETS)
}

/// The `<link>` and `<script defer>` tags of the pack, with SRI. Put them
/// in the layout `<head>`, after the core widget assets.
#[must_use]
pub fn head() -> maud::Markup {
    maud::html! {
        (ASSETS.stylesheet_tag("carousel.css"))
        (ASSETS.deferred_script_tag("carousel.js"))
    }
}

/// The plugin. It serves the pack assets and adds the stories to the
/// `/_stories` gallery.
#[derive(Debug, Default, Clone, Copy)]
pub struct CarouselPlugin;

impl CarouselPlugin {
    /// Make the plugin.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for CarouselPlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(env!("CARGO_PKG_NAME"))
    }

    fn contract(&self) -> Option<PluginContract> {
        Some(
            PluginContract::new(env!("CARGO_PKG_NAME"))
                .plugin_version(env!("CARGO_PKG_VERSION"))
                .autumn_web(AUTUMN_WEB_RANGE),
        )
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.widget_pack(pack())
    }
}
