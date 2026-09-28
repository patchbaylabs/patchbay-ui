# AGENTS.md

## Project Overview

`patchbay-ui` is a small Rust library crate (edition 2021): one designed dark theme plus a set of MD3-style widget constructors for [egui](https://github.com/emilk/egui), shared by the apps **mictrace**, **sidestage** and **rcpmix** so every Patchbay Labs app reads as one product on the same operator's screen.

- Sole runtime dependency: `egui` (0.35). Dev-dependencies: `eframe` (the gallery example) and `egui_kittest`.
- Two modules, both fully re-exported from the crate root (`src/lib.rs`):
  - `src/theme.rs` — the palette (neutral ladder + semantic state colours), MD3 role aliases over it, the type scale, vendored DejaVu Sans Mono font install, and `theme::apply` which installs everything onto an `egui::Context` at startup. Exactly two knobs: `Options { extra_dark, zoom }`.
  - `src/md3.rs` — thin, pre-styled constructors over **stock egui widgets** (buttons, checkbox, switch, slider, text field, select, card, tabs, snackbar, …). No new widget types, no custom painting (the one existing exception: `select`'s arrow).
- `examples/gallery.rs` — an `eframe` app showing every exported widget, for visual inspection.
- This is a library, consumed by the other apps via a git dependency — API changes here are cross-repo changes for all three embedding apps.

## Setup Commands

- Install toolchain: recent stable Rust (`rustc 1.92` works).
- Fetch and build dependencies: `cargo build`
- No environment variables, database, or platform setup required. The UI font is vendored at `assets/fonts/DejaVuSansMono.ttf` and embedded via `include_bytes!`, so nothing to install.

## Development Workflow

- Build: `cargo build`
- Run the visual gallery (opens a real desktop window; toggles extra-dark mode and zoom live): `cargo run --example gallery`
- Format: `cargo fmt` (the tree is `cargo fmt --check` clean; default rustfmt config, no `rustfmt.toml`)
- Lint: `cargo clippy --all-targets` (clean as of this writing — keep it that way)
- Docs: `cargo doc --no-deps` — warns about intra-doc links to private items; use `cargo doc --no-deps --document-private-items` to resolve them. Doc comments here carry the design rationale, so treat them as part of the product: update them alongside the code they explain.
- No watch mode is needed — tests run headless in well under a second.

## Testing Instructions

- Run all tests: `cargo test` (28 tests across 2 suites as of this writing; runs headless, ~0.05s)
- Run a subset by name fragment: `cargo test footprint` (test names are sentence-like, e.g. `cargo test contrast`, `cargo test hover`)
- Tests live **inline** in `#[cfg(test)] mod tests` at the bottom of `src/theme.rs` and `src/md3.rs` — there is no `tests/` directory. New tests go next to the code they pin.
- The suite enforces **contracts**, not pixel snapshots. Existing pins include:
  - Text clears WCAG AA on a card in **both** neutral ladders (standard and extra-dark); every state colour clears the 3:1 non-text floor
  - `ACCENT` (amber) is provably distinct from `WARN` (yellow)
  - No button's footprint moves on hover; every variant visibly lifts on hover/press
  - Hit-target floor: every control ≥ 32 px tall; MD3 and stock egui buttons are the same height; a text field is as tall as the button beside it
  - Every ladder step stays visibly above the one behind it (extra-dark stays a ladder, not a flattening)
  - `select`'s predicted popup direction is the one egui actually uses, verified with a real synthetic click
- **A failing test after a palette or geometry change is the system working.** Do not loosen an assertion to make a change pass — either fix the change, or, if a divergence is deliberate, update the pinned value and document why in the doc comment the test points at.
- Convention: every deliberate divergence from the MD3 spec (4 px button radius instead of a pill, 32 px floor instead of 48 dp, `text_button`'s on-surface label) is pinned by a test. New widgets that diverge need the same treatment.
- New widget constructors need tests pinning at minimum: label contrast ≥ 4.5:1 on the fill it actually renders on, footprint stability on hover, and the 32 px hit-target floor.
- Tests drive egui headless via `Context::run_ui` with synthetic `RawInput` (see `measure_footprint` in `src/md3.rs`); egui needs a couple of frames to lay out, hover, and reflect state — copy that pattern. `theme::tests::contrast` is the shared WCAG contrast helper. `egui_kittest` is available as a dev-dependency if a richer harness is needed.
- When touching `theme`, run both ladders through the relevant assertions — a value that only works in standard mode is a bug.

## Code Style

- `cargo fmt` formatting, no custom config. Run `cargo fmt` before committing.
- Naming: snake_case functions/types, `SCREAMING_CASE` consts. Palette consts are named for their meaning (`ACCENT`, `OK`, `WARN`, `DANGER`, `INFO`), never their hue.
- Every public item has a rustdoc comment that explains **why**, not just what — the rationale (booth legibility, hover feedback, cross-app consistency) is the spec. Module-level `//!` headers walk through the design and its traps; keep them current.
- Test names are full sentences stating the contract they enforce (e.g. `readable_text_clears_wcag_aa_on_a_card`, `outlined_button_footprint_does_not_move_on_hover`).
- Sections within a file are separated by `// ----` divider comments.

## Design Invariants

These rules come from the embedding apps' operating context (dark booth, glance reading, gloved touch) and are load-bearing. Breaking them silently changes three other products.

1. **Colour is state.** A window at rest is monochrome. Each saturated colour has exactly one meaning across all apps: `ACCENT` = wants attention *now*, `OK` = reachable/active, `WARN` = degraded but working, `DANGER` = wrong/irreversible, `INFO` = discovery. Never add a new saturated hue or reuse a state colour decoratively.
2. **No `Button::fill()` or `Button::stroke()`.** `stroke()` makes the footprint state-dependent (jitters on hover); `fill()` kills hover/press feedback. Coloured variants go through `variant_button`, which scopes a `Style` mutation over `ui.style_mut().visuals.widgets.{inactive,hovered,active}` instead. This is documented as "the stroke *and* fill trap" in `src/md3.rs`'s header.
3. **No custom painting in `md3`.** Everything is a stock egui widget underneath, pre-styled. Anything stock widgets can't paint (a real track-and-thumb switch, drop shadows) is called out in its doc comment rather than worked around.
4. **Read surfaces back from `ui.visuals()`**, never from a neutral `const` at a call site — extra-dark mode swaps the whole `Neutrals` ladder at `apply` time, so a const can name a background that isn't on screen. Use the accessors: `theme::surface`, `surface_container`, `surface_container_high`, `outline_variant`. (Role colours not on the ladder — `PRIMARY`, `ERROR`, `OUTLINE` — are the same in both modes and stay consts.)
5. **32 px hit-target floor**, `MIN_TOUCH_TARGET` of 48×32 for buttons. Deliberate divergence from MD3's 48 dp so MD3 and stock egui controls sit at one height in a shared row.
6. **At most one `filled_button` per view** — it marks the one recommended action.
7. **Extra-dark mode is a second full `Neutrals` ladder**, not four overridden fills; spacing between steps must survive.
8. **App-specific values stay in the apps.** Anything that only makes sense in mictrace/sidestage/rcpmix (meter ramps, RF traces, chat strobe colours) belongs there, not here.

## Build and Deployment

- There is no deploy step: this is a library crate. Consumers depend on it via git — `patchbay-ui = { git = "https://github.com/patchbaylabs/patchbay-ui" }` — so `main` is effectively the release channel. Don't push broken builds to `main`.
- `Cargo.lock` is gitignored (library crate). Don't commit it.
- No CI pipeline exists in this repo yet, so the checks below are your responsibility before pushing.

## Pull Request Guidelines

- Before every commit, run all three and fix what they report:
  - `cargo fmt --check`
  - `cargo clippy --all-targets`
  - `cargo test`
- Commit messages: imperative one-line summaries (see the repo's history, e.g. "Add patchbay-ui: shared egui theme and MD3 widgets"). No prefix scheme.
- Anything user-visible in the theme or widgets deserves a look through the gallery (`cargo run --example gallery`) in both standard and extra-dark mode — the tests prove footprints and contrast, not "does this actually look right".

## Additional Notes

- egui is version-pinned across the family: this crate's `egui`/`eframe` versions must match what the embedding apps use, since they pass `egui::Context` and widget types across the crate boundary. Bumping egui is a coordinated cross-repo change.
- The palette's legibility targets assume the vendored DejaVu Sans Mono and its metrics. If the type scale or the face moves, expect the field-height test (`a_text_field_is_as_tall_as_the_button_beside_it`) to fail — that's it doing its job.
- `BG_SUNKEN` is translucent on purpose (a lane must read as *cut into* whatever is behind it, on both ladders) — don't "fix" it to an opaque fill.
- `switch` renders as a checkbox today, and that's documented as a known limitation — don't hand-paint a track-and-thumb without revisiting the no-custom-painting rule first.
- Dual-licensed `MIT OR Apache-2.0`; new contributions are expected to be fine under both. The vendored font's license lives at `assets/fonts/LICENSE-DejaVu.txt`.
