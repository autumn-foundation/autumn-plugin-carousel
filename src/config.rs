//! Carousel settings.

use std::time::Duration;

/// The rotation interval of [`CarouselConfig::auto_rotate`].
pub const DEFAULT_ROTATION_INTERVAL: Duration = Duration::from_secs(6);

/// The shortest rotation interval. A shorter value becomes this value.
pub const MIN_ROTATION_INTERVAL: Duration = Duration::from_secs(1);

/// The longest rotation interval: the largest browser timer delay
/// (2^31 − 1 ms, about 24.8 days). A longer value becomes this value.
pub const MAX_ROTATION_INTERVAL: Duration = Duration::from_millis(2_147_483_647);

/// How many slides show at one time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Layout {
    /// One slide fills the track. This is the default.
    #[default]
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
#[non_exhaustive]
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
/// Start from [`CarouselText::new`] and change the text that you translate:
///
/// ```rust
/// use autumn_plugin_carousel::CarouselText;
///
/// let text = CarouselText::new().previous("Zurück").next("Weiter").position("{n} von {total}");
/// assert_eq!(text.position_of(2, 5), "2 von 5");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarouselText<'a> {
    pub(crate) carousel: &'a str,
    pub(crate) slide: &'a str,
    pub(crate) track: &'a str,
    pub(crate) markers: &'a str,
    pub(crate) previous: &'a str,
    pub(crate) next: &'a str,
    pub(crate) stop: &'a str,
    pub(crate) start: &'a str,
    pub(crate) position: &'a str,
    pub(crate) range: &'a str,
}

impl Default for CarouselText<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> CarouselText<'a> {
    /// The English text.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            carousel: "carousel",
            slide: "slide",
            track: "Slides",
            markers: "Choose a slide",
            previous: "Previous slide",
            next: "Next slide",
            stop: "Stop automatic slide show",
            start: "Start automatic slide show",
            position: "{n} of {total}",
            range: "{first} to {last} of {total}",
        }
    }

    /// The role name of the root. Default: `carousel`.
    #[must_use]
    pub const fn carousel(mut self, text: &'a str) -> Self {
        self.carousel = text;
        self
    }

    /// The role name of each slide. Default: `slide`.
    #[must_use]
    pub const fn slide(mut self, text: &'a str) -> Self {
        self.slide = text;
        self
    }

    /// The name of the track. Default: `Slides`.
    #[must_use]
    pub const fn track(mut self, text: &'a str) -> Self {
        self.track = text;
        self
    }

    /// The name of the marker group. Default: `Choose a slide`.
    #[must_use]
    pub const fn markers(mut self, text: &'a str) -> Self {
        self.markers = text;
        self
    }

    /// The previous button. Default: `Previous slide`.
    #[must_use]
    pub const fn previous(mut self, text: &'a str) -> Self {
        self.previous = text;
        self
    }

    /// The next button. Default: `Next slide`.
    #[must_use]
    pub const fn next(mut self, text: &'a str) -> Self {
        self.next = text;
        self
    }

    /// The rotation button while rotation is on.
    /// Default: `Stop automatic slide show`.
    #[must_use]
    pub const fn stop(mut self, text: &'a str) -> Self {
        self.stop = text;
        self
    }

    /// The rotation button while rotation is off.
    /// Default: `Start automatic slide show`.
    #[must_use]
    pub const fn start(mut self, text: &'a str) -> Self {
        self.start = text;
        self
    }

    /// The name of one slide. `{n}` is the slide number and `{total}` is
    /// the number of slides. Default: `{n} of {total}`.
    #[must_use]
    pub const fn position(mut self, text: &'a str) -> Self {
        self.position = text;
        self
    }

    /// The text that a rail speaks after a move. `{first}` and `{last}`
    /// are the numbers of the first and the last card in view.
    /// Default: `{first} to {last} of {total}`.
    #[must_use]
    pub const fn range(mut self, text: &'a str) -> Self {
        self.range = text;
        self
    }

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

/// The id is not valid. See [`CarouselConfig::try_new`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "carousel id {0:?} is not valid: use ASCII letters, digits, `-` and `_`, \
     with no `--` and no `-` at the start or the end"
)]
pub struct InvalidCarouselId(pub String);

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
    /// Give each carousel on a page a different id. The carousel adds
    /// `--track` and `--slide-N` to the id for its parts.
    ///
    /// # Panics
    ///
    /// Panics when the id is not valid (see [`try_new`](Self::try_new)).
    /// Use `try_new` for an id from data.
    #[must_use]
    pub fn new(id: &'a str, label: &'a str) -> Self {
        match Self::try_new(id, label) {
            Ok(config) => config,
            Err(error) => panic!("{error}"),
        }
    }

    /// Like [`new`](Self::new), but returns an error for an id that is not
    /// valid.
    ///
    /// A valid id has one or more ASCII letters, digits, `-` and `_`. It
    /// has no `--`, and no `-` at the start or the end. Thus the ids of the
    /// carousel parts cannot be the same as the ids of another carousel.
    ///
    /// ```rust
    /// use autumn_plugin_carousel::CarouselConfig;
    ///
    /// assert!(CarouselConfig::try_new("product-42", "Photos").is_ok());
    /// assert!(CarouselConfig::try_new("product 42", "Photos").is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// [`InvalidCarouselId`] when the id is not valid.
    pub fn try_new(id: &'a str, label: &'a str) -> Result<Self, InvalidCarouselId> {
        if !is_valid_id(id) {
            return Err(InvalidCarouselId(id.to_owned()));
        }
        Ok(Self {
            id,
            label,
            labelled_by: None,
            layout: Layout::Slideshow,
            markers: None,
            rotation: None,
            rewind: false,
            start: 0,
            priority: false,
            text: CarouselText::new(),
        })
    }

    /// Use the element `id` (for example a visible heading) as the name.
    /// The `label` is then not used.
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
    /// The interval is clamped to [`MIN_ROTATION_INTERVAL`] and
    /// [`MAX_ROTATION_INTERVAL`]. Give users time to read: 5 s or more is
    /// the usual advice.
    #[must_use]
    pub const fn rotation_interval(mut self, interval: Duration) -> Self {
        self.rotation = Some(if interval.as_nanos() < MIN_ROTATION_INTERVAL.as_nanos() {
            MIN_ROTATION_INTERVAL
        } else if interval.as_nanos() > MAX_ROTATION_INTERVAL.as_nanos() {
            MAX_ROTATION_INTERVAL
        } else {
            interval
        });
        self
    }

    /// Wrap at the ends: next on the last slide goes to the first slide,
    /// and previous on the first slide goes to the last slide.
    #[must_use]
    pub const fn rewind(mut self) -> Self {
        self.rewind = true;
        self
    }

    /// Show slide `index` (from 0) first. An index after the last slide
    /// becomes the last slide.
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
    pub const fn text(mut self, text: CarouselText<'a>) -> Self {
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

/// `true` when `id` has ASCII letters, digits, `-` and `_` only, and no
/// `--`, and no `-` at the start or the end.
fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        && !id.contains("--")
        && !id.starts_with('-')
        && !id.ends_with('-')
}
