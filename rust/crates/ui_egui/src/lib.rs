use eframe::App;
use eframe::NativeOptions;
use eframe::egui::CentralPanel;
use eframe::egui::Context;
use eframe::egui::Hyperlink;
use eframe::egui::Label;
use eframe::egui::ScrollArea;
use eframe::egui::Separator;
use eframe::egui::TopBottomPanel;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use tracing::info;

pub async fn egui_main() -> eyre::Result<()> {
    info!("Hello from egui!");
    let mut native_options = NativeOptions::default();
    native_options.viewport.inner_size = Some(Vec2::new(540., 960.));
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )?;
    Ok(())
}

pub struct MyEguiApp {}
impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self {}
    }
}
impl App for MyEguiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            render_header(ui);
            ScrollArea::both().show(ui, |ui| {
                for i in 0..100 {
                    if i % 2 == 0 {
                        ui.horizontal(|ui| {
                            ui.label("Item");
                            ui.label(format!("{}", i));
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Another item");
                            ui.label(format!("{}", i));
                        });
                    }
                }
            });
            render_footer(ctx);
        });
    }
}

fn render_footer(ctx: &Context) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API source: newsapi.org"));
            ui.add(
                Hyperlink::from_label_and_url("Made with egui", "https://github.com/emilk/egui"), // .text_style(TextStyle::Monospace)
            );
            // ui.style_mut()
            ui.add(
                Hyperlink::from_label_and_url(
                    "creativcoder/MyApp",
                    "https://github.com/creativcoder/MyApp", // todo: change this
                ), // .text_style(TextStyle::Monospace)
            );
            ui.add_space(10.);
        })
    });
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("MyApp");
    });
    ui.add_space(5.0);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);
}
