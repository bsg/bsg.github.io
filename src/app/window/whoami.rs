use egui::{Align2, Color32, Frame, RichText, Style, Ui};

use crate::BsgApp;

pub fn render(app: &mut BsgApp, ctx: &egui::Context, _ui: &mut Ui, is_portrait: bool) {
    egui::Window::new(RichText::from("$ whoami").color(Color32::GREEN))
        .pivot(if is_portrait {Align2::CENTER_TOP} else {Align2::CENTER_CENTER})
        .default_pos(if is_portrait {
            ctx.screen_rect().center_top()
        } else {
            ctx.screen_rect().center()
        })
        .resizable(!is_portrait)
        .default_size([800.0, 800.0])
        .vscroll(true)
        .collapsible(!is_portrait)
        .movable(!is_portrait)
        .title_bar(!is_portrait)
        .frame(if is_portrait {Frame::none()} else {Frame::window(&Style::default())})
        .open(&mut app.whoami_open)
        .show(ctx, |ui| {
            ui.add_space(if is_portrait {8.0} else {4.0});
            ui.horizontal(|ui| {
                if let Some(hnd) = &app.pp_hnd.read().unwrap().to_owned() {
                    ui.image(hnd.id(), hnd.size_vec2());
                } else {
                    ui.spinner();
                }
                ui.vertical(|ui| {
                    ui.label(RichText::new("Cem").text_style(egui::TextStyle::Heading));
                    use egui::special_emojis::GITHUB;
                    ui.hyperlink_to(
                        RichText::new(format!("{GITHUB} github.com/bsg")),
                        "http://github.com/bsg",
                    );
                    ui.end_row();
                    ui.hyperlink_to(
                        RichText::new("✉ cem.saldirim@gmail.com"),
                        "mailto://cem.saldirim@gmail.com",
                    );
                });
            });

            ui.separator();

            egui::CollapsingHeader::new(RichText::from("[Bio]").color(Color32::from_rgb(232, 120, 35)))
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.")
                });

            ui.separator();

            egui::CollapsingHeader::new(RichText::from("[Latest Commits]").color(Color32::from_rgb(232, 120, 35)))
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(commits) = &*app.latest_commits.read().unwrap() {
                        egui::Grid::new("commits_grid")
                            .num_columns(2)
                            .striped(true)
                            .min_col_width(ui.available_width())
                            .show(ui, |ui| {
                                for commit in commits.commits.iter().take(5) {
                                    ui.horizontal(|ui| {
                                        ui.hyperlink_to(
                                            format!("[{}]", commit.repo_name),
                                            &commit.repo_url,
                                        );
                                        ui.label(&commit.message_short);
                                    });
                                    ui.end_row();
                                }
                            });
                    } else {
                        ui.spinner();
                    }
                });
        });
}
