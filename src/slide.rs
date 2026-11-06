//! One slide.

use maud::Markup;

/// The content of a slide.
#[derive(Debug, Clone)]
pub(crate) enum Content<'a> {
    /// App markup.
    Markup(Markup),
    /// An image.
    Image {
        src: &'a str,
        alt: &'a str,
        width: u32,
        height: u32,
        caption: Option<Markup>,
    },
}

/// One slide of a carousel.
#[derive(Debug, Clone)]
pub struct Slide<'a> {
    pub(crate) content: Content<'a>,
    pub(crate) label: Option<&'a str>,
    pub(crate) thumbnail: Option<&'a str>,
}

impl<'a> Slide<'a> {
    /// A slide with app markup.
    #[must_use]
    pub const fn new(content: Markup) -> Self {
        Self {
            content: Content::Markup(content),
            label: None,
            thumbnail: None,
        }
    }

    /// A slide with one image.
    ///
    /// `width` and `height` are the image size in pixels. The browser uses
    /// them to keep space for the image. Give `alt` the text that the image
    /// shows. Give an empty `alt` only when the image is decoration.
    #[must_use]
    pub const fn image(src: &'a str, alt: &'a str, width: u32, height: u32) -> Self {
        Self {
            content: Content::Image {
                src,
                alt,
                width,
                height,
                caption: None,
            },
            label: None,
            thumbnail: None,
        }
    }

    /// Give the slide a unique name. The default name is "N of M".
    ///
    /// Do not put the word "slide" in the name. The role name gives it.
    #[must_use]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Add a caption under an image. A markup slide ignores it.
    #[must_use]
    pub fn caption(mut self, caption: Markup) -> Self {
        let _ = caption;
        self
    }

    /// The image for a [`Markers::Thumbnails`](crate::Markers::Thumbnails)
    /// marker. The default is the slide image.
    #[must_use]
    pub const fn thumbnail(mut self, src: &'a str) -> Self {
        self.thumbnail = Some(src);
        self
    }
}
