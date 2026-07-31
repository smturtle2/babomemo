use arboard::{Clipboard, Error};

pub(crate) struct SystemClipboard {
    backend: Option<Clipboard>,
}

impl SystemClipboard {
    pub(crate) fn new() -> Self {
        Self { backend: None }
    }

    pub(crate) fn write_text(&mut self, text: String) -> Result<(), Error> {
        self.with_backend(|backend| backend.set_text(text))
    }

    pub(crate) fn read_text(&mut self) -> Result<String, Error> {
        self.with_backend(Clipboard::get_text)
    }

    fn with_backend<T>(
        &mut self,
        operation: impl FnOnce(&mut Clipboard) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let mut backend = match self.backend.take() {
            Some(backend) => backend,
            None => Clipboard::new()?,
        };
        let result = operation(&mut backend);
        self.backend = Some(backend);
        result
    }
}
