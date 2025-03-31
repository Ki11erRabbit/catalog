use iced::{widget::{button, Button, text, row, column, horizontal_space, Column}, Element, };


fn main() -> iced::Result {

    iced::application(Catalog::title, Catalog::update, Catalog::view)
        .run()
}


#[derive(Debug, Clone)]
pub enum Message {
    WelcomePressed,
    SearchPressed,
    AddPressed,
}

#[derive(Debug)]
pub enum Screen {
    Welcome,
    Add,
    Search,
}

#[derive(Debug)]
pub struct Catalog {
    screen: Screen,
}

impl Catalog {
    pub fn title(&self) -> String {
        String::from("Catalog")
    }

    pub fn update(&mut self, event: Message) {
        match event {
            Message::WelcomePressed => {
                self.screen = Screen::Welcome;
            }
            Message::AddPressed => {
                self.screen = Screen::Add;
            }
            Message::SearchPressed => {
                self.screen = Screen::Search;
            }

        }
    }

    pub fn view(&self) -> Element<Message> {
        let controls =
            row![
                horizontal_space(),
                padded_button("Welcome")
                    .on_press(Message::WelcomePressed),
                horizontal_space(),
                padded_button("Add")
                    .on_press(Message::AddPressed),
                horizontal_space(),
                padded_button("Search")
                    .on_press(Message::SearchPressed),
                horizontal_space(),
            ];

        let contents = match self.screen {
            Screen::Welcome => self.welcome(),
            Screen::Add => self.add(),
            Screen::Search => self.search(),
        };

        let content: Element<_> = column![controls, contents]
            .into();


        content
    }

    pub fn new() -> Self {
        Catalog {
            screen: Screen::Welcome
        }
    }

    fn welcome(&self) -> Column<'_, Message> {
        Self::container("Welcome!")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            )
    }

    fn add(&self) -> Column<'_, Message> {
        Self::container("Add")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            )
    }

    fn search(&self) -> Column<'_, Message> {
        Self::container("Search")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            )
    }

    fn container(title: &str) -> Column<'_, Message> {
        column![text(title).size(50)].spacing(20)
    }
}


impl Default for Catalog {
    fn default() -> Catalog {
        Catalog::new()
    }
}


fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}
