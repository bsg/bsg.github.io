mod github;

use std::sync::{Arc, RwLock};
use egui::{Align2, RichText};
use self::github::LatestCommits;

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
        egui::TopBottomPanel::top("top_panel").min_height(20.0).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("About", |ui| {
                    if ui.button("whoami").clicked() {
                        self.whoami_open = true;
                    }
                });
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Window::new("$ whoami")
                .pivot(Align2::CENTER_CENTER)
                .default_pos(ctx.screen_rect().center_top())
                .resizable(true)
                .default_size([800.0, 800.0])
                .open(&mut self.whoami_open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(hnd) = &self.pp_hnd.read().unwrap().to_owned() {
                            ui.image(hnd.id(), hnd.size_vec2());
                        } else {
                            ui.spinner();
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Cem").size(24.0));
                            use egui::special_emojis::GITHUB;
                            ui.hyperlink_to(
                                RichText::new(format!("{GITHUB} github.com/bsg")).size(14.0),
                                "http://github.com/bsg",
                            );
                            ui.end_row();
                            ui.hyperlink_to(
                                RichText::new("✉ cem.saldirim@gmail.com").size(14.0),
                                "mailto://cem.saldirim@gmail.com",
                            );
                        });
                    });
                    ui.separator();
                    egui::CollapsingHeader::new("Latest Commits")
                        .default_open(true)
                        .show(ui, |ui| {
                            if let Some(commits) = &*self.latest_commits.read().unwrap() {
                                egui::Grid::new("commits_grid")
                                    .num_columns(2)
                                    .striped(true)
                                    .min_col_width(ui.available_width())
                                    .show(ui, |ui| {
                                        for commit in commits.commits.iter().take(20) {
                                            ui.horizontal(|ui| {
                                                ui.hyperlink_to(
                                                    format!("[{}]", commit.repo_name),
                                                    &commit.repo_url,
                                                );

                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(&commit.message_short);
                                                });
                                            });
                                            ui.end_row();
                                        }
                                    });
                            } else {
                                ui.spinner();
                            }
                        });
                });
        });
    }
}
