use crate::app::App;
use crate::app::Message;
use iced::{
    widget::{Column, Row, Text},
    Element,
};

pub fn render_detail(app: &App) -> Element<'_, Message> {
    if let Some(selected_post) = app.selected_post {
        let post = app.store.get_post(selected_post).unwrap();
        crate::gui::detail_view::render_detail(&post, &app.store, &app.video_player)
    } else {
        Column::new().push(Text::new("no post selected!")).into()
    }
}
