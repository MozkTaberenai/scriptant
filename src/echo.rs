pub trait EchoTrait {
    fn begin_echo(&self) -> std::fmt::Result;
    fn put_echo_item(&self, item: impl std::fmt::Display) -> std::fmt::Result;
    fn end_echo(&self) -> std::fmt::Result;
}

pub enum Echo {
    Null,
    Head,
    Tail,
}

impl Default for Echo {
    fn default() -> Self {
        match std::env::var_os("NO_ECHO").is_some() {
            true => Self::Null,
            false => Self::Head,
        }
    }
}

impl Echo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quiet(&mut self) -> &mut Self {
        *self = Self::Null;
        self
    }

    pub fn put(&mut self, arg: impl std::fmt::Display) -> &mut Self {
        match self {
            Self::Null => {}
            Self::Head => {
                print!("{arg}");
                *self = Self::Tail;
            }
            Self::Tail => print!(" {arg}"),
        }
        self
    }

    pub fn end(self) {
        match self {
            Self::Null => {}
            _ => println!(),
        }
    }
}

#[macro_export]
macro_rules! echo {
    () => {
        println!();
    };
    ($($arg:expr),* $(,)?) => {
        let mut echo = $crate::echo::Echo::new();
        $(echo.put($arg);)*
        echo.end();
    };
}
