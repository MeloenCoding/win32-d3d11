use app::App;

mod app;
mod window;

fn main() {
    // create an app
    let mut app: App = app::App::create(1000, 750);

    // launch the app
    app.launch();

    // print the exit codes
    app.window.print_exit_codes();
}
