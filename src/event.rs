pub trait EventHandler<F> {
    fn on_finish(&self, file: &F) {}
    fn on_write(&self, bytes: &[u8]) {}
    fn on_content_length(&self, length: &u64) {}
}

pub trait HandlerExt<F> {
    fn new<H: EventHandler<F> + 'static>(handler: H) -> Self;
    fn add_handler<H: EventHandler<F> + 'static>(self, handler: H) -> Self;
    fn on_finish(&self, file: &F);
    fn on_write(&self, bytes: &[u8]);
    fn on_content_length(&self, length: &u64);
}

pub type Handlers<F> = Vec<Box<dyn EventHandler<F>>>;

impl<F> HandlerExt<F> for Handlers<F> {
    fn new<H: EventHandler<F> + 'static>(handler: H) -> Self {
        vec![Box::new(handler)]
    }

    fn add_handler<H: EventHandler<F> + 'static>(mut self, handler: H) -> Self {
        self.push(Box::new(handler));
        self
    }

    fn on_finish(&self, file: &F) {
        self.iter().for_each(|handler| handler.on_finish(file));
    }

    fn on_write(&self, bytes: &[u8]) {
        self.iter().for_each(|handler| handler.on_write(&bytes));
    }

    fn on_content_length(&self, length: &u64) {
        self.iter()
            .for_each(|handler| handler.on_content_length(&length));
    }
}
