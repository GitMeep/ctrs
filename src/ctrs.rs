mod scan;
mod scene;

use std::{f32::consts::PI, io, sync::Arc};

use iced::{alignment::Vertical, widget::{button, text_input, column, container, row, shader, text}, window, Alignment::Center, Element, Length::{Fill, FillPortion}, Subscription, Task, Theme};
use iced_winit::runtime::Program;
use scan::CtScan;
use rfd::AsyncFileDialog;
use scene::Scene;

#[derive(Debug, Clone)]
pub enum ScanLoadError {
    NonePicked,
    FileLoadError(Arc<io::Error>),
}

type ScanLoadResult = Result<Arc<CtScan>, ScanLoadError>;

pub struct CTRS {
    scan: Option<Arc<CtScan>>,
    scene: Option<Scene>,
    status_message: String,
    threshold: f32,
}

impl Default for CTRS {
    fn default() -> Self {
        Self {
            scan: Default::default(),
            scene: Default::default(),
            status_message: String::from("Please open a scan"),
            threshold: 0.71,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenPressed,
    HelpPressed,
    ScreenshotPressed,
    ScanLoaded(ScanLoadResult),
    ThresholdEdited(String),
    Tick,
}

impl Program for CTRS {
    type Theme = Theme;
    type Message = Message;
    type Renderer = iced_wgpu::Renderer;

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenPressed => {
                self.status_message = String::from("Loading scan...");

                Task::perform(load_scan(), Message::ScanLoaded)
            },
            Message::HelpPressed => Task::none(),
            Message::ScreenshotPressed => Task::none(),
            Message::ScanLoaded(Ok(scan)) => {
                self.status_message = format!("Scan {} loaded", scan.name);
                self.scene = Some(Scene::new(scan.clone(), self.threshold));
                self.scan = Some(scan);

                log::info!("Updated scan");
                Task::none()
            },
            Message::ScanLoaded(Err(err)) => {// TODO: notify user that error happened
                log::error!("Error loading scan: {err:?}");
                self.status_message = match err {
                    ScanLoadError::NonePicked => String::from("Please pick a file"),
                    ScanLoadError::FileLoadError(err) => format!("{err}"),
                };

                self.scan = None;

                Task::none()
            },
            Message::ThresholdEdited(str) => {
                if let Ok(new) = str.parse::<f32>() {
                    self.threshold = new;
                    
                    if let Some(scene) = &mut self.scene {
                        scene.set_threshold(new);
                    }
                }

                Task::none()
            },
            Message::Tick => {
                if let Some(scene) = &mut self.scene {
                    scene.rotate(PI/16.);
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let status_message = text(self.status_message.clone());

        let top_bar = row![
            button("Open").on_press(Message::OpenPressed),
            button("Help").on_press(Message::HelpPressed),
            status_message,
        ].spacing(5).padding(2).align_y(Vertical::Center).height(40).width(Fill);

        let shader_container = container({
            let element: Element<'_, Self::Message, Self::Theme, Self::Renderer> = match &self.scene {
                Some(scene) => shader(scene)
                    .width(Fill)
                    .height(Fill)
                    .into(),
                None => text("No scan loaded").into(),
            };

            element
        })
        .align_x(Center)
        .align_y(Center)
        .width(FillPortion(80))
        .height(Fill);

        let threshold_input = row![
            text("Threshold: "),
            text_input("Enter threshold", &self.threshold.to_string())
                .on_input(Message::ThresholdEdited)
                .width(Fill)
        ]
        .width(Fill)
        .align_y(Center);

        let sidebar = container(
            column![
                button(
                    container("Screenshot")
                        .width(Fill)
                        .align_x(Center)
                    )
                    .on_press(Message::ScreenshotPressed).width(Fill),
                threshold_input,
            ]
            .spacing(5)
        )
        .style(container::dark)
        .width(FillPortion(20))
        .height(Fill)
        .padding(5);

        let work_area = row![
            shader_container,
            sidebar,
        ].width(Fill).height(Fill);

        column![
            top_bar,
            work_area
        ].height(Fill).width(Fill)
        
        .into()
    }
}

impl CTRS {
    pub fn subscription(&self) -> Subscription<Message> {
        window::frames().map(|_| Message::Tick )
    }
}

async fn load_scan() -> ScanLoadResult {
    let handle = AsyncFileDialog::new()
        .add_filter("Scan description file", &["json"])
        .set_title("Pick scan")
        .pick_file()
        .await;

    log::info!("Loading scan: {:?}", handle.as_ref());

    match handle {
        Some(path) => CtScan::from_file(path.path())
            .await
            .map_err(|err| ScanLoadError::FileLoadError(Arc::new(err)))
            .map(Arc::new),
        None => Err(ScanLoadError::NonePicked),
    }
}
