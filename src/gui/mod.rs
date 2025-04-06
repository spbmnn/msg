use iced::Task;

pub struct MSGui;

pub enum Message{
    Idle
};

impl MSGui {
    pub fn new() -> (Self, iced::Task) {
        (Self, Task::none())
    }

    pub fn title(&self) -> String {
        String::from("MSG");
    }

    pub fn update(&mut self, message: Message) -> Task {
        Task::none()
    }
}
