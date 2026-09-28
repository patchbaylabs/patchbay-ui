<div align="center">

<img src="https://raw.githubusercontent.com/patchbaylabs/patchbaylabs/main/assets/patchbaylabs-mark-transparent.png" width="96" alt="Patchbay Labs">

# patchbay-ui

*The shared egui theme and Material Design 3 widgets behind every Patchbay Labs app.*

[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust)](https://www.rust-lang.org)
[![egui](https://img.shields.io/badge/egui-0.35-blue)](https://github.com/emilk/egui)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue)](LICENSE-MIT)

[Features](#features) • [Getting started](#getting-started) • [Theme](#theme) • [Widgets](#widgets) • [Gallery](#gallery) • [Testing](#testing)

</div>

`patchbay-ui` is a small Rust crate: one designed dark theme plus a set of MD3-style widget constructors for [egui](https://github.com/emilk/egui), shared by **mictrace**, **sidestage** and **rcpmix** so that every app reads as one product when its window sits next to a sibling's on the same operator's screen.

The palette is built around one question: *what does an operator need to be able to answer without focusing?* Every colour answers exactly one — read at a glance, in a dark booth, by someone who is also watching a stage.

## Features

- **One palette, one meaning per colour** — a window at rest is monochrome; every other saturated pixel means something
- **MD3 widgets with no custom painting** — thin, pre-styled wrappers over stock egui widgets
- **Contrast that's tested, not eyeballed** — primary text ≈13:1 on a card, and the test suite fails any change that drops below WCAG AA
- **32 px hit-target floor** — techs wear gloves, use touchscreens, and hit controls in the dark mid-cue
- **Extra-dark mode** — a full second neutral ladder for FOH positions, not four overridden fills
- **Identical layout on every platform** — DejaVu Sans Mono is vendored, so macOS, Windows and Linux all render the same

## Getting started

The crate depends only on `egui` itself, so it works with any frontend (eframe, custom backends). Depend on it via git:

```toml
[dependencies]
patchbay-ui = { git = "https://github.com/patchbaylabs/patchbay-ui" }
```

Install the theme once at startup — and again whenever its options change, not every frame:

```rust
use patchbay_ui::{md3, theme};

theme::apply(&ctx, theme::Options::default());

// Then build out of the controls every embedding app shares:
md3::section_heading(ui, "RF");
md3::card(ui).show(ui, |ui| {
    ui.label(md3::dim("SM58 · handheld"));
    if md3::filled_button(ui, "Arm").clicked() {
        // ...
    }
});
```

## Theme

`theme::apply` installs the whole booth theme onto an `egui::Context`: fonts, visuals, spacing, and egui's five built-in text styles wired to their MD3 equivalents — so most widgets pick up the type scale for free. The caller gets exactly two knobs (`theme::Options`):

| Option | What it does |
|---|---|
| `extra_dark` | Swaps the standard neutral ladder for a darker one, for FOH positions where even a near-black window blooms |
| `zoom` | Drives `Context::set_zoom_factor` — the one whole-UI size knob, so text, spacing and hit targets stay in proportion |

Everything else is fixed: this is one designed palette, not a themable surface.

### Colour is state

Neutrals carry structure — a ladder of steps from the window background up to primary text, so a card sits above the window and a recessed lane reads as cut into it. Colour is reserved for state, with exactly one meaning fixed across every app that uses this crate:

| Colour | Meaning |
|---|---|
| `ACCENT` | This wants your attention *now* — a solo ring, a flash pulse, unread traffic |
| `OK` | Reachable / active / healthy |
| `WARN` | Degraded, but still working — a low battery, a peer gone quiet |
| `DANGER` | Wrong, or irreversible — a clip, a broadcast clear |
| `INFO` | Discovery only — RF, rooms, peers, hyperlinks |

The MD3 role set (`PRIMARY`, `ERROR`, `ON_*`, containers, elevation, shape and state-layer tokens) is a layer over that same palette, not a second one — no new saturated hues anywhere.

The typeface is vendored and *prepended* to both egui font families rather than replacing them, so user-supplied text in any script still renders as glyphs instead of tofu.

### Reading surfaces back

Extra-dark mode swaps the ladder at `apply` time, so anything that paints its own surface reads the installed one back off `ui.visuals()` instead of naming a `const`:

```rust
let fill = theme::surface_container_high(ui.visuals()); // a card, in either mode
```

The accessors — `surface`, `surface_container`, `surface_container_high`, `outline_variant` — are the whole transport, so no call site needs to know which egui slot a role rides in.

## Widgets

`md3` holds thin, pre-styled constructors over **stock egui widgets** — no new widget types:

| Category | Constructors |
|---|---|
| Buttons | `filled_button` · `filled_tonal_button` · `elevated_button` · `outlined_button` · `danger_button` · `text_button` · `nav_item` |
| Building blocks | `variant_button` · `variant_button_sized` · `state_layers` · `transparent_state_layers` |
| Selection | `checkbox` · `switch` · `slider` |
| Text input | `text_field` · `text_area` · `select` |
| Surfaces | `card` · `chrome_frame` · `list_item` · `tabs` · `snackbar` |
| Text styling | `section_heading` · `dim` · `faint` |

Convention: **at most one `filled_button` per view** — it marks the one recommended action, the same "don't make everything shout" reasoning that reserves colour for state.

> [!NOTE]
> **Deliberate divergences from the MD3 spec**, each pinned by a test:
> - Buttons use a 4 px corner radius, not the spec's pill — a panel docked into an app whose shape language is cards and lanes would otherwise read as borrowed from somewhere else
> - The hit-target floor is 32 px, not MD3's 48 dp, so MD3 controls and stock egui controls sit side by side in one settings row at one height
> - `text_button`'s label is on-surface rather than primary, because this palette's primary is deliberately dark and a primary label would render at 1.5:1 — invisible

> [!NOTE]
> **Known limitation:** `switch` renders as a checkbox today. egui ships no track-and-thumb switch, and hand-painting one is custom painting this module rules out; the semantic distinction exists at call sites. `select`'s arrow, which flips to point at the side its menu will actually open on, is the one place in the crate that does paint.

## Gallery

```bash
cargo run --example gallery
```

A visual smoke-check of every widget the crate exports — the test suite proves footprints and contrast ratios, but not "does this actually look right". Not a snapshot test: look at it. The toolbar toggles extra-dark mode and zoom live.

## Testing

```bash
cargo test
```

The suite enforces the theme's contracts rather than snapshotting pixels:

- Text clears WCAG AA on a card in *both* ladders, and every state colour clears the 3:1 non-text floor
- The "wants you now" amber and the warning yellow are provably distinct, and chrome's primary is provably not the accent
- No button's footprint moves on hover — the regression `Button::fill()` / `Button::stroke()` would reintroduce
- Every ladder step stays visibly above the one behind it, so extra-dark mode stays a ladder and not a flattening
- A text field is exactly as tall as the button beside it, and MD3 and stock buttons are the same height
- `select`'s predicted popup direction is the one egui actually uses, verified with a real click

---

Built by [Patchbay Labs](https://github.com/patchbaylabs) — tools for the booth, the deck, and the desk.
