use std::sync::{Arc, RwLock};

use egui::{Align2, ColorImage, RichText};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commit {
    message: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    #[serde(default)]
    commits: Option<Vec<Commit>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    id: u64,
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    repo: Repo,
    payload: Payload,
}

pub struct LatestCommit {
    message: String,
    repo_name: String,
    repo_url: String,
}

pub struct LatestCommits {
    pub commits: Vec<LatestCommit>,
}

impl LatestCommits {
    fn from(events: Vec<Event>) -> Self {
        LatestCommits {
            commits: events
                .iter()
                .filter(|event| event.event_type == "PushEvent")
                .map(|event| {
                    event.payload.commits.as_ref().map(|commits| {
                        commits
                            .iter()
                            .map(|commit| {
                                let repo_name = event.repo.url.split("repos/").nth(1).unwrap();
                                LatestCommit {
                                    message: commit.message.clone(),
                                    repo_url: format!("https://github.com/{}", repo_name),
                                    repo_name: repo_name.to_string(),
                                }
                            })
                            .collect::<Vec<LatestCommit>>()
                    })
                })
                .flatten()
                .flatten()
                .collect(),
        }
    }
}

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
    async fn fetch_pp(pp_hnd: Arc<RwLock<Option<egui::TextureHandle>>>, ctx: egui::Context) {
        log::info!("Fetching pp...");
        let img_bytes = reqwest::get("https://avatars.githubusercontent.com/u/227873")
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        let mut image = image::load_from_memory(&img_bytes).unwrap();
        image = image.resize_exact(120, 120, image::imageops::Lanczos3);
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        let _ = pp_hnd.write().unwrap().insert(ctx.load_texture(
            "profile_pic",
            color_image,
            Default::default(),
        ));
    }

    async fn fetch_latest_commits(latest_commits: Arc<RwLock<Option<LatestCommits>>>) {
        log::info!("Fetching latest commits...");
        match reqwest::Client::new()
            .get("https://api.github.com/users/bsg/events/public")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
        {
            Ok(res) => {
                let latest = LatestCommits::from(res.json::<Vec<Event>>().await.unwrap());
                _ = latest_commits.write().unwrap().insert(latest);
            }
            Err(_) => todo!(),
        }
    }

    fn fetch_initial_data(me: &BsgApp, cc: &eframe::CreationContext<'_>) {
        let pp_hnd_cloned = me.pp_hnd.clone();
        let ctx_cloned = cc.egui_ctx.clone();
        if me.pp_hnd.read().unwrap().is_none() {
            wasm_bindgen_futures::spawn_local(async {
                Self::fetch_pp(pp_hnd_cloned, ctx_cloned).await
            });
        }

        let latest_commits_cloned = me.latest_commits.clone();
        if me.latest_commits.read().unwrap().is_none() {
            wasm_bindgen_futures::spawn_local(async {
                Self::fetch_latest_commits(latest_commits_cloned).await
            });
        }
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "roboto-regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/Roboto-Regular.ttf")),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "roboto-regular".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("roboto-regular".to_owned());

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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                .default_pos(ctx.screen_rect().center())
                .resizable(false)
                .fixed_size([400.0, 400.0])
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
                            let commits = &*self.latest_commits.read().unwrap();
                            if let Some(commits) = commits {
                                egui::Grid::new("my_grid")
                                    .num_columns(2)
                                    .spacing([40.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for commit in commits.commits.iter().take(10) {
                                            ui.horizontal(|ui| {
                                                ui.hyperlink_to(
                                                    format!("[{}]", commit.repo_name),
                                                    &commit.repo_url,
                                                );
                                                ui.label(&commit.message);
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
