use iced_winit::runtime::Program;

mod ctrs;

fn main() -> iced::Result {
    env_logger::init();

    let app = iced::application(
        "CTRS - Rust CT Viewer",
        ctrs::CTRS::update,
        ctrs::CTRS::view
    )
    .settings(iced::Settings {
        id: Some("ctrs".into()),
        ..Default::default()
    })
    .subscription(ctrs::CTRS::subscription);
    app.run()
}
