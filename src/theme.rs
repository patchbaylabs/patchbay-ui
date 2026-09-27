//! The booth theme shared by mictrace, sidestage and rcpmix: one module
//! holding every colour, size and shape any of their UIs is allowed to use.
//!
//! Originally mictrace's `theme.rs`, generalized here so every embedding app
//! reads as one product when its window sits next to a sibling's on the same
//! operator's screen. Values that only ever made sense in one app (meter
//! ramps, RF traces, LTC seven-segment colours, a chat flash's strobe
//! colours) stay in that app; nothing that survived here was re-tuned.
//!
//! ## The palette's job
//!
//! Every embedding app is read at a glance, in a dark booth, by someone who
//! is also watching a stage. So the palette is built around **one question
//! per colour**: what does an operator need to be able to answer without
//! focusing?
//!
//! - **Neutrals carry structure.** A ladder of steps from the window
//!   background up to primary text. Cards sit above the window, sunken lanes
//!   (a meter track, a message well, a text field) below the card — so a
//!   panel reads as a physical object with recesses in it, at rest, using no
//!   colour at all.
//! - **Colour is reserved for state.** A window at rest is monochrome; every
//!   other saturated pixel means something.
//! - **Each semantic colour has exactly one meaning**, fixed across every
//!   app that uses this crate: [`ACCENT`] is always "this wants your
//!   attention *now*". [`WARN`] is always "degraded, but working".
//!   [`DANGER`] is always "wrong, or irreversible". [`INFO`] is always
//!   discovery/RF. [`OK`] is always "reachable/active". An app-specific
//!   identity colour (a chat sender's hashed colour, say) is the one kind of
//!   deliberate exception, and belongs in that app, not here.
//!
//! ## Extra-dark mode
//!
//! An embedding app can offer an extra-dark setting for FOH positions where
//! even a near-black window blooms. [`Options::extra_dark`] swaps the whole
//! [`Neutrals`] ladder for a darker one rather than overriding four fills ad
//! hoc, so a card still sits one step above the window and a recess still
//! reads as a recess.
//!
//! Because the ladder is chosen at [`apply`] time and not at compile time,
//! **call sites that paint a surface must read it back off `ui.visuals()`**
//! (via [`surface`], [`surface_container`], [`surface_container_high`],
//! [`outline_variant`]) rather than naming a `const` — [`apply`]'s doc has
//! the role → egui-slot table. The consts below are the *standard* ladder,
//! and stay named so the tests and the docs have something to point at.
//!
//! ## Contrast
//!
//! Text tokens are checked against [`BG_CARD`], the busiest background they
//! land on: [`TEXT`] ≈ 13:1, [`TEXT_DIM`] ≈ 6:1, [`TEXT_FAINT`] ≈ 3.4:1 — so
//! faint text is only ever used for labels that repeat on every row (a
//! timestamp, a unit suffix), never for a value that has to be read. The
//! extra-dark ladder only ever raises those ratios.

use egui::{Color32, CornerRadius, FontId, Margin, Stroke, Vec2};

// ---------------------------------------------------------------------------
// Neutrals — structure
// ---------------------------------------------------------------------------

/// The window itself: near-black, so a projector-lit booth doesn't bloom.
pub const BG_APP: Color32 = Color32::from_rgb(15, 16, 19);
/// Toolbars and side panels — one step up from [`BG_APP`].
pub const BG_CHROME: Color32 = Color32::from_rgb(23, 25, 29);
/// A card at rest: a roster row, a room entry, a message group.
pub const BG_CARD: Color32 = Color32::from_rgb(30, 33, 38);
/// A card the pointer is over, and the resting fill of a chrome control.
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 42, 48);
/// Recessed lanes *inside* a card — a text field's well, a scrollback area.
///
/// **Translucent, not opaque**, for the same reason it is in mictrace: a
/// lane has to read as cut *into* whatever is behind it, and darkening what
/// is already there does that at any point on the ladder — including under
/// extra-dark, where an opaque near-black well and an opaque near-black card
/// would be indistinguishable.
pub const BG_SUNKEN: Color32 = Color32::from_black_alpha(120);

/// Hairline between a card and the surface behind it; also a control's
/// hovered fill.
pub const STROKE_CARD: Color32 = Color32::from_rgb(52, 57, 65);
/// Separators inside chrome.
pub const STROKE_SUBTLE: Color32 = Color32::from_rgb(42, 46, 53);

/// Values you have to be able to read: message text, a sender's name, a
/// channel id.
pub const TEXT: Color32 = Color32::from_rgb(233, 236, 240);
/// Supporting text: peer metadata, toolbar labels, notices.
pub const TEXT_DIM: Color32 = Color32::from_rgb(154, 161, 172);
/// Repeated-on-every-row labels only — timestamps, source badges.
pub const TEXT_FAINT: Color32 = Color32::from_rgb(108, 115, 126);

/// The neutral ladder, as one value, so extra-dark mode is a second ladder
/// rather than four overridden fills.
///
/// Every field is a step on one scale: `app` is the furthest back,
/// `stroke_active` the furthest forward. The control steps
/// (`control_*`) continue the same scale past the card — a toolbar button at
/// rest is one step above a card, and lifts two more as the pointer arrives
/// and presses. That is the whole hover-feedback mechanism in this theme, so
/// it has to survive the extra-dark swap intact rather than being a set of
/// literals tuned for one background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Neutrals {
    /// The window. Carried to call sites as `Visuals::panel_fill`.
    pub app: Color32,
    /// Toolbars, side panels, dialog chrome. Carried as `Visuals::window_fill`.
    pub chrome: Color32,
    /// A card at rest. Carried as `Visuals::faint_bg_color`.
    pub card: Color32,
    /// A hovered card. Also a chrome control's resting fill.
    pub card_hover: Color32,
    /// Separators inside chrome. Carried as
    /// `Visuals::widgets.noninteractive.bg_stroke`.
    pub stroke_subtle: Color32,
    /// A card's boundary. Also a chrome control's hovered fill.
    pub stroke_card: Color32,
    /// A chrome control being pressed.
    pub control_active: Color32,
    /// The border on a hovered chrome control.
    pub stroke_hover: Color32,
    /// The border on a pressed chrome control.
    pub stroke_active: Color32,
}

impl Neutrals {
    /// The ladder every `const` above names — mictrace's values unchanged.
    pub const STANDARD: Self = Self {
        app: BG_APP,
        chrome: BG_CHROME,
        card: BG_CARD,
        card_hover: BG_CARD_HOVER,
        stroke_subtle: STROKE_SUBTLE,
        stroke_card: STROKE_CARD,
        control_active: Color32::from_rgb(64, 70, 80),
        stroke_hover: Color32::from_rgb(72, 78, 88),
        stroke_active: Color32::from_rgb(90, 98, 110),
    };

    /// The same ladder pulled down toward black for FOH positions, keeping
    /// the *spacing* between steps rather than clamping the bottom three to
    /// the same near-black (which is what the pre-theme
    /// `extra_dark` branch did, and why a card used to disappear into the
    /// window in that mode).
    pub const EXTRA_DARK: Self = Self {
        app: Color32::from_rgb(5, 5, 6),
        chrome: Color32::from_rgb(10, 11, 13),
        card: Color32::from_rgb(16, 17, 20),
        card_hover: Color32::from_rgb(24, 26, 30),
        stroke_subtle: Color32::from_rgb(26, 28, 32),
        stroke_card: Color32::from_rgb(36, 39, 45),
        control_active: Color32::from_rgb(48, 52, 60),
        stroke_hover: Color32::from_rgb(56, 61, 70),
        stroke_active: Color32::from_rgb(72, 78, 90),
    };

    fn for_mode(extra_dark: bool) -> Self {
        if extra_dark {
            Self::EXTRA_DARK
        } else {
            Self::STANDARD
        }
    }
}

// ---------------------------------------------------------------------------
// Reading the installed ladder back
// ---------------------------------------------------------------------------
//
// See this module's header: the ladder isn't known at compile time, so
// anything that paints a surface asks `ui.visuals()` for it. These four are
// the whole transport — the inverse of the table in [`apply`]'s doc — and
// exist so no call site has to know *which* egui slot a role happens to ride
// in. If that mapping ever changes, it changes in two places on one screen.

/// The window's own fill — MD3 `surface`.
pub fn surface(visuals: &egui::Visuals) -> Color32 {
    visuals.panel_fill
}

/// Toolbars, side panels, dialog chrome — MD3 `surface-container`.
pub fn surface_container(visuals: &egui::Visuals) -> Color32 {
    visuals.window_fill
}

/// A raised surface: cards, list rows at rest — MD3 `surface-container-high`.
pub fn surface_container_high(visuals: &egui::Visuals) -> Color32 {
    visuals.faint_bg_color
}

/// Borders that should recede — MD3 `outline-variant`.
pub fn outline_variant(visuals: &egui::Visuals) -> Color32 {
    visuals.widgets.noninteractive.bg_stroke.color
}

// ---------------------------------------------------------------------------
// Semantics — state
// ---------------------------------------------------------------------------

/// **This wants your attention now.** The solo ring in mictrace, a flash
/// pulse's tint in sidestage, a channel with unread traffic. Nothing else may
/// use this amber: "does something need me right now" is the one question an
/// operator asks mid-show that must never need a second look.
pub const ACCENT: Color32 = Color32::from_rgb(255, 146, 10);

/// Reachable / active / healthy: a good battery, a peer seen just now, a
/// healthy bind.
pub const OK: Color32 = Color32::from_rgb(76, 200, 122);
/// Approaching a limit, or degraded but still working: a low battery, a
/// peer that has gone quiet.
///
/// A clear yellow, where [`ACCENT`] is a clear orange — they started life as
/// near-identical ambers and
/// [`armed_amber_is_distinct_from_warning_amber`](tests::armed_amber_is_distinct_from_warning_amber)
/// is what caught it: two affordances this different (e.g. "you are
/// listening to this" and "this transmitter is about to die") must never
/// read as the same colour.
pub const WARN: Color32 = Color32::from_rgb(232, 202, 64);
/// Wrong *now*, or irreversible: a clip, a peak alert, a broadcast clear, a
/// rejected name.
pub const DANGER: Color32 = Color32::from_rgb(232, 76, 76);
/// Discovery, and only discovery: RF, rooms and peers found on the network,
/// hyperlinks.
pub const INFO: Color32 = Color32::from_rgb(94, 172, 240);

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

/// "Hit targets ≥ 32 px" — techs wear gloves, use touchscreens, and hit
/// controls in the dark mid-cue.
pub const HIT_TARGET: f32 = 32.0;
pub const RADIUS_CARD: u8 = 6;
pub const RADIUS_LANE: u8 = 2;
/// Deliberately the same step as [`SHAPE_BUTTON`]: stock egui controls and
/// [`crate::md3`] controls appear side by side in the same panels, and two
/// radii four pixels apart read as a mistake rather than a distinction.
pub const RADIUS_CONTROL: u8 = SHAPE_BUTTON;

// ---------------------------------------------------------------------------
// Type
// ---------------------------------------------------------------------------

/// DejaVu Sans Mono, vendored rather than looked up by name so macOS,
/// Windows and Linux all lay out identically — and so every embedding app
/// gets the same face this theme was designed against without having to ship
/// one itself.
///
/// Licensed under the Bitstream Vera / DejaVu terms (see
/// `assets/fonts/LICENSE-DejaVu.txt`), which permit embedding and
/// redistribution.
const DEJAVU_SANS_MONO: &[u8] = include_bytes!("../assets/fonts/DejaVuSansMono.ttf");

/// Font family name registered with egui.
const FAMILY: &str = "DejaVuSansMono";

/// Installs [`DEJAVU_SANS_MONO`] as the face for *both* of egui's families.
///
/// Every embedding app is monospaced throughout: labels and ids line up
/// column-to-column down a grid, a roster or a channel list, and a value
/// that changes every frame (a dB readout, a timestamp) holds still instead
/// of jittering as its digits change width.
///
/// DejaVu is *prepended* to each family rather than replacing it. Some of
/// what these apps render is user-supplied text (an actor's name, a chat
/// message) and can be in any script at all, so leaving egui's built-ins
/// behind it means an unusual string renders as glyphs instead of tofu.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        FAMILY.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(DEJAVU_SANS_MONO)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, FAMILY.to_owned());
    }
    ctx.set_fonts(fonts);
}

/// Text whose columns have to line up: timestamps, ports, peer addresses.
///
/// Both families resolve to DejaVu Sans Mono today (see [`install_fonts`]),
/// so this and [`text`] currently render alike — they stay separate because
/// the *reason* differs: this one is a layout constraint and must not follow
/// if `text` ever moves to a proportional face.
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// Names, labels and message bodies.
pub fn text(size: f32) -> FontId {
    FontId::proportional(size)
}

// ---------------------------------------------------------------------------
// MD3 — role names over the same palette
// ---------------------------------------------------------------------------
//
// Everything below is a *layer*, not a second palette: every MD3 colour role
// is either an alias of a constant above, or a value derived from one via
// [`mix`]. Nothing here is a new saturated hue.

/// Blends `a` toward `b` by `percent` (0..=100), integer math so it stays a
/// `const fn` — MD3's tonal "container" roles and its state-layer overlays
/// are both "one colour blended toward another by a fixed amount", so one
/// helper derives both instead of two hand-picked sets of RGB literals.
const fn mix_channel(a: u8, b: u8, percent: u32) -> u8 {
    ((a as u32 * (100 - percent) + b as u32 * percent) / 100) as u8
}

/// See [`mix_channel`].
pub const fn mix(a: Color32, b: Color32, percent: u32) -> Color32 {
    Color32::from_rgb(
        mix_channel(a.r(), b.r(), percent),
        mix_channel(a.g(), b.g(), percent),
        mix_channel(a.b(), b.b(), percent),
    )
}

/// Text/icons on any `surface*` role. Alias of [`TEXT`].
pub const ON_SURFACE: Color32 = TEXT;
/// Supporting text/icons on any `surface*` role. Alias of [`TEXT_DIM`].
pub const ON_SURFACE_VARIANT: Color32 = TEXT_DIM;
/// Borders that need to read as structure. Alias of [`STROKE_CARD`].
///
/// Unlike the surface roles this is *not* swapped by extra-dark mode: it is
/// a boundary drawn on top of a surface rather than the surface itself, and
/// a border that dimmed along with its background would stop being a border.
pub const OUTLINE: Color32 = STROKE_CARD;

/// The one call-to-action colour **in chrome**.
///
/// Deliberately *not* [`ACCENT`]. [`ACCENT`] means "something wants you now"
/// and nothing else, which is the rule the whole palette is built on. Paint
/// a Settings "Apply" button the same amber as a flash and that colour is on
/// screen meaning something else.
///
/// So chrome gets its own primary: a dark, recessive brown-amber, related to
/// [`ACCENT`] by hue but nowhere near it in luminance. Low separation from
/// the surfaces behind it (≈1.3:1 against [`BG_CARD`]) is the intent, not an
/// oversight — what makes the button findable is its [`ON_PRIMARY`] label at
/// ≈10:1, not the fill shouting.
pub const PRIMARY: Color32 = Color32::from_rgb(0x50, 0x2E, 0x0E);
/// Label/icon colour *on* a [`PRIMARY`]-filled surface: near-white, ≈10:1.
pub const ON_PRIMARY: Color32 = ON_SURFACE;
/// A muted, low-emphasis tint of [`PRIMARY`] for tonal surfaces (a Filled
/// Tonal button's fill, a selected nav entry). A tonal container has to be
/// *lighter* than its surface in a dark theme, so this is the primary hue at
/// roughly tone 30 rather than a blend of the surface toward [`PRIMARY`] —
/// which, with a primary this dark, would land within 1.02:1 of the surface.
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(0x6B, 0x40, 0x18);
/// Label/icon colour on [`PRIMARY_CONTAINER`]. Alias of [`ON_SURFACE`] —
/// deliberately *not* [`PRIMARY`]/[`ACCENT`] itself, so a tonal button's
/// text never paints the "wants you now" amber.
pub const ON_PRIMARY_CONTAINER: Color32 = ON_SURFACE;

/// Alias of [`DANGER`]. Used directly rather than through a filled surface
/// by [`crate::md3::danger_button`], which is outlined: its label and border
/// are both this red.
pub const ERROR: Color32 = DANGER;
/// Label/icon colour on an [`ERROR`]-filled surface — [`DANGER`] fails AA
/// against white text (3.2:1) but clears it against the darkest neutral
/// (5.0:1). Kept so the role set is complete even where no app fills a
/// surface with [`ERROR`] today.
pub const ON_ERROR: Color32 = BG_APP;

// ---------------------------------------------------------------------------
// MD3 — type scale
// ---------------------------------------------------------------------------
//
// Sizes follow MD3's default scale (in sp, treated 1:1 as egui points),
// rendered in DejaVu Sans Mono rather than the Roboto MD3 assumes — see
// [`install_fonts`]. [`apply`] wires the five roles egui's `TextStyle` enum
// has a slot for (Heading/Body/Button/Small/Monospace), so most call sites
// pick the scale up for free; these exist for the roles egui has no slot for
// and for anything that wants an explicit size.

pub const DISPLAY_LARGE_SIZE: f32 = 36.0;
pub const DISPLAY_MEDIUM_SIZE: f32 = 32.0;
pub const DISPLAY_SMALL_SIZE: f32 = 28.0;
pub const HEADLINE_LARGE_SIZE: f32 = 26.0;
pub const HEADLINE_MEDIUM_SIZE: f32 = 24.0;
pub const HEADLINE_SMALL_SIZE: f32 = 22.0;
/// MD3 Title Large — also `TextStyle::Heading`'s size, see [`apply`].
pub const TITLE_LARGE_SIZE: f32 = 22.0;
/// MD3 Title Medium — used by [`crate::md3::section_heading`].
pub const TITLE_MEDIUM_SIZE: f32 = 16.0;
pub const TITLE_SMALL_SIZE: f32 = 14.0;
pub const BODY_LARGE_SIZE: f32 = 16.0;
/// MD3 Body Medium — also `TextStyle::Body`/`TextStyle::Monospace`'s size.
pub const BODY_MEDIUM_SIZE: f32 = 14.0;
pub const BODY_SMALL_SIZE: f32 = 12.0;
/// MD3 Label Large — also `TextStyle::Button`'s size.
pub const LABEL_LARGE_SIZE: f32 = 14.0;
pub const LABEL_MEDIUM_SIZE: f32 = 12.0;
/// MD3 Label Small — also `TextStyle::Small`'s size.
pub const LABEL_SMALL_SIZE: f32 = 11.0;

pub fn display_large() -> FontId {
    text(DISPLAY_LARGE_SIZE)
}
pub fn display_medium() -> FontId {
    text(DISPLAY_MEDIUM_SIZE)
}
pub fn display_small() -> FontId {
    text(DISPLAY_SMALL_SIZE)
}
pub fn headline_large() -> FontId {
    text(HEADLINE_LARGE_SIZE)
}
pub fn headline_medium() -> FontId {
    text(HEADLINE_MEDIUM_SIZE)
}
pub fn headline_small() -> FontId {
    text(HEADLINE_SMALL_SIZE)
}
pub fn title_large() -> FontId {
    text(TITLE_LARGE_SIZE)
}
pub fn title_medium() -> FontId {
    text(TITLE_MEDIUM_SIZE)
}
pub fn title_small() -> FontId {
    text(TITLE_SMALL_SIZE)
}
pub fn body_large() -> FontId {
    text(BODY_LARGE_SIZE)
}
pub fn body_medium() -> FontId {
    text(BODY_MEDIUM_SIZE)
}
pub fn body_small() -> FontId {
    text(BODY_SMALL_SIZE)
}
pub fn label_large() -> FontId {
    text(LABEL_LARGE_SIZE)
}
pub fn label_medium() -> FontId {
    text(LABEL_MEDIUM_SIZE)
}
pub fn label_small() -> FontId {
    text(LABEL_SMALL_SIZE)
}

// ---------------------------------------------------------------------------
// MD3 — shape scale
// ---------------------------------------------------------------------------

pub const SHAPE_NONE: u8 = 0;
pub const SHAPE_EXTRA_SMALL: u8 = 4;
pub const SHAPE_SMALL: u8 = 8;
pub const SHAPE_MEDIUM: u8 = 12;
pub const SHAPE_LARGE: u8 = 16;
pub const SHAPE_EXTRA_LARGE: u8 = 28;
/// Fully rounded ("stadium"/pill). `CornerRadius` is a `u8`; egui clamps a
/// corner radius to whatever the rect's shorter side allows, so a value this
/// large always resolves to "as round as the shape permits".
pub const SHAPE_FULL: u8 = u8::MAX;

/// The corner radius every [`crate::md3`] button variant uses.
///
/// **This deliberately diverges from MD3, which specifies [`SHAPE_FULL`]
/// (stadium/pill) as a button's default shape — do not "correct" it back.**
/// Every embedding app's shape language is set by its own grid geometry
/// (cards at [`RADIUS_CARD`], lanes at [`RADIUS_LANE`]); a panel with pill
/// buttons docked into one of these apps would read as borrowed from
/// somewhere else. MD3 treats the shape scale as a brand-tunable token set,
/// so this is a supported move rather than a violation.
pub const SHAPE_BUTTON: u8 = SHAPE_EXTRA_SMALL;

// ---------------------------------------------------------------------------
// MD3 — elevation
// ---------------------------------------------------------------------------
//
// Levels 0-5 at MD3's reference dp (0, 1, 3, 6, 8, 12), as `egui::Shadow`.
// Kept subtle relative to MD3's own defaults: those assume a *light* surface
// where a shadow is the only way to separate two similarly-toned panes, but
// this theme already does that with the neutral ladder, so these exist for
// the handful of surfaces that genuinely float over other chrome.

pub const ELEVATION_0: egui::Shadow = egui::Shadow {
    offset: [0, 0],
    blur: 0,
    spread: 0,
    color: Color32::TRANSPARENT,
};
pub const ELEVATION_1: egui::Shadow = egui::Shadow {
    offset: [0, 1],
    blur: 3,
    spread: 0,
    color: Color32::from_black_alpha(60),
};
pub const ELEVATION_2: egui::Shadow = egui::Shadow {
    offset: [0, 1],
    blur: 5,
    spread: 0,
    color: Color32::from_black_alpha(70),
};
pub const ELEVATION_3: egui::Shadow = egui::Shadow {
    offset: [0, 2],
    blur: 8,
    spread: 0,
    color: Color32::from_black_alpha(80),
};
pub const ELEVATION_4: egui::Shadow = egui::Shadow {
    offset: [0, 2],
    blur: 10,
    spread: 0,
    color: Color32::from_black_alpha(90),
};
pub const ELEVATION_5: egui::Shadow = egui::Shadow {
    offset: [0, 3],
    blur: 14,
    spread: 0,
    color: Color32::from_black_alpha(100),
};

// ---------------------------------------------------------------------------
// MD3 — state layers
// ---------------------------------------------------------------------------
//
// MD3's reference opacities for interaction feedback on a *role-coloured*
// surface (a Filled button, a tonal container — anything whose fill isn't
// already one of the neutral ladder's steps). This is a second vocabulary
// alongside [`apply`]'s `widgets.{inactive,hovered,active}` block, not a
// competing implementation of it: that block is the state-layer mechanism
// for *neutral* controls (a toolbar button stepping up the ladder on hover).
// These opacities are what [`crate::md3`]'s role-coloured variants use
// instead, via [`mix`] against the role's `on_*` colour — mixing toward
// white/black would ignore the surface's own hue.

pub const STATE_HOVER: u32 = 8;
pub const STATE_FOCUS: u32 = 10;
pub const STATE_PRESSED: u32 = 10;
pub const STATE_DRAGGED: u32 = 16;
/// Plugged into egui's own `Visuals::disabled_alpha`, so disabled content in
/// this app really is 38% opacity rather than a token that merely says so.
pub const STATE_DISABLED_CONTENT: f32 = 0.38;
/// Unused: `disabled_alpha` fades a disabled widget's content and container
/// uniformly, so [`STATE_DISABLED_CONTENT`] — the more visible of the two,
/// and the safer for legibility — is what actually applies to both.
pub const STATE_DISABLED_CONTAINER: f32 = 0.12;

// ---------------------------------------------------------------------------
// Style installation
// ---------------------------------------------------------------------------

/// What the caller gets to choose about the theme. Everything else is fixed
/// — this is one designed palette, not a themable surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    /// Swap [`Neutrals::STANDARD`] for [`Neutrals::EXTRA_DARK`].
    pub extra_dark: bool,
    /// `egui::Context::set_zoom_factor` — the whole-UI scale a "large text"
    /// toggle or a font-size slider drives. Scaling the context rather than
    /// overriding per-widget font sizes is what keeps spacing, hit targets
    /// and the type scale in proportion with each other.
    pub zoom: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            extra_dark: false,
            zoom: 1.0,
        }
    }
}

/// Installs the booth theme onto `ctx`.
///
/// Call once at startup and again whenever [`Options`] changes — not every
/// frame. It is idempotent, but it rebuilds `FontDefinitions` and both
/// `Style`s each time, which is wasted work at 60 Hz.
///
/// ## How a role reaches a call site
///
/// Most widgets never need to know: they're stock egui widgets reading
/// `ui.visuals()`, which this function has already loaded with the palette.
/// Anything that paints its own surface needs the *installed* ladder rather
/// than a `const` (extra-dark mode swaps it), and gets it from these slots:
///
/// | role | egui slot | accessor |
/// |---|---|---|
/// | `surface` | `Visuals::panel_fill` | [`surface`] |
/// | `surface-container` | `Visuals::window_fill` | [`surface_container`] |
/// | `surface-container-high` | `Visuals::faint_bg_color` | [`surface_container_high`] |
/// | `outline-variant` | `Visuals::widgets.noninteractive.bg_stroke` | [`outline_variant`] |
///
/// The sunken well rides in `Visuals::extreme_bg_color`, which is already
/// where egui looks for a `TextEdit`'s background, so it needs no accessor.
pub fn apply(ctx: &egui::Context, options: Options) {
    install_fonts(ctx);

    // This is a booth app for a dark room — it never follows the OS theme
    // (`ThemePreference::System` is egui's default) and there are no
    // light-mode colours defined anywhere in this module, so switching would
    // just break.
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.set_zoom_factor(options.zoom);

    let n = Neutrals::for_mode(options.extra_dark);
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = n.app;
    visuals.window_fill = n.chrome;
    visuals.extreme_bg_color = BG_SUNKEN;
    visuals.faint_bg_color = n.card;
    visuals.window_stroke = Stroke::new(1.0, n.stroke_subtle);
    visuals.window_corner_radius = CornerRadius::same(RADIUS_CARD);

    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = INFO;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.warn_fg_color = WARN;
    visuals.error_fg_color = DANGER;
    visuals.disabled_alpha = STATE_DISABLED_CONTENT;
    // MD3-ish sliders: a filled leading track and a round handle, closer to
    // the spec's slider than egui's default rectangular handle with no fill.
    // Global rather than per-slider since nothing here wants the old look.
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;

    // Controls: flat and low-contrast at rest, lifting on hover. A toolbar
    // of equally-bright buttons is what makes the one button that matters
    // impossible to find.
    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = n.chrome;
    w.noninteractive.weak_bg_fill = n.chrome;
    w.noninteractive.bg_stroke = Stroke::new(1.0, n.stroke_subtle);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    w.noninteractive.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    w.inactive.bg_fill = n.card_hover;
    w.inactive.weak_bg_fill = n.card_hover;
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    w.inactive.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    w.hovered.bg_fill = n.stroke_card;
    w.hovered.weak_bg_fill = n.stroke_card;
    w.hovered.bg_stroke = Stroke::new(1.0, n.stroke_hover);
    w.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    w.hovered.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    w.active.bg_fill = n.control_active;
    w.active.weak_bg_fill = n.control_active;
    w.active.bg_stroke = Stroke::new(1.0, n.stroke_active);
    w.active.fg_stroke = Stroke::new(1.0, TEXT);
    w.active.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    w.open.bg_fill = n.stroke_card;
    w.open.weak_bg_fill = n.stroke_card;
    w.open.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    // Written into the Dark slot explicitly (not `set_visuals`, which writes
    // to whichever slot `ctx.theme()` currently reports) so this survives
    // even if something upstream reads `theme()` before the `set_theme` call
    // above has taken effect for the current pass.
    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    ctx.all_styles_mut(|style| {
        // `interact_size` is egui's floor for anything clickable. MD3's own
        // 48dp minimum touch target exceeds it — `md3.rs`'s constructors ask
        // for that explicitly per-widget rather than raising the floor here,
        // since 48px is too tall for a dense settings row.
        style.spacing.interact_size = Vec2::new(48.0, HIT_TARGET);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.window_margin = Margin::same(12);
        style.spacing.menu_margin = Margin::same(8);

        // Nearly no call site sets an explicit `FontId`, so wiring egui's
        // five built-in `TextStyle` roles to their MD3 equivalents migrates
        // the whole app's typography in one edit.
        style
            .text_styles
            .insert(egui::TextStyle::Heading, title_large());
        style
            .text_styles
            .insert(egui::TextStyle::Body, body_medium());
        style
            .text_styles
            .insert(egui::TextStyle::Button, label_large());
        style
            .text_styles
            .insert(egui::TextStyle::Small, label_small());
        style
            .text_styles
            .insert(egui::TextStyle::Monospace, mono(BODY_MEDIUM_SIZE)); // MD3 Body Medium
    });
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Relative luminance per WCAG 2.x.
    fn luminance(c: Color32) -> f64 {
        let ch = |v: u8| {
            let s = f64::from(v) / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
    }

    /// Shared with `md3.rs`'s and `colors.rs`'s tests, which check a label
    /// against the fill it actually renders on.
    pub(crate) fn contrast(a: Color32, b: Color32) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        let (hi, lo) = if x > y { (x, y) } else { (y, x) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// The two text tokens a *value* is ever rendered in have to survive a
    /// dim booth. [`TEXT_FAINT`] is deliberately exempt — it only ever
    /// labels, never reports.
    #[test]
    fn readable_text_clears_wcag_aa_on_a_card() {
        for (mode, n) in [
            ("standard", Neutrals::STANDARD),
            ("extra dark", Neutrals::EXTRA_DARK),
        ] {
            assert!(
                contrast(TEXT, n.card) >= 7.0,
                "{mode}: primary text {:.1}:1",
                contrast(TEXT, n.card)
            );
            assert!(
                contrast(TEXT_DIM, n.card) >= 4.5,
                "{mode}: dim text {:.1}:1",
                contrast(TEXT_DIM, n.card)
            );
        }
    }

    /// Every state colour has to be distinguishable from the card it sits
    /// on — a 3:1 floor is WCAG's own bar for non-text graphics.
    #[test]
    fn state_colors_clear_the_non_text_contrast_floor() {
        for (name, color) in [
            ("ok", OK),
            ("warn", WARN),
            ("danger", DANGER),
            ("info", INFO),
            ("accent", ACCENT),
        ] {
            for (mode, n) in [
                ("standard", Neutrals::STANDARD),
                ("extra dark", Neutrals::EXTRA_DARK),
            ] {
                let ratio = contrast(color, n.card);
                assert!(ratio >= 3.0, "{mode}: {name} vs card is only {ratio:.2}:1");
            }
        }
    }

    /// "Someone is flashing you" and "the network is degraded" must never be
    /// confusable, which is the whole reason [`WARN`] is not [`ACCENT`]
    /// reused.
    #[test]
    fn armed_amber_is_distinct_from_warning_amber() {
        let d = (i32::from(ACCENT.r()) - i32::from(WARN.r())).abs()
            + (i32::from(ACCENT.g()) - i32::from(WARN.g())).abs()
            + (i32::from(ACCENT.b()) - i32::from(WARN.b())).abs();
        assert!(d >= 60, "accent and warn differ by only {d}");
    }

    /// A button's label has to clear WCAG AA against the fill it lands on —
    /// the check that flipped [`ON_PRIMARY`] to near-white when [`PRIMARY`]
    /// stopped being [`ACCENT`], and that keeps [`ON_ERROR`] dark.
    #[test]
    fn on_primary_and_on_error_clear_the_label_floor() {
        assert!(
            contrast(ON_PRIMARY, PRIMARY) >= 4.5,
            "on-primary {:.1}:1",
            contrast(ON_PRIMARY, PRIMARY)
        );
        assert!(
            contrast(ON_PRIMARY_CONTAINER, PRIMARY_CONTAINER) >= 4.5,
            "on-primary-container {:.1}:1",
            contrast(ON_PRIMARY_CONTAINER, PRIMARY_CONTAINER)
        );
        assert!(
            contrast(ON_ERROR, ERROR) >= 4.5,
            "on-error {:.1}:1",
            contrast(ON_ERROR, ERROR)
        );
    }

    /// Extra-dark mode is a *ladder*, not a flattening: the whole point of
    /// the original setting was a darker window, and the bug in the version
    /// it replaces was that the window, the panels and a card all collapsed
    /// onto the same near-black. Each step has to stay visibly above the one
    /// behind it, in both ladders.
    #[test]
    fn every_ladder_step_stays_above_the_one_behind_it() {
        for (mode, n) in [
            ("standard", Neutrals::STANDARD),
            ("extra dark", Neutrals::EXTRA_DARK),
        ] {
            let steps = [
                ("app", n.app),
                ("chrome", n.chrome),
                ("card", n.card),
                ("card_hover", n.card_hover),
                ("stroke_card", n.stroke_card),
                ("control_active", n.control_active),
                ("stroke_hover", n.stroke_hover),
                ("stroke_active", n.stroke_active),
            ];
            for pair in steps.windows(2) {
                let ((below, a), (above, b)) = (pair[0], pair[1]);
                assert!(
                    luminance(b) > luminance(a),
                    "{mode}: {above} is not brighter than {below}"
                );
            }
        }
    }

    /// The transport table in [`apply`]'s doc is the contract every painter
    /// in `md3.rs` reads through. If a slot is repointed, this fails rather
    /// than a card quietly rendering in the window's colour.
    #[test]
    fn apply_publishes_the_ladder_through_the_documented_slots() {
        for (extra_dark, n) in [(false, Neutrals::STANDARD), (true, Neutrals::EXTRA_DARK)] {
            let ctx = egui::Context::default();
            apply(
                &ctx,
                Options {
                    extra_dark,
                    ..Default::default()
                },
            );
            let visuals = ctx.style_of(egui::Theme::Dark).visuals.clone();
            assert_eq!(surface(&visuals), n.app);
            assert_eq!(surface_container(&visuals), n.chrome);
            assert_eq!(surface_container_high(&visuals), n.card);
            assert_eq!(outline_variant(&visuals), n.stroke_subtle);
        }
    }

    /// The zoom factor a "large text" toggle and a font-size slider both
    /// drive is the *only* size knob — nothing may reach past it and set
    /// per-widget font sizes, which would rescale text without rescaling the
    /// spacing and hit targets around it.
    ///
    /// A pass has to run first: `Context::set_zoom_factor` queues the change
    /// and applies it at the *start of the next pass*, so reading
    /// `zoom_factor()` straight after [`apply`] still reports the old value.
    #[test]
    fn zoom_is_installed_on_the_context() {
        let ctx = egui::Context::default();
        apply(
            &ctx,
            Options {
                zoom: 1.4,
                ..Default::default()
            },
        );
        let _ = ctx.run_ui(egui::RawInput::default(), |_| {});
        assert!((ctx.zoom_factor() - 1.4).abs() < f32::EPSILON);
    }

    /// The inverse of [`armed_amber_is_distinct_from_warning_amber`], and the
    /// more important half: chrome's [`PRIMARY`] must **not** be [`ACCENT`].
    /// Aliasing them puts the "this wants your attention now" amber on an
    /// ordinary Settings button. A future tidy-up that notices the two are
    /// both amber-ish and folds them back together has to delete this test
    /// to do it.
    #[test]
    fn chrome_primary_is_not_the_accent_amber() {
        assert_ne!(PRIMARY, ACCENT);
        assert!(
            contrast(PRIMARY, ACCENT) >= 3.0,
            "chrome primary and accent amber are only {:.2}:1 apart",
            contrast(PRIMARY, ACCENT)
        );
    }

    /// [`ERROR`] is a *bright* fill, so the white-text default is exactly the
    /// footgun [`ON_ERROR`] exists to avoid — pin down that [`ON_SURFACE`]
    /// (near-white) specifically fails on it, so nobody "simplifies" this
    /// crate by reaching for it later. [`PRIMARY`] doesn't need the same
    /// check: it's a dark fill, so near-white (what [`ON_PRIMARY`] resolves
    /// to) is the *correct* label on it.
    #[test]
    fn on_surface_is_not_a_safe_label_for_an_error_fill() {
        assert!(contrast(ON_SURFACE, ERROR) < 4.5);
    }

    /// The `*_container` roles have to still read as "part of the neutral
    /// chrome, just tinted" — not a saturated colour in their own right —
    /// which the non-text 3:1 floor against their parent surface confirms
    /// without requiring them to pass as body text.
    #[test]
    fn container_roles_clear_the_non_text_floor_against_surface() {
        let ratio = contrast(PRIMARY_CONTAINER, Neutrals::STANDARD.card_hover);
        assert!(
            ratio >= 1.2,
            "primary_container vs card_hover is only {ratio:.2}:1 — too close to read as a distinct tint"
        );
    }

    /// [`mix`] is the one primitive both `*_container` derivation and
    /// `md3.rs`'s state-layer overlays are built on — pin its two endpoints
    /// and the direction of travel down directly.
    #[test]
    fn mix_interpolates_from_a_to_b() {
        let a = Color32::from_rgb(0, 0, 0);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(mix(a, b, 0), a);
        assert_eq!(mix(a, b, 100), b);
        let half = mix(a, b, 50);
        assert_eq!(half, Color32::from_rgb(100, 50, 25));
    }

    /// The shape scale has to actually be a scale — each step strictly
    /// larger than the last, `FULL` the largest of all.
    #[test]
    fn shape_scale_is_monotonic() {
        let steps = [
            SHAPE_NONE,
            SHAPE_EXTRA_SMALL,
            SHAPE_SMALL,
            SHAPE_MEDIUM,
            SHAPE_LARGE,
            SHAPE_EXTRA_LARGE,
        ];
        for pair in steps.windows(2) {
            assert!(pair[0] < pair[1], "shape scale not monotonic: {steps:?}");
        }
        assert_eq!(SHAPE_FULL, u8::MAX);
    }

    /// Same for elevation — a higher level has to look more raised (bigger
    /// blur, more opaque shadow), or the scale doesn't mean anything.
    #[test]
    fn elevation_scale_is_monotonic() {
        let levels = [
            ELEVATION_0,
            ELEVATION_1,
            ELEVATION_2,
            ELEVATION_3,
            ELEVATION_4,
            ELEVATION_5,
        ];
        for pair in levels.windows(2) {
            assert!(
                pair[0].blur <= pair[1].blur && pair[0].color.a() <= pair[1].color.a(),
                "elevation scale not monotonic between {:?} and {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    /// `RADIUS_CONTROL` sits on the shape scale — pin it to [`SHAPE_BUTTON`]
    /// specifically (not just "whatever `SHAPE_BUTTON`` happens to equal
    /// today") so stock egui controls and `md3.rs` buttons are provably the
    /// same step, not two constants that happened to match by coincidence.
    #[test]
    fn radius_control_matches_the_button_shape() {
        assert_eq!(RADIUS_CONTROL, SHAPE_BUTTON);
        assert_eq!(RADIUS_CONTROL, SHAPE_EXTRA_SMALL);
    }
}
