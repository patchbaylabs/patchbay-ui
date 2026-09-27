//! Material Design 3 widget constructors — thin, pre-styled wrappers around
//! **stock egui widgets**. No custom painting, no new widget types:
//! everything here is an `egui::Button`/`Checkbox`/`ComboBox`/`Slider`/
//! `Frame` underneath, just built with the role colours and shape scale
//! [`crate::theme`] defines. Shared by mictrace, sidestage and rcpmix, so a
//! panel from one embedded in another is built from the same controls.
//!
//! Where a component needs painting egui's stock widgets can't do (a real
//! track-and-thumb switch, a drop shadow on a button), that's called out in
//! its doc comment rather than worked around — see [`switch`] and
//! [`elevated_button`].
//!
//! ## The stroke *and* fill trap
//!
//! An MD3 outlined button can't be built with `Button::stroke()`: that
//! overrides the frame's stroke *after* egui has computed the frame's inner
//! margin from the `Style`'s per-state stroke width, so the two disagree and
//! the button's footprint becomes state-dependent — it jitters as the
//! pointer enters it.
//!
//! `Button::fill()` doesn't have that specific footprint bug (fill doesn't
//! participate in the margin calculation) but egui's own doc on both methods
//! carries the same warning: *"Note that this will override any on-hover
//! effects."* A `Button::fill(PRIMARY)` filled button renders the identical
//! colour resting, hovered and pressed, because the override replaces the
//! frame's fill unconditionally instead of per-state — a real regression
//! from how every other control here behaves, since `theme::apply`'s
//! `widgets.{inactive,hovered,active}` block exists specifically so an
//! operator can tell where the pointer is.
//!
//! So every coloured variant below uses one fix for both cases:
//! [`variant_button`] scopes a `Style` mutation over
//! `ui.style_mut().visuals.widgets.{inactive,hovered,active}` and adds a
//! plain `egui::Button` with no `.fill()`/`.stroke()` override at all. The
//! per-state colours it sets *are* what the frame's fill and stroke resolve
//! to, so hover/press feedback survives, and an outline's width is one
//! constant across every state (0 for filled/tonal/elevated/text, 1 for
//! outlined) so the footprint never moves.
//!
//! ## Surfaces come from `ui`, not from a `const`
//!
//! An embedding app can offer an extra-dark mode, which swaps the whole
//! neutral ladder (see [`crate::theme::Neutrals`]). So anything below that
//! needs a neutral
//! surface reads it off `ui.visuals()` through `theme`'s accessors rather
//! than naming `theme::BG_CHROME` — otherwise a hover tint would be mixed
//! against a background that isn't on screen. Role colours that aren't part
//! of the ladder ([`crate::theme::PRIMARY`], [`crate::theme::ERROR`],
//! [`crate::theme::OUTLINE`]) are the same in both modes and stay consts.

use crate::theme;
use egui::{Color32, Frame, RichText, Stroke, Vec2};

/// The floor every button in this module is laid out against: 48px wide,
/// [`theme::HIT_TARGET`] (32px) tall — i.e. exactly `theme::apply`'s app-wide
/// `style.spacing.interact_size`.
///
/// **This deliberately diverges from MD3's 48dp minimum touch target.** A
/// square 48 makes an MD3 button visibly taller than the stock egui control
/// beside it in the same settings row, which is the opposite of what this
/// module is for: consolidating controls onto *one* size. 32px remains the
/// normative floor (see [`theme::HIT_TARGET`]: "hit targets ≥ 32 px — techs
/// wear gloves, use touchscreens"), and matching `interact_size` means stock
/// and MD3 controls clear it identically.
const MIN_TOUCH_TARGET: Vec2 = Vec2::new(48.0, theme::HIT_TARGET);

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------
//
// Five MD3 emphasis levels, all built on [`variant_button`]. Convention:
// **at most one [`filled_button`] per view** — it marks the one recommended
// action, the same "don't make everything shout" reasoning `theme.rs`
// gives for reserving colour for state.

/// Applies `fill`/`label_color` per interactive state via a scoped `Style`
/// mutation (see this module's header for why), adds a plain `egui::Button`,
/// and returns its `Response`. `outline` is `Some((width, colour))` for a
/// constant-width border across every state — never varied by state, which
/// is the part that keeps the footprint fixed.
///
/// Public alongside the five named variants because a state-driven toggle —
/// a toolbar button whose colour communicates on/off rather than one of
/// MD3's emphasis levels — has no business being a sixth named variant with
/// one caller in some app. Same safety property either way: no call site
/// anywhere should reach for `Button::fill()`/`::stroke()` directly.
pub fn variant_button(
    ui: &mut egui::Ui,
    label: impl Into<String>,
    fill: [Color32; 3],
    outline: Option<(f32, Color32)>,
    label_color: Color32,
) -> egui::Response {
    variant_button_sized(ui, label, fill, outline, label_color, MIN_TOUCH_TARGET)
}

/// [`variant_button`] with the touch-target floor exposed instead of fixed
/// at [`MIN_TOUCH_TARGET`] — the preset rail's buttons are deliberately much
/// larger than the floor (they get hit in the dark, mid-cue), and a nav
/// entry fills its column's width. Same scoped-`Style` mechanism either way,
/// so this is a size parameter rather than a second copy of the body.
pub fn variant_button_sized(
    ui: &mut egui::Ui,
    label: impl Into<String>,
    fill: [Color32; 3],
    outline: Option<(f32, Color32)>,
    label_color: Color32,
    min_size: Vec2,
) -> egui::Response {
    let label = label.into();
    ui.scope(|ui| {
        let stroke = outline.map_or(Stroke::NONE, |(w, c)| Stroke::new(w, c));
        let w = &mut ui.style_mut().visuals.widgets;
        for (visuals, colour) in [
            (&mut w.inactive, fill[0]),
            (&mut w.hovered, fill[1]),
            (&mut w.active, fill[2]),
        ] {
            visuals.weak_bg_fill = colour;
            visuals.bg_fill = colour;
            visuals.bg_stroke = stroke;
            visuals.fg_stroke = Stroke::new(1.0, label_color);
            visuals.corner_radius = egui::CornerRadius::same(theme::SHAPE_BUTTON);
        }
        ui.add(
            egui::Button::new(RichText::new(label).color(label_color))
                .corner_radius(theme::SHAPE_BUTTON)
                .min_size(min_size),
        )
    })
    .inner
}

/// The three per-state fills for a role-coloured surface: the role itself at
/// rest, then blended toward its `on_*` colour by MD3's hover and pressed
/// state-layer opacities. Every variant below is one of these plus an
/// outline choice, which is what keeps their hover feedback identical.
///
/// `pub`, alongside [`transparent_state_layers`] and [`variant_button`], so
/// an app that needs a sixth emphasis level keyed to its own role colour (a
/// destructive action in a role other than [`theme::ERROR`], say) can build
/// one without hand-rolling this blend.
pub fn state_layers(base: Color32, on: Color32) -> [Color32; 3] {
    [
        base,
        theme::mix(base, on, theme::STATE_HOVER),
        theme::mix(base, on, theme::STATE_PRESSED),
    ]
}

/// The per-state fills for a control with **no fill at rest** — outlined and
/// text buttons — which tint the chrome behind them toward `on` on hover
/// instead. Reads the chrome colour off `ui` so extra-dark mode tints the
/// background that's actually on screen.
pub fn transparent_state_layers(ui: &egui::Ui, on: Color32) -> [Color32; 3] {
    let chrome = theme::surface_container(ui.visuals());
    [
        Color32::TRANSPARENT,
        theme::mix(chrome, on, theme::STATE_HOVER),
        theme::mix(chrome, on, theme::STATE_PRESSED),
    ]
}

/// The one recommended action in the current view. Solid
/// [`theme::PRIMARY`] fill (the dark chrome brown-amber) with a near-white
/// [`theme::ON_PRIMARY`] label at ≈10:1 — see `theme::PRIMARY`'s doc for why
/// chrome's primary is deliberately dark and recessive rather than
/// [`theme::ACCENT`].
pub fn filled_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = state_layers(theme::PRIMARY, theme::ON_PRIMARY);
    variant_button(ui, label, fill, None, theme::ON_PRIMARY)
}

/// A secondary action with more emphasis than [`outlined_button`]/
/// [`text_button`] but less than [`filled_button`] — a muted tint of
/// [`theme::PRIMARY`] rather than the full-strength colour. For a view with
/// two competing committed actions ("Save"/"Save As"), where a second
/// [`filled_button`] would make neither read as the recommended one.
pub fn filled_tonal_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = state_layers(theme::PRIMARY_CONTAINER, theme::ON_PRIMARY_CONTAINER);
    variant_button(ui, label, fill, None, theme::ON_PRIMARY_CONTAINER)
}

/// A secondary action that wants to read as its own surface without a
/// [`filled_tonal_button`]'s colour commitment.
///
/// MD3 pairs this with a drop shadow that lifts on hover; egui's stock
/// `Button` has no shadow parameter to hang that on (`Frame::shadow` exists,
/// but `Button` builds its `Frame` internally and only exposes fill, stroke
/// and corner radius), and painting one by hand is exactly the custom
/// painting this module rules out. So this is [`filled_button`] with a
/// neutral surface instead of [`theme::PRIMARY`] and no shadow —
/// [`theme::ELEVATION_1`] sits unused, ready for whatever eventually paints
/// a surface that can use it.
pub fn elevated_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = state_layers(theme::surface_container_high(ui.visuals()), theme::PRIMARY);
    variant_button(ui, label, fill, None, theme::ON_SURFACE)
}

/// A secondary action with a visible boundary but no fill at rest — see this
/// module's header for why the boundary is a scoped `Style` mutation and not
/// `Button::stroke()`.
pub fn outlined_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = transparent_state_layers(ui, theme::PRIMARY);
    variant_button(
        ui,
        label,
        fill,
        Some((1.0, theme::OUTLINE)),
        theme::ON_SURFACE,
    )
}

/// A destructive action — revoking a paired client, clearing a broadcast,
/// and anything else that's wrong or irreversible.
///
/// Not one of MD3's five emphasis levels: MD3 reserves a separate `error`
/// role for this, layered on top of whichever level a view would otherwise
/// use. Same shape as [`outlined_button`] (transparent at rest, an outline
/// that lifts on hover, never a solid fill) but in [`theme::ERROR`] — a
/// destructive control shouldn't also claim a view's one-[`filled_button`]
/// slot. The label is the same red as the outline, unlike
/// [`outlined_button`]'s neutral one, because this control's entire point is
/// to read as "irreversible" at a glance.
pub fn danger_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = transparent_state_layers(ui, theme::ERROR);
    variant_button(ui, label, fill, Some((1.0, theme::ERROR)), theme::ERROR)
}

/// The lowest-emphasis action — no fill, no border, just a label that picks
/// up a faint tint on hover/press. A dialog's "Cancel" next to a
/// [`filled_button`]'s "OK".
///
/// The label is [`theme::ON_SURFACE`], not [`theme::PRIMARY`] as MD3 would
/// have it. MD3 can use `primary` for a label on a transparent surface
/// because its dark schemes put `primary` at a *light* tone; this palette's
/// is deliberately dark (see `theme::PRIMARY`), so the same role would
/// render this label at 1.5:1 — invisible.
pub fn text_button(ui: &mut egui::Ui, label: impl Into<String>) -> egui::Response {
    let fill = transparent_state_layers(ui, theme::PRIMARY);
    variant_button(ui, label, fill, None, theme::ON_SURFACE)
}

/// One entry in a vertical navigation rail — MD3's navigation-drawer item.
/// Full-width, so every entry in a rail is the same size regardless of label
/// length, and taller than the [`MIN_TOUCH_TARGET`] floor since a rail is
/// scanned and clicked far more than it's read.
///
/// Built on [`variant_button_sized`] rather than delegating to
/// [`filled_tonal_button`]/[`outlined_button`] because those fix the
/// footprint at `MIN_TOUCH_TARGET` and a rail entry's whole point is that it
/// fills its column. The two colour sets are those variants' though:
/// selected reads as a tonal fill, unselected as an outline with no fill, so
/// "which one am I on" is answered by fill weight and not by colour alone.
pub fn nav_item(ui: &mut egui::Ui, selected: bool, label: impl Into<String>) -> egui::Response {
    let size = Vec2::new(ui.available_width(), 40.0);
    if selected {
        let fill = state_layers(theme::PRIMARY_CONTAINER, theme::ON_PRIMARY_CONTAINER);
        variant_button_sized(
            ui,
            label,
            fill,
            Some((1.0, theme::OUTLINE)),
            theme::ON_PRIMARY_CONTAINER,
            size,
        )
    } else {
        let fill = transparent_state_layers(ui, theme::PRIMARY);
        let outline = theme::outline_variant(ui.visuals());
        variant_button_sized(
            ui,
            label,
            fill,
            Some((1.0, outline)),
            theme::ON_SURFACE_VARIANT,
            size,
        )
    }
}

// ---------------------------------------------------------------------------
// Selection controls
// ---------------------------------------------------------------------------

/// A multi/either-way selection control — "pick this from a set of options",
/// where the choice is reviewed or confirmed as part of a larger form rather
/// than taking effect the instant it's toggled. Stock `egui::Checkbox`,
/// unstyled beyond what `theme::apply` already does globally.
pub fn checkbox<'a>(
    checked: &'a mut bool,
    label: impl Into<egui::WidgetText>,
) -> egui::Checkbox<'a> {
    egui::Checkbox::new(checked, label)
}

/// An immediate on/off *state*, independent of any other control — MD3
/// distinguishes this from [`checkbox`] (selection) with a track-and-thumb
/// pill that visibly slides between two positions.
///
/// **Known limitation:** egui ships no such widget, and painting one by hand
/// is exactly the custom painting this module's header rules out. This is
/// `egui::Checkbox` today — a real semantic distinction at call sites, with
/// no visual distinction yet. A real MD3 switch is either a small
/// custom-painted widget or an upstream egui feature request.
pub fn switch<'a>(checked: &'a mut bool, label: impl Into<egui::WidgetText>) -> egui::Checkbox<'a> {
    egui::Checkbox::new(checked, label)
}

/// The inner padding [`text_field`]/[`text_area`] give their text.
///
/// egui's stock `TextEdit` uses `Margin::symmetric(4, 2)` — 4px of vertical
/// padding in total, which makes a single-line field noticeably shorter than
/// the [`MIN_TOUCH_TARGET`] button standing next to it in the same row. A
/// field is as much a hit target as a button (you click it to focus it), so
/// it should be as tall as one.
///
/// The value is chosen so that a one-row field lands on
/// [`theme::HIT_TARGET`], not picked by eye —
/// [`tests::a_text_field_is_as_tall_as_the_button_beside_it`] measures a real
/// one against a real [`filled_button`] and fails if the type scale, the
/// vendored face's metrics or this constant drift apart.
pub const FIELD_MARGIN: egui::Margin = egui::Margin::symmetric(8, 8);

/// A single-line text input at [`FIELD_MARGIN`].
///
/// Every text input in the app should come from here rather than
/// `ui.text_edit_singleline` / `egui::TextEdit::singleline`, which is the
/// only way to get a consistent field height: `TextEdit`'s margin is a
/// per-widget builder option with no `Style` hook behind it, so unlike the
/// rest of the theme it can't be installed once in `theme::apply` and
/// inherited.
pub fn text_field(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text).margin(FIELD_MARGIN)
}

/// [`text_field`]'s multi-line counterpart, same padding.
pub fn text_area(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::multiline(text).margin(FIELD_MARGIN)
}

/// The width every select field is laid out at: wider than egui's 100px
/// default (`Style::combo_width`), since MD3 select fields are sized for a
/// sentence, not a single word — a MIDI device or console host name runs
/// long.
const SELECT_WIDTH: f32 = 280.0;

/// A dropdown selection field — MD3's "Select" over a bare `ComboBox`,
/// [`SELECT_WIDTH`] wide with an arrow that points at the side the menu will
/// actually open on. Returns the `ComboBox` builder so callers keep using
/// `show_ui` for the option list exactly as before.
///
/// **Takes `ui` only to predict the popup direction**, and it must be the
/// same `ui` the returned builder's `show_ui` is handed — that's what makes
/// [`select_opens_above`]'s id match the one `ComboBox::show_ui` derives
/// internally (`ui.make_persistent_id(egui::IdSalt::new(id_salt))`, hence
/// the `Copy` bound so the salt can be spent twice).
///
/// **This is the one place in this module that paints.** egui 0.35 draws an
/// unconditional down-triangle: it used to flip when the popup opened
/// upwards and the feature was removed as looking odd in edge cases
/// (emilk/egui#5713). A select near the bottom of a settings window opens
/// upwards essentially always, so a permanent down-arrow points away from
/// the menu every single time. [`select_opens_above`] replicates egui's own
/// placement decision rather than guessing, so the arrow can't disagree with
/// the popup.
pub fn select(
    ui: &egui::Ui,
    id_salt: impl egui::AsIdSalt + Copy,
    selected_text: impl Into<egui::WidgetText>,
) -> egui::ComboBox {
    let opens_above = select_opens_above(ui, select_button_id(ui, id_salt));
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .width(SELECT_WIDTH)
        .icon(
            move |ui: &egui::Ui, rect, visuals: &egui::style::WidgetVisuals, _open| {
                // egui's own icon geometry (`paint_default_icon`), mirrored
                // vertically when the menu is above.
                let rect = egui::Rect::from_center_size(
                    rect.center(),
                    Vec2::new(rect.width() * 0.7, rect.height() * 0.45),
                );
                let points = if opens_above {
                    vec![rect.left_bottom(), rect.right_bottom(), rect.center_top()]
                } else {
                    vec![rect.left_top(), rect.right_top(), rect.center_bottom()]
                };
                ui.painter().add(egui::Shape::convex_polygon(
                    points,
                    visuals.fg_stroke.color,
                    Stroke::NONE,
                ));
            },
        )
}

/// The `Id` `ComboBox` will give this select's *button* — the one its
/// popup's own id and last frame's laid-out rect both hang off.
///
/// The `IdSalt::new` wrapper is load-bearing, not decoration:
/// `ComboBox::from_id_salt` stores `IdSalt::new(salt)` and *then* derives the
/// button id with `ui.make_persistent_id(that_salt)`, so the salt gets hashed
/// twice. Deriving it here with one hash yields a different id, every lookup
/// misses, and the arrow silently never flips.
fn select_button_id(ui: &egui::Ui, id_salt: impl egui::AsIdSalt) -> egui::Id {
    ui.make_persistent_id(egui::IdSalt::new(id_salt))
}

/// Will this select's popup open above the field rather than below it?
///
/// Re-runs the placement search `Popup::menu` will run when the menu opens,
/// on the same inputs: last frame's button rect as the anchor, the popup
/// `Area`'s last known size, zero gap, and `BOTTOM_START` first with its
/// symmetries and then `MENU_ALIGNS` as fallbacks. Same inputs, same answer
/// — so this can't drift out of sync with where the menu lands.
///
/// Returns `false` (arrow down) on the first frame a select is laid out, or
/// before its menu has ever been opened — which is what egui does too, since
/// it falls back to a zero-height popup that always "fits" below.
fn select_opens_above(ui: &egui::Ui, button_id: egui::Id) -> bool {
    let ctx = ui.ctx();
    let Some(anchor) = ctx.read_response(button_id).map(|r| r.rect) else {
        return false;
    };
    let popup_size = egui::AreaState::load(ctx, button_id.with("popup"))
        .and_then(|area| area.size)
        .unwrap_or_else(|| Vec2::new(SELECT_WIDTH, 0.0));
    popup_lands_above(ctx.content_rect(), anchor, popup_size)
}

/// The geometry half of [`select_opens_above`], split out so it can be
/// tested without driving a real popup open across frames.
fn popup_lands_above(content_rect: egui::Rect, anchor: egui::Rect, popup_size: Vec2) -> bool {
    let align = egui::RectAlign::find_best_align(
        std::iter::once(egui::RectAlign::BOTTOM_START)
            .chain(egui::RectAlign::BOTTOM_START.symmetries())
            .chain(egui::RectAlign::MENU_ALIGNS),
        content_rect,
        anchor,
        0.0,
        popup_size,
    )
    .unwrap_or_default();
    // The menu is above iff its *bottom* edge is pinned to the field — which
    // is what distinguishes `TOP_*` from both `BOTTOM_*` and the sideways
    // fallbacks.
    align.child.y() == egui::Align::Max
}

/// A continuous-value control. Stock `egui::Slider` — the MD3 look (filled
/// leading track, round handle) comes from `theme::apply`'s global
/// `Visuals::slider_trailing_fill`/`handle_shape`, so every slider already
/// has it; this exists so call sites reach for `md3::slider` alongside the
/// other constructors instead of `egui::Slider` directly.
pub fn slider<Num: egui::emath::Numeric>(
    value: &mut Num,
    range: std::ops::RangeInclusive<Num>,
) -> egui::Slider<'_> {
    egui::Slider::new(value, range)
}

// ---------------------------------------------------------------------------
// Surfaces
// ---------------------------------------------------------------------------

/// A raised surface for grouping — one card per roster entry, per
/// discovered room, per cast member. Both fills come off `ui` so extra-dark
/// mode reaches them (see this module's header).
///
/// Replaces the "a few labels, then `ui.separator()`" pattern: a hairline
/// between rows says where one ends, a card says what belongs together,
/// which is what a row of four `weak().small()` labels actually needs.
pub fn card(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(theme::surface_container_high(ui.visuals()))
        .stroke(Stroke::new(1.0, theme::outline_variant(ui.visuals())))
        .corner_radius(theme::SHAPE_MEDIUM)
        .inner_margin(egui::Margin::same(12))
}

/// The frame a chrome panel wears: a toolbar, a side rail, the preset rail.
///
/// egui's panels default to `Visuals::panel_fill`, which is the *window's*
/// colour — so a window built entirely from panels renders as one flat
/// rectangle and the neutral ladder [`crate::theme`] is built around never
/// appears on screen. This is the step up from it that makes a rail read as
/// chrome docked against the content rather than as more content.
pub fn chrome_frame(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(theme::surface_container(ui.visuals()))
        .inner_margin(egui::Margin::symmetric(8, 6))
}

/// A single-row list entry: horizontal content, MD3's ~48dp row height,
/// consistent padding. Stock `Frame` + `Ui::horizontal`.
pub fn list_item<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    Frame::new()
        .inner_margin(egui::Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_min_height(theme::HIT_TARGET.max(48.0));
            ui.horizontal(|ui| add_contents(ui)).inner
        })
}

/// A row of mutually-exclusive selectable labels, with the clicked index
/// returned if any — `ui.selectable_label` in a loop, named for what it is
/// so call sites reach for one shared helper instead of re-deriving it.
pub fn tabs(ui: &mut egui::Ui, selected: usize, labels: &[&str]) -> Option<usize> {
    let mut clicked = None;
    ui.horizontal(|ui| {
        for (i, label) in labels.iter().enumerate() {
            if ui.selectable_label(i == selected, *label).clicked() {
                clicked = Some(i);
            }
        }
    });
    clicked
}

/// A transient bottom-anchored status strip — MD3's Snackbar. Stock
/// `egui::Area` + `Frame`; callers still own *when* to show one and for how
/// long.
///
/// Uses the same neutral-surface roles as [`card`] rather than MD3's literal
/// "inverse surface" token: this theme has no inverse-neutral ladder (the
/// whole palette is built around one dark booth, not a light/dark pair to
/// invert between), and a raised card already reads as distinctly forward of
/// the chrome around it.
pub fn snackbar(ctx: &egui::Context, id_salt: impl egui::AsId, message: &str) {
    let visuals = ctx.style_of(ctx.theme()).visuals.clone();
    egui::Area::new(egui::Id::new(id_salt))
        .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            Frame::new()
                .fill(theme::surface_container_high(&visuals))
                .stroke(Stroke::new(1.0, theme::outline_variant(&visuals)))
                .corner_radius(theme::SHAPE_EXTRA_SMALL)
                .inner_margin(egui::Margin::symmetric(16, 12))
                .show(ui, |ui| {
                    ui.label(RichText::new(message).color(theme::ON_SURFACE));
                });
        });
}

/// [`theme::title_large`] and friends return a bare `FontId`; this is the
/// same idea for the one MD3 label role most likely to head a group of
/// controls. Kept here rather than in `theme.rs` since it's a
/// widget-construction concern, not a token.
pub fn section_heading(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into())
            .font(theme::title_medium())
            .color(theme::ON_SURFACE),
    );
}

/// Secondary text — supporting metadata, timestamps, the explanatory line
/// under a control. Named so call sites stop reaching for `RichText::weak()`,
/// which resolves to egui's own washed-out grey rather than
/// [`theme::ON_SURFACE_VARIANT`] and so drifts off the palette.
pub fn dim(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(theme::ON_SURFACE_VARIANT)
}

/// Labels that repeat on every row and must never compete with the value
/// beside them — a timestamp column, a source badge. [`theme::TEXT_FAINT`]
/// at [`theme::LABEL_SMALL_SIZE`]; deliberately below the contrast floor
/// [`theme::tests::readable_text_clears_wcag_aa_on_a_card`] holds
/// [`dim`] to, which is why nothing that has to be *read* may use it.
pub fn faint(text: impl Into<String>) -> RichText {
    RichText::new(text.into())
        .color(theme::TEXT_FAINT)
        .size(theme::LABEL_SMALL_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> egui::Context {
        let ctx = egui::Context::default();
        theme::apply(&ctx, theme::Options::default());
        ctx
    }

    /// The stroke/fill trap this module's header walks through: an MD3
    /// outlined button's footprint must not move when the pointer enters it.
    fn measure_footprint(
        hover: bool,
        add: impl Fn(&mut egui::Ui) -> egui::Response,
    ) -> (f32, bool) {
        let ctx = ctx();
        let mut rect = egui::Rect::NOTHING;
        let mut hovered = false;
        // Three passes: egui needs a frame to lay the button out before a
        // pointer position can land on it, and another for the hover to be
        // reflected in the response.
        for _ in 0..3 {
            let mut input = egui::RawInput::default();
            if hover {
                input.events.push(egui::Event::PointerMoved(rect.center()));
            }
            let _ = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let response = add(ui);
                    rect = response.rect;
                    hovered = response.hovered();
                });
            });
        }
        (rect.width(), hovered)
    }

    #[test]
    fn outlined_button_footprint_does_not_move_on_hover() {
        let (resting, _) = measure_footprint(false, |ui| outlined_button(ui, "Join"));
        let (hovered_width, hovered) = measure_footprint(true, |ui| outlined_button(ui, "Join"));
        assert!(hovered, "the test never actually hovered the button");
        assert_eq!(
            resting, hovered_width,
            "outlined button footprint moved on hover"
        );
    }

    #[test]
    fn filled_button_footprint_does_not_move_on_hover() {
        let (resting, _) = measure_footprint(false, |ui| filled_button(ui, "Send"));
        let (hovered_width, hovered) = measure_footprint(true, |ui| filled_button(ui, "Send"));
        assert!(hovered, "the test never actually hovered the button");
        assert_eq!(
            resting, hovered_width,
            "filled button footprint moved on hover"
        );
    }

    /// A hovered button has to look different from a resting one — the
    /// regression `Button::fill()` would introduce (see this module's
    /// header), which no footprint test would catch.
    #[test]
    fn every_variant_lifts_on_hover_and_press() {
        for (name, fill) in [
            ("filled", state_layers(theme::PRIMARY, theme::ON_PRIMARY)),
            (
                "tonal",
                state_layers(theme::PRIMARY_CONTAINER, theme::ON_PRIMARY_CONTAINER),
            ),
        ] {
            assert_ne!(fill[0], fill[1], "{name}: hover fill matches resting fill");
            assert_ne!(fill[1], fill[2], "{name}: pressed fill matches hover fill");
        }
    }

    /// Extra-dark mode has to reach the variants that tint the chrome behind
    /// them: an outlined button hovering to the *standard* chrome colour
    /// would flash a lighter patch than the panel it sits on.
    #[test]
    fn transparent_variants_tint_the_installed_chrome() {
        let mut fills = Vec::new();
        for extra_dark in [false, true] {
            let ctx = egui::Context::default();
            theme::apply(
                &ctx,
                theme::Options {
                    extra_dark,
                    ..Default::default()
                },
            );
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    fills.push(transparent_state_layers(ui, theme::PRIMARY));
                });
            });
        }
        assert_eq!(fills[0][0], fills[1][0], "both rest transparent");
        assert_ne!(
            fills[0][1], fills[1][1],
            "extra-dark hover tint matches the standard one"
        );
    }

    /// A field and the button beside it are both hit targets and both sit in
    /// the same row, so a field that is visibly shorter reads as a mistake.
    /// This is what [`FIELD_MARGIN`] is tuned against — if the type scale or
    /// the vendored face's metrics move, this fails instead of the layout
    /// quietly going lopsided again.
    #[test]
    fn a_text_field_is_as_tall_as_the_button_beside_it() {
        let ctx = ctx();
        let mut text = String::new();
        let (mut field, mut button) = (0.0, 0.0);
        for _ in 0..2 {
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        field = ui.add(text_field(&mut text)).rect.height();
                        button = filled_button(ui, "Send").rect.height();
                    });
                });
            });
        }
        assert!(
            field >= theme::HIT_TARGET,
            "a text field is {field}px, below the {}px hit-target floor",
            theme::HIT_TARGET
        );
        assert!(
            (field - button).abs() <= 2.0,
            "a text field ({field}px) and the button beside it ({button}px) \
             are more than 2px apart"
        );
    }

    /// [`select`]'s arrow must point at the side the menu lands on. A popup
    /// tall enough that it can't fit below a field near the bottom edge is
    /// the case egui's unconditional down-triangle gets wrong.
    #[test]
    fn select_arrow_follows_the_popup() {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(400.0, 400.0));
        let near_top = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), Vec2::new(280.0, 24.0));
        let near_bottom =
            egui::Rect::from_min_size(egui::pos2(10.0, 360.0), Vec2::new(280.0, 24.0));
        let popup = Vec2::new(280.0, 200.0);
        assert!(!popup_lands_above(screen, near_top, popup));
        assert!(popup_lands_above(screen, near_bottom, popup));
    }

    /// Same footprint guarantee as the two dedicated tests above, for every
    /// other variant — most of them are cheaper since they never touch
    /// `bg_stroke.width` at all, but this pins down that `variant_button`'s
    /// shared plumbing doesn't regress for any of them.
    #[test]
    fn every_button_variant_footprint_does_not_move_on_hover() {
        type Variant = fn(&mut egui::Ui, &str) -> egui::Response;
        let variants: [(&str, Variant); 6] = [
            ("filled_tonal", |ui, s| filled_tonal_button(ui, s)),
            ("elevated", |ui, s| elevated_button(ui, s)),
            ("danger", |ui, s| danger_button(ui, s)),
            ("text", |ui, s| text_button(ui, s)),
            ("nav_item selected", |ui, s| nav_item(ui, true, s)),
            ("nav_item unselected", |ui, s| nav_item(ui, false, s)),
        ];
        for (name, make) in variants {
            let (at_rest, _) = measure_footprint(false, |ui| make(ui, "Label"));
            let (under_pointer, hovered) = measure_footprint(true, |ui| make(ui, "Label"));
            assert!(hovered, "{name}: pointer never hovered");
            assert_eq!(at_rest, under_pointer, "{name}: resized on hover");
        }
    }

    /// `min_size` enforces the hit-target floor regardless of how short the
    /// label is — 48px wide, [`theme::HIT_TARGET`] tall.
    #[test]
    fn buttons_meet_the_minimum_hit_target() {
        let ctx = ctx();
        let mut rect = egui::Rect::NOTHING;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                rect = filled_button(ui, "Go").rect;
            });
        });
        assert!(rect.width() >= 48.0, "width {}", rect.width());
        assert!(
            rect.height() >= theme::HIT_TARGET,
            "height {} is under the {}px hit-target floor",
            rect.height(),
            theme::HIT_TARGET
        );
    }

    /// Every variant's label has to be readable on the fill it actually sits
    /// on. Transparent-filled variants are checked against the chrome
    /// surface they're drawn over, which is what "transparent" resolves to
    /// wherever this module is used.
    #[test]
    fn every_variant_label_is_readable_on_its_own_fill() {
        let ctx = ctx();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            let chrome = theme::surface_container(ui.visuals());
            let surface_high = theme::surface_container_high(ui.visuals());
            for (name, label, fill) in [
                ("filled", theme::ON_PRIMARY, theme::PRIMARY),
                (
                    "filled_tonal",
                    theme::ON_PRIMARY_CONTAINER,
                    theme::PRIMARY_CONTAINER,
                ),
                ("elevated", theme::ON_SURFACE, surface_high),
                ("outlined", theme::ON_SURFACE, chrome),
                ("danger", theme::ERROR, chrome),
                ("text", theme::ON_SURFACE, chrome),
                ("nav_item unselected", theme::ON_SURFACE_VARIANT, chrome),
            ] {
                let ratio = theme::tests::contrast(label, fill);
                assert!(
                    ratio >= 4.5,
                    "{name} button's label is only {ratio:.2}:1 on its fill"
                );
            }
        });
    }

    /// The consolidation this module is for: an MD3 button and the stock
    /// egui control beside it in the same row have to be the same height,
    /// not a head taller just because the number came from a spec.
    #[test]
    fn md3_and_stock_buttons_are_the_same_height() {
        let ctx = ctx();
        let (mut md3_h, mut stock_h) = (0.0, 0.0);
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                md3_h = filled_button(ui, "Go").rect.height();
                stock_h = ui.button("Go").rect.height();
            });
        });
        assert_eq!(md3_h, stock_h, "md3 {md3_h} vs stock {stock_h}");
    }

    /// Smoke test: every remaining constructor builds and shows without
    /// panicking.
    #[test]
    fn surface_constructors_show_without_panicking() {
        let ctx = ctx();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                card(ui).show(ui, |ui| {
                    ui.label("card");
                });
                chrome_frame(ui).show(ui, |ui| {
                    ui.label("chrome");
                });
                list_item(ui, |ui| {
                    ui.label("list item");
                });
                let _ = tabs(ui, 0, &["One", "Two"]);
                section_heading(ui, "Heading");
                ui.label(dim("dim"));
                ui.label(faint("faint"));
            });
        });
        snackbar(&ctx, "test-snackbar", "Denied");
    }

    #[test]
    fn checkbox_and_switch_are_addable_widgets() {
        let ctx = ctx();
        let mut a = false;
        let mut b = true;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(checkbox(&mut a, "Checkbox"));
                ui.add(switch(&mut b, "Switch"));
            });
        });
        assert!(!a);
        assert!(b);
    }

    #[test]
    fn slider_and_select_build_without_panicking() {
        let ctx = ctx();
        let mut value = 5_i32;
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add(slider(&mut value, 0..=10));
                select(ui, "test-select", "Choice").show_ui(ui, |ui| {
                    ui.label("Option");
                });
            });
        });
    }

    /// [`select_opens_above`] can only see the popup it's predicting if
    /// [`select_button_id`] derives the same `Id` `ComboBox` does — and that
    /// derivation is a double hash of the salt, easy to get wrong in a way
    /// nothing else notices: a mismatched id just makes every lookup miss
    /// and the arrow point down forever. So this drives a real select open
    /// with a real click and asks *egui* whether the popup behind our id is
    /// the one that opened.
    #[test]
    fn our_select_id_is_the_one_egui_gives_the_combo_box() {
        let ctx = ctx();
        let mut button_id = None;
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(400.0, 300.0));
        let mut frame = |events: Vec<egui::Event>| {
            let input = egui::RawInput {
                screen_rect: Some(screen),
                events,
                ..Default::default()
            };
            let _ = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    button_id = Some(select_button_id(ui, "test-select"));
                    select(ui, "test-select", "Choice").show_ui(ui, |ui| {
                        ui.label("Option");
                    });
                });
            });
        };
        frame(vec![]);
        // The select is the panel's first widget, so its button is under
        // the panel's top-left corner.
        let pos = screen.min + Vec2::new(30.0, 20.0);
        let click = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(vec![egui::Event::PointerMoved(pos), click(true)]);
        frame(vec![click(false)]);

        let button_id = button_id.expect("the select was laid out");
        assert!(
            egui::ComboBox::is_open(&ctx, button_id),
            "clicking the select opened a popup egui doesn't file under the id \
             we predict its placement with"
        );
    }
}
