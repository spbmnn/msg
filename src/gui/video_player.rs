use std::fmt;
use std::time::Duration;

use iced::{
    widget::{button, column, row, slider, text, text::Shaping},
    Alignment, Element, Task,
};
use iced_video_player::{Video, VideoPlayer};
use tracing::{error, trace};

use crate::app::Message;

const PAUSE_SYMBOL: &str = "\u{23F8}\u{FE0E}";
const PLAY_SYMBOL: &str = "\u{23F5}\u{FE0E}";

pub struct VideoPlayerWidget {
    pub video: Video,
    pub position: f64,
    pub playing: bool,
    pub dragging_cursor: bool,
}

impl fmt::Debug for VideoPlayerWidget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VideoPlayerWidget")
            .field("video", &"[Video omitted]")
            .field("position", &self.position)
            .field("dragging_cursor", &self.dragging_cursor)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum VideoPlayerMessage {
    TogglePause,
    //ToggleLoop,
    Seek(f64),
    SeekRelease,
    EndOfStream,
    NewFrame,
}

impl VideoPlayerWidget {
    pub fn new(video: Video) -> Self {
        VideoPlayerWidget {
            video,
            position: 0.0,
            playing: true,
            dragging_cursor: false,
        }
    }

    pub fn update(&mut self, message: VideoPlayerMessage) -> Task<Message> {
        match message {
            VideoPlayerMessage::TogglePause => {
                self.video.set_paused(!self.video.paused());
                self.playing = !self.video.paused();
            }
            //VideoPlayerMessage::ToggleLoop => {
            //    self.video.set_looping(!self.video.looping());
            //}
            VideoPlayerMessage::Seek(time) => {
                self.dragging_cursor = true;
                self.video.set_paused(true);
                self.playing = false;
                self.position = time;
            }
            VideoPlayerMessage::SeekRelease => {
                self.dragging_cursor = false;
                let video_result = self
                    .video
                    .seek(Duration::from_secs_f64(self.position), false);
                match video_result.err() {
                    None => {}
                    Some(err) => error!("Seeking failed: {err}"),
                }
                self.playing = true;
                self.video.set_paused(false);
            }
            VideoPlayerMessage::EndOfStream => {
                trace!("end of video");
                self.playing = false;
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
        let duration = self.video.duration().as_secs_f64();

        let controls = row![
            button(if self.playing {
                text(PAUSE_SYMBOL).shaping(Shaping::Advanced)
            } else {
                text(PLAY_SYMBOL).shaping(Shaping::Advanced)
            })
            .on_press(VideoPlayerMessage::TogglePause),
            slider(0.0..=duration, self.position, VideoPlayerMessage::Seek)
                .on_release(VideoPlayerMessage::SeekRelease),
            text(timestamp(self.position)).font(iced::font::Font::MONOSPACE)
        ]
        .align_y(Alignment::Center);

        column![
            VideoPlayer::new(&self.video)
                .on_new_frame(VideoPlayerMessage::NewFrame)
                .on_end_of_stream(VideoPlayerMessage::EndOfStream),
            controls
        ]
        .spacing(12)
        .into()
    }
}

fn timestamp(t: f64) -> String {
    let time: usize = t.round() as usize;
    let minutes = time / 60;
    let seconds = time % 60;

    format!("{}:{:02}", minutes, seconds)
}
