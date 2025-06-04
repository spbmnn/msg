use super::{message::ViewMessage, App, Message};
use iced::{event, mouse, window, Event, Subscription};

pub fn subscription(app: &App) -> Subscription<Message> {
    use iced::time;
    use std::time::Duration;

    let mut subs = vec![];

    subs.push(event::listen_with(|event, _, _| match event {
        Event::Window(window::Event::CloseRequested) => Some(Message::Exit),
        Event::Window(window::Event::Resized(size)) => Some(Message::View(
            ViewMessage::WindowResized(size.width.floor() as u32, size.height.floor() as u32),
        )),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) => {
            Some(Message::View(ViewMessage::Back))
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => {
            Some(Message::View(ViewMessage::Forward))
        }
        _ => None,
    }));

    if !app.search.thumbnail_queue.is_empty() {
        subs.push(time::every(Duration::from_millis(50)).map(|_| Message::Tick));
    }

    Subscription::batch(subs)
}
