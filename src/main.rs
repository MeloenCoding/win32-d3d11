use app::App;

mod app;
mod graphics;
mod window;

fn main() {
    // create an app
    let mut app: App = app::App::create();

    // launch the app
    app.launch();

    // print the exit codes
    app.window.print_exit_codes();
}
