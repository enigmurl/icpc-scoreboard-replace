mod scoreboard;
mod parser;
mod views;

use quarve::prelude::*;
use crate::views::viewer;

struct App;
struct MainWindow;
pub(crate) struct Env(StandardConstEnv, StandardVarEnv);

// mainly boilerplate
impl ApplicationProvider for App {
    // used for calculating certain paths
    fn name(&self) -> &str {
        "Quarve Example"
    }

    fn will_spawn(&self, app: &quarve::core::Application, s: MSlock) {
        // init code goes here
        app.spawn_window(MainWindow, s);
    }
}

// The main code where you specify a view hierarchy
impl WindowProvider for MainWindow {
    type Environment = Env;

    fn title(&self, _env: &<Env as Environment>::Const, _s: MSlock) -> impl Signal<Target=String> {
        FixedSignal::new("Quarve App".into())
    }

    fn size(&self, _env: &<Env as Environment>::Const, _s: MSlock) -> (Size, Size, Size) {
        (
            // min
            Size::new(400.0, 400.0),
            // intrinsic
            Size::new(1000.0, 800.0),
            // max
            Size::new(2400.0, 2000.0)
        )
    }

    fn root(&self, env: &<Env as Environment>::Const, s: MSlock) -> impl ViewProvider<Env, DownContext=()> {
        viewer()
            .into_view_provider(env, s)
    }

    fn menu(&self, env: &<Self::Environment as Environment>::Const, s: MSlock) -> WindowMenu {
        WindowMenu::standard(
            env,
            Menu::new("File"),
            Menu::new("Edit"),
            Menu::new("View"),
            Menu::new("Help"),
            s
        )
    }
}

// Boilerplate; mainly necessary for advanced projects
impl Environment for Env {
    type Const = StandardConstEnv;
    type Variable = StandardVarEnv;

    fn root_environment() -> Self {
        Env(StandardConstEnv::new(), StandardVarEnv::new())
    }

    fn const_env(&self) -> &Self::Const {
        &self.0
    }

    fn variable_env(&self) -> &Self::Variable { &self.1 }

    fn variable_env_mut(&mut self) -> &mut Self::Variable { &mut self.1 }
}

// boilerplate trait alias for IntoViewProvider<Env, DownContext=(), UpContext=()>
// which is referenced a lot but tedious to type
// Note that because of Rust semantics, when creating your own IVP
// you should implement IntoViewProvider<Env> (NOT IVP)
pub(crate) trait IVP: IntoViewProvider<Env, DownContext=(), UpContext=()> { }

impl<I> IVP for I where I: IntoViewProvider<Env, DownContext=(), UpContext=()> { }


#[tokio::main]
async fn main() {
    quarve::core::launch(App);
}

