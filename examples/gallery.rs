//! A visual smoke-check for every widget this crate exports — `cargo run
//! --example gallery` — since the test suite proves footprints and contrast
//! ratios but not "does this actually look right". Not a snapshot test:
//! look at it.

use patchbay_ui::{md3, theme};

struct Gallery {
    checked: bool,
    switched: bool,
    slider_value: f32,
    text: String,
    selected_tab: usize,
    selected_option: usize,
    options: Vec<String>,
    show_snackbar: bool,
    extra_dark: bool,
    zoom: f32,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            checked: false,
            switched: true,
            slider_value: 40.0,
            text: String::new(),
            selected_tab: 0,
            selected_option: 0,
            options: vec!["Option A".into(), "Option B".into(), "Option C".into()],
            show_snackbar: false,
            extra_dark: false,
            zoom: 1.0,
        }
    }
}

impl Gallery {
    fn apply_theme(&self, ctx: &egui::Context) {
        theme::apply(
            ctx,
            theme::Options {
                extra_dark: self.extra_dark,
                zoom: self.zoom,
            },
        );
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);

        egui::Panel::top("toolbar").show(ui, |ui| {
            md3::chrome_frame(ui).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(md3::checkbox(&mut self.extra_dark, "Extra-dark"));
                    ui.add(md3::slider(&mut self.zoom, 0.8..=1.6).text("Zoom"));
                });
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                md3::section_heading(ui, "Buttons");
                ui.horizontal_wrapped(|ui| {
                    md3::filled_button(ui, "Filled");
                    md3::filled_tonal_button(ui, "Filled tonal");
                    md3::elevated_button(ui, "Elevated");
                    md3::outlined_button(ui, "Outlined");
                    md3::danger_button(ui, "Danger");
                    md3::text_button(ui, "Text");
                });
                ui.add_space(12.0);

                md3::section_heading(ui, "Nav rail");
                ui.horizontal(|ui| {
                    for (i, label) in ["Overview", "Cues", "Settings"].iter().enumerate() {
                        md3::nav_item(ui, i == self.selected_tab, *label);
                    }
                });
                ui.add_space(12.0);

                md3::section_heading(ui, "Selection controls");
                ui.horizontal(|ui| {
                    ui.add(md3::checkbox(&mut self.checked, "Checkbox"));
                    ui.add(md3::switch(&mut self.switched, "Switch"));
                    ui.add(md3::slider(&mut self.slider_value, 0.0..=100.0));
                });
                ui.add_space(12.0);

                md3::section_heading(ui, "Text field & select");
                ui.horizontal(|ui| {
                    ui.add(md3::text_field(&mut self.text));
                    md3::select(
                        ui,
                        "gallery-select",
                        self.options[self.selected_option].clone(),
                    )
                    .show_ui(ui, |ui| {
                        for (i, option) in self.options.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_option, i, option);
                        }
                    });
                });
                ui.add_space(12.0);

                md3::section_heading(ui, "Tabs");
                if let Some(i) = md3::tabs(ui, self.selected_tab, &["One", "Two", "Three"]) {
                    self.selected_tab = i;
                }
                ui.add_space(12.0);

                md3::section_heading(ui, "Surfaces");
                md3::card(ui).show(ui, |ui| {
                    ui.label("A card, raised above the surface behind it.");
                    ui.label(md3::dim("Secondary text via dim()."));
                    ui.label(md3::faint("Repeated-per-row text via faint()."));
                });
                ui.add_space(8.0);
                md3::list_item(ui, |ui| {
                    ui.label("A list item row.");
                });
                ui.add_space(12.0);

                if ui.button("Show snackbar").clicked() {
                    self.show_snackbar = true;
                }
            });
        });

        if self.show_snackbar {
            md3::snackbar(&ctx, "gallery-snackbar", "This is a snackbar.");
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "patchbay-ui gallery",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(Gallery::default()))),
    )
}
