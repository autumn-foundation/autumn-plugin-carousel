//! An accessible carousel widget pack for Autumn.

mod config;
mod render;
mod slide;

use autumn_web::app::AppBuilder;
use autumn_web::assets::PluginAssets;
use autumn_web::plugin::Plugin;
use autumn_web::widget_pack::WidgetPack;

pub use config::{
    CarouselConfig, CarouselText, DEFAULT_ROTATION_INTERVAL, Layout, MIN_ROTATION_INTERVAL,
    Markers, PerView,
};
pub use render::carousel;
pub use slide::Slide;

/// The pack name.
pub const PACK_NAME: &str = "carousel";

/// The pack CSS and JS.
pub static ASSETS: PluginAssets = PluginAssets::from_files("carousel", &[]);

/// The pack.
#[must_use]
pub fn pack() -> WidgetPack {
    WidgetPack::new(PACK_NAME)
}

/// The plugin.
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
    fn build(self, app: AppBuilder) -> AppBuilder {
        app
    }
}
