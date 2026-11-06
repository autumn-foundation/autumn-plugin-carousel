//! Carousel settings.

use std::borrow::Cow;
use std::time::Duration;

/// The rotation interval of [`CarouselConfig::auto_rotate`].
pub const DEFAULT_ROTATION_INTERVAL: Duration = Duration::from_secs(6);

/// The shortest rotation interval. A shorter value becomes this value.
pub const MIN_ROTATION_INTERVAL: Duration = Duration::from_secs(1);

/// How many slides show at one time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// One slide fills the track. This is the default.
    Slideshow,
    /// Many cards show at one time.
    Rail(PerView),
}

/// The number of cards in view in a [`Layout::Rail`], from 1 to 6.
///
/// On a screen narrower than 48rem, a rail shows one card and a part of
/// the next card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerView(u8);

impl PerView {
    /// The largest value.
    pub const MAX: u8 = 6;

    /// Make a value. A value below 1 becomes 1. A value above 6 becomes 6.
    #[must_use]
    pub const fn new(count: u8) -> Self {
        if count == 0 {
            Self(1)
        } else if count > Self::MAX {
            Self(Self::MAX)
        } else {
            Self(count)
        }
    }

    /// The value.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// The markers that select a slide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Markers {
    /// No markers.
    None,
    /// One dot for each slide.
    Dots,
    /// One number for each slide.
    Numbers,
    /// One small image for each slide.
    Thumbnails,
}

/// All text that the carousel shows or speaks.
///
/// Change it to translate the carousel. In `position`, `{n}` is the slide
/// number and `{total}` is the number of slides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CarouselText<'a> {
    /// The role name of the root. Default: `carousel`.
    pub carousel: Cow<'a, str>,
    /// The role name of each slide. Default: `slide`.
    pub slide: Cow<'a, str>,
    /// The name of the slide track. Default: `Slides`.
    pub track: Cow<'a, str>,
    /// The name of the marker group. Default: `Choose a slide`.
    pub markers: Cow<'a, str>,
    /// The previous button. Default: `Previous slide`.
    pub previous: Cow<'a, str>,
    /// The next button. Default: `Next slide`.
    pub next: Cow<'a, str>,
    /// The rotation button while rotation is on.
    /// Default: `Stop automatic slide show`.
    pub stop: Cow<'a, str>,
    /// The rotation button while rotation is off.
    /// Default: `Start automatic slide show`.
    pub start: Cow<'a, str>,
    /// The slide position. Default: `{n} of {total}`.
    pub position: Cow<'a, str>,
}

impl Default for CarouselText<'_> {
    fn default() -> Self {
        Self {
            carousel: Cow::Borrowed("carousel"),
            slide: Cow::Borrowed("slide"),
            track: Cow::Borrowed("Slides"),
            markers: Cow::Borrowed("Choose a slide"),
            previous: Cow::Borrowed("Previous slide"),
            next: Cow::Borrowed("Next slide"),
            stop: Cow::Borrowed("Stop automatic slide show"),
            start: Cow::Borrowed("Start automatic slide show"),
            position: Cow::Borrowed("{n} of {total}"),
        }
    }
}

impl CarouselText<'_> {
    /// The position text for slide `n` (from 1) of `total`.
    #[must_use]
    // `{n}` and `{total}` are template slots, not format arguments.
    #[allow(clippy::literal_string_with_formatting_args)]
    pub fn position_of(&self, n: usize, total: usize) -> String {
        self.position
            .replace("{n}", &n.to_string())
            .replace("{total}", &total.to_string())
    }
}

/// The settings of one carousel.
#[derive(Debug, Clone)]
pub struct CarouselConfig<'a> {
    pub(crate) id: &'a str,
    pub(crate) label: &'a str,
    pub(crate) labelled_by: Option<&'a str>,
    pub(crate) layout: Layout,
    pub(crate) markers: Option<Markers>,
    pub(crate) rotation: Option<Duration>,
    pub(crate) rewind: bool,
    pub(crate) start: usize,
    pub(crate) priority: bool,
    pub(crate) text: CarouselText<'a>,
}

impl<'a> CarouselConfig<'a> {
    /// A slideshow with the root `id` and the accessible name `label`.
    ///
    /// Do not put the word "carousel" in `label`. The role name gives it.
    ///
    /// # Panics
    ///
    /// Panics when `id` is not `[A-Za-z][A-Za-z0-9_-]*`. The id goes into
    /// fragment links and `aria-controls`.
    #[must_use]
    pub fn new(id: &'a str, label: &'a str) -> Self {
        assert!(
            is_valid_id(id),
            "carousel id {id:?} is not valid: use an ASCII letter, then ASCII \
             letters, digits, `-` or `_`"
        );
        Self {
            id,
            label,
            labelled_by: None,
            layout: Layout::Slideshow,
            markers: None,
            rotation: None,
            rewind: false,
            start: 0,
            priority: false,
            text: CarouselText::default(),
        }
    }

    /// Use the element `id` (for example a visible heading) as the name.
    #[must_use]
    pub const fn labelled_by(mut self, id: &'a str) -> Self {
        self.labelled_by = Some(id);
        self
    }

    /// Show many cards at one time.
    #[must_use]
    pub const fn rail(mut self, per_view: PerView) -> Self {
        self.layout = Layout::Rail(per_view);
        self
    }

    /// Set the markers.
    #[must_use]
    pub const fn markers(mut self, markers: Markers) -> Self {
        self.markers = Some(markers);
        self
    }

    /// Turn on automatic rotation, every 6 s.
    #[must_use]
    pub const fn auto_rotate(mut self) -> Self {
        self.rotation = Some(DEFAULT_ROTATION_INTERVAL);
        self
    }

    /// Turn on automatic rotation with an interval.
    ///
    /// An interval below [`MIN_ROTATION_INTERVAL`] becomes that value. Give
    /// users time to read: 5 s or more is the usual advice.
    #[must_use]
    pub const fn rotation_interval(mut self, interval: Duration) -> Self {
        self.rotation = Some(if interval.as_nanos() < MIN_ROTATION_INTERVAL.as_nanos() {
            MIN_ROTATION_INTERVAL
        } else {
            interval
        });
        self
    }

    /// Wrap at the ends: next on the last slide goes to the first slide.
    #[must_use]
    pub const fn rewind(mut self) -> Self {
        self.rewind = true;
        self
    }

    /// Show slide `index` (from 0) first.
    #[must_use]
    pub const fn start(mut self, index: usize) -> Self {
        self.start = index;
        self
    }

    /// Load the start image first, with `fetchpriority="high"`.
    #[must_use]
    pub const fn priority(mut self) -> Self {
        self.priority = true;
        self
    }

    /// Set the text.
    #[must_use]
    pub fn text(mut self, text: CarouselText<'a>) -> Self {
        self.text = text;
        self
    }

    /// The layout.
    #[must_use]
    pub const fn layout(&self) -> Layout {
        self.layout
    }

    /// The markers. The default is [`Markers::Dots`] for a slideshow and
    /// [`Markers::None`] for a rail.
    #[must_use]
    pub const fn marker_kind(&self) -> Markers {
        match (self.markers, self.layout) {
            (Some(markers), _) => markers,
            (None, Layout::Slideshow) => Markers::Dots,
            (None, Layout::Rail(_)) => Markers::None,
        }
    }

    /// The rotation interval, or `None` when rotation is off.
    #[must_use]
    pub const fn rotation(&self) -> Option<Duration> {
        self.rotation
    }
}

/// `true` when `id` is `[A-Za-z][A-Za-z0-9_-]*`.
fn is_valid_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
