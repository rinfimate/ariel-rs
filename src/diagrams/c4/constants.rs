//! Layout and styling constants for the C4 diagram renderer.
//!
//! Values match Mermaid's defaultConfig.c4 unless otherwise noted.

// ---------------------------------------------------------------------------
// Diagram margins (diagramMarginX / diagramMarginY)
// ---------------------------------------------------------------------------

/// Horizontal margin around the diagram canvas (px).
pub(crate) const DIAGRAM_MARGIN_X: f64 = 50.0;

/// Vertical margin around the diagram canvas (px).
pub(crate) const DIAGRAM_MARGIN_Y: f64 = 10.0;

// ---------------------------------------------------------------------------
// Element spacing
// ---------------------------------------------------------------------------

/// Margin between elements and between elements and boundary edges (px).
pub(crate) const C4_SHAPE_MARGIN: f64 = 50.0;

/// Internal padding inside each element box (px).
pub(crate) const C4_SHAPE_PADDING: f64 = 20.0;

/// Minimum element width (px).
pub(crate) const MIN_WIDTH: f64 = 216.0;

/// Minimum element height (px).
pub(crate) const MIN_HEIGHT: f64 = 60.0;

/// Maximum number of elements per row inside a boundary.
pub(crate) const C4_SHAPE_IN_ROW: usize = 4;

/// Maximum number of top-level boundaries per row.
pub(crate) const C4_BOUNDARY_IN_ROW: usize = 2;

/// Extra x-padding added when wrapping to the next line (px).
pub(crate) const NEXT_LINE_PADDING_X: f64 = 0.0;

/// Epsilon for floating-point near-zero comparisons in intersection math.
pub(crate) const EPS: f64 = 1e-9;

/// Viewport width used for row-wrapping decisions (px).
///
/// The reference SVG was generated in an ~800 px viewport; 768 (common mobile
/// width) gives the correct wrap point where `ex ≥ width_limit` fires.
pub(crate) const SCREEN_WIDTH: f64 = 768.0;

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// Font size for stereotype labels (e.g. `<<system>>`, px).
pub(crate) const STEREO_FONT_SIZE: f64 = 12.0;

/// Font size for element name labels (px).
pub(crate) const LABEL_FONT_SIZE: f64 = 16.0;

/// Font size for element description text (px).
pub(crate) const DESCR_FONT_SIZE: f64 = 14.0;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

// Fill/stroke colour reference per element type — see element_colors():
// Person:         #08427B / #073B6F
// PersonExt:      #686868 / #8A8A8A
// System:         #1168BD / #3C7FC0
// SystemExt:      #999999 / #8A8A8A
// Container:      #438DD5 / #3C7FC0
// ContainerExt:   #B3B3B3 / #A6A6A6
// Component:      #85BBF0 / #78A8D8
// ComponentExt:   #CCCCCC / #BFBFBF
// Node:           #438DD5 / #3C7FC0
// NodeExt:        #B3B3B3 / #A6A6A6

/// Text fill for boundary labels and relationship text (dark grey).
pub(crate) const BOUNDARY_TEXT_COLOR: &str = "#444444";

/// Stroke colour for boundary rectangles.
#[allow(dead_code)]
pub(crate) const BOUNDARY_STROKE: &str = "#444444";

/// Text fill for element shape labels (white, on coloured backgrounds).
pub(crate) const SHAPE_TEXT_COLOR: &str = "#FFFFFF";

// ---------------------------------------------------------------------------
// Message font
// ---------------------------------------------------------------------------

/// Font size used for relationship message labels (conf.messageFontSize, px).
pub(crate) const MSG_FONT_SIZE: f64 = 12.0;

/// Font family for shape labels (Open Sans, quoted for use in style="" attributes).
pub(crate) const FF_SHAPE: &str = "&quot;Open Sans&quot;, sans-serif";

/// Font family for relationship message labels (trebuchet ms).
pub(crate) const FF_MSG_LABEL: &str = "&quot;Open Sans&quot;, sans-serif";

/// Font family for relationship technology labels.
pub(crate) const FF_TECHN_LABEL: &str = "&quot;trebuchet ms&quot;, verdana, arial, sans-serif";

// ---------------------------------------------------------------------------
// SVG icon paths
// ---------------------------------------------------------------------------

/// SVG path data for the computer/desktop icon (scaled ×0.5).
pub(crate) const COMPUTER_ICON_PATH: &str = "M2 2v13h20v-13h-20zm18 11h-16v-9h16v9zm-10.228 6l.466-1h3.524l.467 1h-4.457zm14.228 3h-24l2-6h2.104l-1.33 4h18.45l-1.297-4h2.073l2 6zm-5-10h-14v-7h14v7z";

/// SVG path data for the database icon (scaled ×0.5).
pub(crate) const DATABASE_ICON_PATH: &str = "M12.258.001l.256.004.255.005.253.008.251.01.249.012.247.015.246.016.242.019.241.02.239.023.236.024.233.027.231.028.229.031.225.032.223.034.22.036.217.038.214.04.211.041.208.043.205.045.201.046.198.048.194.05.191.051.187.053.183.054.18.056.175.057.172.059.168.06.163.061.16.063.155.064.15.066.074.033.073.033.071.034.07.034.069.035.068.035.067.035.066.035.064.036.064.036.062.036.06.036.06.037.058.037.058.037.055.038.055.038.053.038.052.038.051.039.05.039.048.039.047.039.045.04.044.04.043.04.041.04.04.041.039.041.037.041.036.041.034.041.033.042.032.042.03.042.029.042.027.042.026.043.024.043.023.043.021.043.02.043.018.044.017.043.015.044.013.044.012.044.011.045.009.044.007.045.006.045.004.045.002.045.001.045v17l-.001.045-.002.045-.004.045-.006.045-.007.045-.009.044-.011.045-.012.044-.013.044-.015.044-.017.043-.018.044-.02.043-.021.043-.023.043-.024.043-.026.043-.027.042-.029.042-.03.042-.032.042-.033.042-.034.041-.036.041-.037.041-.039.041-.04.041-.041.04-.043.04-.044.04-.045.04-.047.039-.048.039-.05.039-.051.039-.052.038-.053.038-.055.038-.055.038-.058.037-.058.037-.06.037-.06.036-.062.036-.064.036-.064.036-.066.035-.067.035-.068.035-.069.035-.07.034-.071.034-.073.033-.074.033-.15.066-.155.064-.16.063-.163.061-.168.06-.172.059-.175.057-.18.056-.183.054-.187.053-.191.051-.194.05-.198.048-.201.046-.205.045-.208.043-.211.041-.214.04-.217.038-.22.036-.223.034-.225.032-.229.031-.231.028-.233.027-.236.024-.239.023-.241.02-.242.019-.246.016-.247.015-.249.012-.251.01-.253.008-.255.005-.256.004-.258.001-.258-.001-.256-.004-.255-.005-.253-.008-.251-.01-.249-.012-.247-.015-.245-.016-.243-.019-.241-.02-.238-.023-.236-.024-.234-.027-.231-.028-.228-.031-.226-.032-.223-.034-.22-.036-.217-.038-.214-.04-.211-.041-.208-.043-.204-.045-.201-.046-.198-.048-.195-.05-.19-.051-.187-.053-.184-.054-.179-.056-.176-.057-.172-.059-.167-.06-.164-.061-.159-.063-.155-.064-.151-.066-.074-.033-.072-.033-.072-.034-.07-.034-.069-.035-.068-.035-.067-.035-.066-.035-.064-.036-.063-.036-.062-.036-.061-.036-.06-.037-.058-.037-.057-.037-.056-.038-.055-.038-.053-.038-.052-.038-.051-.039-.049-.039-.049-.039-.046-.039-.046-.04-.044-.04-.043-.04-.041-.04-.04-.041-.039-.041-.037-.041-.036-.041-.034-.041-.033-.042-.032-.042-.03-.042-.029-.042-.027-.042-.026-.043-.024-.043-.023-.043-.021-.043-.02-.043-.018-.044-.017-.043-.015-.044-.013-.044-.012-.044-.011-.045-.009-.044-.007-.045-.006-.045-.004-.045-.002-.045-.001-.045v-17l.001-.045.002-.045.004-.045.006-.045.007-.045.009-.044.011-.045.012-.044.013-.044.015-.044.017-.043.018-.044.02-.043.021-.043.023-.043.024-.043.026-.043.027-.042.029-.042.03-.042.032-.042.033-.042.034-.041.036-.041.037-.041.039-.041.04-.041.041-.04.043-.04.044-.04.046-.04.046-.039.049-.039.049-.039.051-.039.052-.038.053-.038.055-.038.056-.038.057-.037.058-.037.06-.037.061-.036.062-.036.063-.036.064-.036.066-.035.067-.035.068-.035.069-.035.07-.034.072-.034.072-.033.074-.033.151-.066.155-.064.159-.063.164-.061.167-.06.172-.059.176-.057.179-.056.184-.054.187-.053.19-.051.195-.05.198-.048.201-.046.204-.045.208-.043.211-.041.214-.04.217-.038.22-.036.223-.034.226-.032.228-.031.231-.028.234-.027.236-.024.238-.023.241-.02.243-.019.245-.016.247-.015.249-.012.251-.01.253-.008.255-.005.256-.004.258-.001.258.001z";

/// SVG path data for the clock icon (scaled ×0.5).
pub(crate) const CLOCK_ICON_PATH: &str = "M12 2c5.514 0 10 4.486 10 10s-4.486 10-10 10-10-4.486-10-10 4.486-10 10-10zm0-2c-6.627 0-12 5.373-12 12s5.373 12 12 12 12-5.373 12-12-5.373-12-12-12zm5.848 12.459c.202.038.202.333.001.372-1.907.361-6.045 1.111-6.547 1.111-.719 0-1.301-.582-1.301-1.301 0-.512.77-5.447 1.125-7.445.034-.192.312-.181.343.014l.985 6.238 5.394 1.011z";

// ---------------------------------------------------------------------------
// Person icon
// ---------------------------------------------------------------------------

/// Base64-encoded PNG icon used for Person elements.
pub const PERSON_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAIAAADYYG7QAAACD0lEQVR4Xu2YoU4EMRCGT+4j8Ai8AhaH4QHgAUjQuFMECUgMIUgwJAgMhgQsAYUiJCiQIBBY+EITsjfTdme6V24v4c8vyGbb+ZjOtN0bNcvjQXmkH83WvYBWto6PLm6v7p7uH1/w2fXD+PBycX1Pv2l3IdDm/vn7x+dXQiAubRzoURa7gRZWd0iGRIiJbOnhnfYBQZNJjNbuyY2eJG8fkDE3bbG4ep6MHUAsgYxmE3nVs6VsBWJSGccsOlFPmLIViMzLOB7pCVO2AtHJMohH7Fh6zqitQK7m0rJvAVYgGcEpe//PLdDz65sM4pF9N7ICcXDKIB5Nv6j7tD0NoSdM2QrU9Gg0ewE1LqBhHR3BBdvj2vapnidjHxD/q6vd7Pvhr31AwcY8eXMTXAKECZZJFXuEq27aLgQK5uLMohCenGGuGewOxSjBvYBqeG6B+Nqiblggdjnc+ZXDy+FNFpFzw76O3UBAROuXh6FoiAcf5g9eTvUgzy0nWg6I8cXHRUpg5bOVBCo+KDpFajOf23GgPme7RSQ+lacIENUgJ6gg1k6HjgOlqnLqip4tEuhv0hNEMXUD0clyXE3p6pZA0S2nnvTlXwLJEZWlb7cTQH1+USgTN4VhAenm/wea1OCAOmqo6fE1WCb9WSKBah+rbUWPWAmE2Rvk0ApiB45eOyNAzU8xcTvj8KvkKEoOaIYeHNA3ZuygAvFMUO0AAAAASUVORK5CYII=";

/// Base64-encoded PNG icon used for external Person elements (transparent background).
pub const PERSON_EXT_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAADAAAAAwCAIAAADYYG7QAAAB6ElEQVR4Xu2YLY+EMBCG9+dWr0aj0Wg0Go1Go0+j8Xdv2uTCvv1gpt0ebHKPuhDaeW4605Z9mJvx4AdXUyTUdd08z+u6flmWZRnHsWkafk9DptAwDPu+f0eAYtu2PEaGWuj5fCIZrBAC2eLBAnRCsEkkxmeaJp7iDJ2QMDdHsLg8SxKFEJaAo8lAXnmuOFIhTMpxxKATebo4UiFknuNo4OniSIXQyRxEA3YsnjGCVEjVXD7yLUAqxBGUyPv/Y4W2beMgGuS7kVQIBycH0fD+oi5pezQETxdHKmQKGk1eQEYldK+jw5GxPfZ9z7Mk0Qnhf1W1m3w//EUn5BDmSZsbR44QQLBEqrBHqOrmSKaQAxdnLArCrxZcM7A7ZKs4ioRq8LFC+NpC3WCBJsvpVw5edm9iEXFuyNfxXAgSwfrFQ1c0iNda8AdejvUgnktOtJQQxmcfFzGglc5WVCj7oDgFqU18boeFSs52CUh8LE8BIVQDT1ABrB0HtgSEYlX5doJnCwv9TXocKCaKbnwhdDKPq4lf3SwU3HLq4V/+WYhHVMa/3b4IlfyikAduCkcBc7mQ3/z/Qq/cTuikhkzB12Ae/mcJC9U+Vo8Ej1gWAtgbeGgFsAMHr50BIWOLCbezvhpBFUdY6EJuJ/QDW0XoMX60zZ0AAAAASUVORK5CYII=";
