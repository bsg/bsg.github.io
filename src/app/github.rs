use std::sync::{Arc, RwLock};
use egui::ColorImage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Commit {
    message: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    #[serde(default)]
    commits: Option<Vec<Commit>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    id: u64,
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    repo: Repo,
    payload: Payload,
}

pub struct LatestCommit {
    pub message: String,
    pub message_short: String,
    pub repo_name: String,
    pub repo_url: String,
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
                                // TODO make this a util
                                let mut msg_short: String = commit
                                    .message
                                    .split('\n')
                                    .nth(0)
                                    .unwrap_or_default()
                                    .to_string();
                                if msg_short.len() > 50 { // FIXME :'(
                                    msg_short.truncate(47);
                                    msg_short.push_str("...");
                                }
                                LatestCommit {
                                    message: commit.message.clone(),
                                    message_short: msg_short,
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

pub async fn fetch_latest_commits(latest_commits: Arc<RwLock<Option<LatestCommits>>>) {
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

pub async fn fetch_pp(pp_hnd: Arc<RwLock<Option<egui::TextureHandle>>>, ctx: egui::Context) {
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