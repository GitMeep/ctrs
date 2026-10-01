use crate::ctrs::CTRS;

mod ctrs;

fn main() -> iced::Result {
    env_logger::init();

    let app = iced::application(
        CTRS::default,
        CTRS::update,
        CTRS::view
    )
    .settings(iced::Settings {
        id: Some("ctrs".into()),
        ..Default::default()
    })
    .subscription(CTRS::subscription);
    app.run()
}
