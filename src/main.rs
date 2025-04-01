mod database;

use iced::{
    window,
    futures::{SinkExt, Stream},
    stream,
    widget::{button, column, horizontal_space, row, text, Button, Column, Row, text_input},
    Element, Subscription, Task
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Pool};


fn main() -> iced::Result {

    iced::application(Catalog::title, Catalog::update, Catalog::view)
        .subscription(Catalog::subscriptions)
        .run()
}



#[derive(Debug, Clone)]
pub enum Message {
    Shutdown,
    DumpedConfig,
    WelcomePressed,
    SearchPressed,
    AddPressed,
    InitializationFailed(String),
    InitializationSuccessful(Config),
    InitializeInputChanged(String),
    InitializeSubmit,
    InitializeOpenFilePicker,
    CreateOpenDatabase(String),
    CreateDatabaseSuccess(Pool<Sqlite>),
    CreateDatabaseFailure(String),
    ClosedDatabase,
}

#[derive(Debug)]
pub enum Screen {
    Starting,
    InitializeEmpty(String),
    InitializeChoice(String),
    InitializeError(String),
    Welcome,
    Add,
    Search,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    database_paths: Vec<String>
}

#[derive(Debug)]
pub struct Catalog {
    screen: Screen,
    config: Config,
    current_database: Option<Pool<Sqlite>>,
}

impl Catalog {
    pub fn title(&self) -> String {
        String::from("Catalog")
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::Shutdown => {
                if let Some(pool) = self.current_database.take() {
                    Task::perform(database::close_database(pool), |x| x)
                } else {
                    Task::perform(Self::dump_config(self.config.clone()), |x| x)
                }
            }
            Message::DumpedConfig => {
                window::get_latest().and_then(window::close)
            }
            Message::WelcomePressed => {
                self.screen = Screen::Welcome;
                Task::none()
            }
            Message::AddPressed => {
                self.screen = Screen::Add;
                Task::none()
            }
            Message::SearchPressed => {
                self.screen = Screen::Search;
                Task::none()
            }
            Message::InitializationFailed(msg) => {
                //TODO: prevent the user doing anything
                self.screen = Screen::InitializeError(msg);
                Task::none()
            }
            Message::InitializationSuccessful(config) => {
                self.config = config;
                if self.config.database_paths.is_empty() {
                    self.screen = Screen::InitializeEmpty(String::new());

                    return Task::none();
                }
                self.screen = Screen::InitializeChoice(String::new());
                Task::none()
            }
            Message::InitializeInputChanged(input) => {
                match &mut self.screen {
                    Screen::InitializeEmpty(msg) => {
                        *msg = input
                    }
                    _ => {},
                }
                Task::none()
            },
            Message::InitializeSubmit => {
                match &mut self.screen {
                    Screen::InitializeEmpty(path) => {
                        self.config.database_paths.push(path.clone());
                        self.screen = Screen::InitializeChoice(String::new());
                        Task::none()
                    }
                    Screen::InitializeChoice(path) => {
                        self.config.database_paths.push(path.clone());
                        *path = String::new();
                        Task::none()
                    }
                    _ => {
                        Task::none()
                    }
                }
            }
            Message::InitializeOpenFilePicker => {
                use rfd::AsyncFileDialog;
                use directories::UserDirs;

                let future = async {

                    let user_dir = UserDirs::new().expect("user doesn't have a home directory");
                    let Some(path) = user_dir.home_dir()
                        .as_os_str()
                        .to_str() else {
                            return Message::InitializationFailed(String::from("Failed to select a path"))
                        };

                    let path = path.to_string();

                    let file = AsyncFileDialog::new()
                        .set_directory(path)
                        .add_filter("sqlite", &["sqlite"])
                        .save_file()
                        .await;

                    let Some(file) = file else {
                        return Message::InitializationFailed(String::from("Failed to select a path"))
                    };

                    let path = file.path()
                        .as_os_str()
                        .to_str()
                        .expect("Could not turn os_str into str")
                        .to_string();

                    Message::InitializeInputChanged(path)
                };

                Task::perform(future, |x| x)
            }
            Message::CreateOpenDatabase(path) => {
                Task::perform(database::create_database(path), |x| x)
            }
            Message::CreateDatabaseFailure(msg) => {
                Task::none()
            }
            Message::CreateDatabaseSuccess(database) => {
                self.current_database = Some(database);
                Task::none()
            }
            Message::ClosedDatabase => {
                let future = async {
                    Message::Shutdown
                };
                
                Task::perform(future, |x| x)
            }
        }
    }

    pub fn view(&self) -> Element<Message> {

        match self.screen {
            Screen::Starting => self.starting(),
            Screen::InitializeEmpty(_) => self.initialize_empty(),
            Screen::InitializeChoice(_) => self.initialize_choice(),
            Screen::InitializeError(_) => self.initialize_error(),
            Screen::Welcome => self.welcome(),
            Screen::Add => self.add(),
            Screen::Search => self.search(),
        }
    }

    fn close_events(&self) -> Subscription<Message> {
        window::close_events().map(|_| Message::Shutdown)
    }

    fn subscriptions(&self) -> Subscription<Message> {
        Subscription::batch([self.initialize_subscription(), self.close_events()])
    }

    fn initialize_subscription(&self) -> Subscription<Message> {
        Subscription::run(Self::initialize_subscription_worker)
    }

    fn initialize_subscription_worker() -> impl Stream<Item = Message> {
        use directories::ProjectDirs;
        use std::path::PathBuf;
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncReadExt;

        stream::channel(100, |mut output| async move {
            let project_dirs = if let Some(project_dirs) = ProjectDirs::from("org", "Ki11erRabbit", "Catalog") {
                project_dirs
            } else {
                match Catalog::setup_config_dir() {
                    Err(error) => {
                        let _ = output.send(Message::InitializationFailed(error.to_string() + ": setting config dir")).await;
                        return;
                    }
                    _ => {}
                }
                ProjectDirs::from("org", "Ki11erRabbit", "Catalog")
                    .expect("just created directory but somehow it doesn't exist")
            };

            let config_dir = project_dirs.config_dir();
            let mut catalog_toml = PathBuf::new();
            catalog_toml.push(config_dir);
            catalog_toml.push("databases.toml");

            let mut file = match OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(catalog_toml).await {
                    Err(error) => {
                        let _ = output.send(Message::InitializationFailed(error.to_string() + ": opening file")).await;
                        return;
                    }
                    Ok(file) => file,
                };

            let mut buf = Vec::new();
            
            match file.read_to_end(&mut buf).await {
                Err(err) => {
                    let _ = output.send(Message::InitializationFailed(err.to_string() + ": reading file")).await;
                    return;
                }
                _ => {}
            }
            drop(file);

            let config_contents = match String::from_utf8(buf) {
                Err(err) => {
                    let _ = output.send(Message::InitializationFailed(err.to_string())).await;
                    return;
                }
                Ok(config) => config,
            };

            let config: Config = match toml::from_str::<Config>(&config_contents) {
                Err(_) => {
                    let _ = output.send(Message::InitializationSuccessful(Config::default())).await;
                    return;
                }
                Ok(config) => config,
            };

            let _ = output.send(Message::InitializationSuccessful(config)).await;
        })
    }

    async fn dump_config(config: Config) -> Message {
        use directories::ProjectDirs;
        use std::path::PathBuf;
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;
        
        let project_dirs = if let Some(project_dirs) = ProjectDirs::from("org", "Ki11erRabbit", "Catalog") {
            project_dirs
        } else {
            match Catalog::setup_config_dir() {
                Err(error) => {
                    // TODO: send out notification that getting failed
                    return Message::DumpedConfig;
                }
                _ => {}
            }
            ProjectDirs::from("org", "Ki11erRabbit", "Catalog")
                .expect("just created directory but somehow it doesn't exist")
        };

        let config_dir = project_dirs.config_dir();
        let mut catalog_toml = PathBuf::new();
        catalog_toml.push(config_dir);
        catalog_toml.push("databases.toml");

        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(catalog_toml).await {
                Err(error) => {
                    // TODO: report error opening config file
                    return Message::DumpedConfig;
                }
                Ok(file) => file,
            };

        let config = toml::to_string(&config).expect("todo: handle config serialization");

        file.write_all(config.as_bytes())
            .await
            .expect("todo: handle failure writing to config");

        Message::DumpedConfig
    }

    fn setup_config_dir_common() -> std::io::Result<directories::UserDirs> {
        use std::io::{Error, ErrorKind};
        use directories::UserDirs;
        UserDirs::new()
            .ok_or(Error::new(ErrorKind::Other, String::from("Failure to get user directory")))
    }
    
    #[cfg(target_os = "windows")]
    fn setup_config_dir() -> std::io::Result<()> {
        use std::path::PathBuf;

        let user_dirs = Self::setup_config_dir_common()?;

        let home_path = user_dirs.home_dir();
        let mut config_path = PathBuf::new();
        config_path.push(home_path);
        config_path.push("AppData");
        config_path.push("Roaming");
        config_path.push("Ki11erRabbit");
        config_path.push("Catalog");
        config_path.push("config");
        
        std::fs::create_dir_all(config_path)
    }
    #[cfg(target_os = "macos")]
    fn setup_config_dir() -> std::io::Result<()> {
        use std::path::PathBuf;

        let user_dirs = Self::setup_config_dir_common()?;

        let home_path = user_dirs.home_dir();
        let mut config_path = PathBuf::new();
        config_path.push(home_path);
        config_path.push("Library");
        config_path.push("Application Support");
        config_path.push("org.Ki11erRabbit.Catalog");
        
        std::fs::create_dir_all(config_path)
    }
    #[cfg(target_os = "linux")]
    fn setup_config_dir() -> std::io::Result<()> {
        use std::path::PathBuf;

        let user_dirs = Self::setup_config_dir_common()?;

        let home_path = user_dirs.home_dir();
        let mut config_path = PathBuf::new();
        config_path.push(home_path);
        config_path.push(".config");
        config_path.push("catalog");
        
        std::fs::create_dir_all(config_path)
    }

    pub fn new() -> Self {
        Catalog {
            screen: Screen::Starting,
            config: Config::default(),
            current_database: None,
        }
    }

    fn get_controls(&self) -> Row<'_, Message> {
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

        controls
    }

    fn starting(&self) -> Element<Message> {
        let controls = self.get_controls();
        let contents = Self::container("Starting")
            .push(
                "we are waiting for things to start so please be patient"
            );

        let content: Element<_> = column![controls, contents]
            .into();
        content
    }

    fn initialize_empty(&self) -> Element<Message> {
        let Screen::InitializeEmpty(input_value) = &self.screen else {
            unreachable!("already checked for screen state");
        };

        let input = text_input("Enter absolute path for database", input_value)
            .id("new-database")
            .on_input(Message::InitializeInputChanged)
            .on_submit(Message::InitializeSubmit);


        let file_picker_button = padded_button("open file picker")
            .on_press(Message::InitializeOpenFilePicker);

        let submit_input_button = padded_button("Create Database")
            .on_press(Message::InitializeSubmit);

        let buttons = column![submit_input_button, file_picker_button];
        
        let contents = Self::container("Create a new database")
            .push(
                "Create a new database to begin"
            )
            .push(
                row![input, buttons]
                );


        let content: Element<_> = column![contents]
            .into();
        content
    }

    fn initialize_choice(&self) -> Element<Message> {
        let Screen::InitializeChoice(input_value) = &self.screen else {
            unreachable!("already checked for screen state");
        };

        let input = text_input("Enter absolute path for database", input_value)
            .id("new-database")
            .on_input(Message::InitializeInputChanged)
            .on_submit(Message::InitializeSubmit);


        let file_picker_button = padded_button("open file picker")
            .on_press(Message::InitializeOpenFilePicker);

        let submit_input_button = padded_button("Create Database")
            .on_press(Message::InitializeSubmit);

        let buttons = column![submit_input_button, file_picker_button];
        
        let contents = Self::container("Create a new database")
            .push(
                "Create a new database to begin"
            )
            .push(
                row![input, buttons]
                );

        let mut database_list = column![];
            
        for config in self.config.database_paths.iter() {
            use std::path::Path;
            let text = text(config.as_str());

            let button_text = if Path::new(config).exists() {
                "Open Database"
            } else {
                "Create Database"
            };

            let button = padded_button(button_text)
                .on_press(Message::CreateOpenDatabase(config.clone()));

            database_list = database_list.push(row![text, button]);
        }

        let content: Element<_> = column![database_list, contents]
            .into();
        content
    }

    fn initialize_error(&self) -> Element<Message> {
        let Screen::InitializeError(msg) = &self.screen else {
            panic!("calling initialize_error when not set to proper state");
        };
        let contents = Self::container("Initialization Error")
            .push(
                msg.as_str()
            );

        let content: Element<_> = column![contents]
            .into();
        content
    }

    fn welcome(&self) -> Element<Message> {
        let controls = self.get_controls();
        let contents = Self::container("Welcome!")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            );

        let content: Element<_> = column![controls, contents]
            .into();
        content
    }

    fn add(&self) -> Element<Message> {
        let controls = self.get_controls();
        let contents = Self::container("Add")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            );
        let content: Element<_> = column![controls, contents]
            .into();
        content
    }

    fn search(&self) -> Element<Message> {
        let controls = self.get_controls();
        let contents = Self::container("Search")
            .push(
                "This is a simple cataloging software, driven by sqlite"
            );
        let content: Element<_> = column![controls, contents]
            .into();
        content
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
