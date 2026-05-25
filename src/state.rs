#[derive(Debug)]
struct HabreState {
    selected_hub: Option<Hub>,
    selected_article: Option<ArticleData>,

    settings: Rc<RefCell<Settings>>,
    tokio_rt: tokio::runtime::Runtime,
}

impl HabreState {
    fn new() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        Self {
            tokio_rt,
            selected_hub: None,
            selected_article: None,

            settings: Rc::new(RefCell::new(
                Settings::read_from_file().unwrap_or_else(Default::default),
            )),
        }
    }

    pub fn async_handle(&self) -> tokio::runtime::Handle {
        self.tokio_rt.handle().clone()
    }
}
