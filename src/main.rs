use frontend::context::data::Context as AppContext;
use frontend::framework::Frontend;
use frontend::screens::launcher::LauncherScreen;
use frontend::eframe;

fn main() -> eframe::Result {
    eframe::run_native(
        "goymusic.cum",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            let app = AppContext::default();
            let mut frontend = Frontend::new(cc, app);
            frontend.window_mut(0).unwrap().push(LauncherScreen::default());
            Ok(Box::new(frontend))
        }),
    )
}