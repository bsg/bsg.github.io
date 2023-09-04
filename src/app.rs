mod github;
mod window;

use egui::{FontId, TextStyle};

use self::github::LatestCommits;
use std::sync::{Arc, RwLock};

pub struct BsgApp {
    whoami_open: bool,
    pp_hnd: Arc<RwLock<Option<egui::TextureHandle>>>,
    latest_commits: Arc<RwLock<Option<LatestCommits>>>,
}

impl Default for BsgApp {
    fn default() -> Self {
        Self {
            whoami_open: true,
            pp_hnd: Arc::new(RwLock::new(None)),
            latest_commits: Arc::new(RwLock::new(None)),
        }
    }
}

impl BsgApp {
    fn fetch_initial_data(me: &BsgApp, cc: &eframe::CreationContext<'_>) {
        let pp_hnd_cloned = me.pp_hnd.clone();
        let ctx_cloned = cc.egui_ctx.clone();
        if me.pp_hnd.read().unwrap().is_none() {
            wasm_bindgen_futures::spawn_local(async {
                github::fetch_pp(pp_hnd_cloned, ctx_cloned).await
            });
        }

        let latest_commits_cloned = me.latest_commits.clone();
        if me.latest_commits.read().unwrap().is_none() {
            wasm_bindgen_futures::spawn_local(async {
                github::fetch_latest_commits(latest_commits_cloned).await
            });
        }
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "hack-regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/Hack-Regular.ttf")),
        );

        fonts.font_data.insert(
            "roboto-regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/Roboto-Regular.ttf")),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "hack-regular".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("hack-regular".to_owned());

        ctx.set_fonts(fonts);
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let me: Self = Default::default();
        Self::fetch_initial_data(&me, cc);
        Self::setup_custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        me
    }
}

impl eframe::App for BsgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_portrait = ctx.available_rect().aspect_ratio() < 1.0;

        if is_portrait {
            // TODO we probably don't wanna do this every frame
            let mut style: egui::Style = (*ctx.style()).clone();
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(11.0, egui::FontFamily::Monospace),
            );
            _ = style.override_text_style.insert(egui::TextStyle::Small);
            ctx.set_style(style);
        }

        if !is_portrait {
            egui::TopBottomPanel::top("top_panel")
                .min_height(20.0)
                .show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("About", |ui| {
                            if ui.button("whoami").clicked() {
                                self.whoami_open = true;
                            }
                        });
                        egui::warn_if_debug_build(ui);
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            window::whoami::render(self, ctx, ui, is_portrait);
            if !is_portrait {
                // TODO sum shit
            };
        });
    }
}
