use crate::app::message::ViewMessage;
use crate::app::state::ViewMode;
use crate::app::App;
use crate::app::Message;
use crate::core::model::Rating;
use iced::widget::Row;
use iced::Alignment;
use iced::{
    widget::{button, row, text, Column},
    Element,
};

pub fn detail_bar(app: &App) -> Row<'_, Message> {
    let mut bar =
        row![button("back").on_press(Message::View(ViewMessage::Back))].align_y(Alignment::Center);

    if app.selected_post.is_none() {
        return bar.into();
    }

    let post = app.store.get_post(app.selected_post.unwrap()).unwrap();
    bar = bar.push(text(format!("post #{}", post.id)).size(20));
    bar = bar.push(text(match post.rating {
        Rating::Safe => "Rating: Safe",
        Rating::Questionable => "Rating: Questionable",
        Rating::Explicit => "Rating: Explicit",
    }));
    bar = bar.push(text(format!("Score: {}", post.score.total)));

    bar
}

pub fn render_detail(app: &App) -> Element<'_, Message> {
    if let Some(selected_post) = app.selected_post {
        let post = app.store.get_post(selected_post).unwrap();
        crate::gui::detail_view::render_detail(&post, &app.store, &app.video_player)
    } else {
        Column::new().push(text("no post selected!")).into()
    }
}
