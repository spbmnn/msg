use std::time::Duration;

use iced::{widget::{button, row, column, text, Slider}, Task, Element, Length};
use iced_video_player::{Video, VideoPlayer};
use url::Url;
use tracing::trace;

use crate::app::Message;

pub struct VideoPlayerWidget {
    pub video: Video,
    pub position: f64,
    pub dragging_cursor: bool,
}

#[derive(Debug, Clone)]
pub enum VideoPlayerMessage {
    TogglePause,
    ToggleLoop,
    Seek(f64),
    SeekRelease,
    EndOfStream,
    NewFrame
}

impl VideoPlayerWidget {
    pub fn new(video: Video) -> Self {
        VideoPlayerWidget {
            video: video,
            position: 0.0,
            dragging_cursor: false
        }
    }


    pub fn update(&mut self, message: VideoPlayerMessage) -> Task<Message> {
        match message {
            VideoPlayerMessage::TogglePause => {
                self.video.set_paused(!self.video.paused());
            }
            VideoPlayerMessage::ToggleLoop => {
                self.video.set_looping(!self.video.looping());
            }
            VideoPlayerMessage::Seek(time) => {
                self.dragging_cursor = true;
                self.video.set_paused(true);
                self.position = time;
            }
            VideoPlayerMessage::SeekRelease => {
                self.dragging_cursor = false;
                self.video
                    .seek(Duration::from_secs_f64(self.position))
                    .expect("couldn't seek");
                self.video.set_paused(false);
            }
            VideoPlayerMessage::EndOfStream => {
                trace!("end of video");
            }
            VideoPlayerMessage::NewFrame => {
                if !self.dragging_cursor {
                    self.position = self.video.position().as_secs_f64();
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, VideoPlayerMessage> {
        let controls = row![ // TODO: add loop controls/progress bar, slider
            button(if self.video.paused() {
                "⏵︎"
            } else {
                "⏸︎"
            })
            .on_press(VideoPlayerMessage::TogglePause),

        ];

        column![
            VideoPlayer::new(&self.video),
            controls
        ]
        .spacing(12)
        .into()
    }
}
