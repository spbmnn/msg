use crate::app::message::{
    DetailMessage, FollowedMessage, MediaMessage, Message, PostMessage, SearchMessage,
    SettingsMessage, StartupMessage, ViewMessage,
};
use crate::App;
use iced::Task;

pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Startup(msg) => update_startup(StartupMessage),
        Message::Search(msg) => update_search(SearchMessage),
        Message::Post(msg) => update_post(PostMessage),
        Message::Media(msg) => update_media(MediaMessage),
        Message::Detail(msg) => update_detail(DetailMessage),
        Message::Settings(msg) => update_settings(SettingsMessage),
        Message::Followed(msg) => update_followed(FollowedMessage),
        Message::View(msg) => update_view(ViewMessage),
    }
}

fn update_startup(app: &mut App, msg: StartupMessage) -> Task<Message> {
    Task::none();
}

fn update_settings(app: &mut App, msg: SettingsMessage) -> Task<Message> {
    let settings = &mut app.settings;
    match msg {
        SettingsMessage::UsernameChanged(username) => {
            app.settings_username = username;
        }
        SettingsMessage::ApiKeyChanged(key) => {
            app.settings_api_key = key;
        }
        SettingsMessage::BlacklistEdited(action) => {
            if let Some(state) = app.get_loaded_state_mut() {
                state.settings_blacklist_content.borrow_mut().update(action);
            }
        }
        SettingsMessage::Save => {
            app.save_settings();
        }
    }
    Task::none()
}
