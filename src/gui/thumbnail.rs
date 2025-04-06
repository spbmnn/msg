use crate::api::schema::Post;
use iced::{
    widget::{image, text},
    Element,
};

impl Post {
    fn view_thumbnail(&self) -> Element {
        column![
            image(self.preview.url).width(150).height(150).into(),
            text(format!("#{}", self.id))
        ]
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
    }
}
