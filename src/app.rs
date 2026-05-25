use std::cell::RefCell;
use std::rc::Rc;

use super::views::article_details::ArticleDetails;
use super::views::articles_list::ArticlesList;
use super::views::comments::Comments;
use super::views::hubs_list::HubsList;
use super::views::settings::Settings;

use super::habr_client::article::ArticleData;
use super::habr_client::hub::Hub;

use super::view_stack::ViewStack;

pub struct MyApp {
    pub state: Rc<RefCell<HabreState>>,
    pub view_stack: ViewStack,
}

impl Default for MyApp {
    fn default() -> Self {
        let state = Rc::new(RefCell::new(HabreState::new()));
        let article_details = Rc::new(RefCell::new(ArticleDetails::new(state.clone())));
        let articles_list = Rc::new(RefCell::new(ArticlesList::new(state.clone())));
        let hubs_list = Rc::new(RefCell::new(HubsList::new(state.clone())));
        let mut view_stack = ViewStack::new();

        articles_list.borrow_mut().on_article_selected({
            move |_article_data, view_stack| {
                article_details.borrow_mut().load_data();
                view_stack.push(article_details.clone());
            }
        });

        articles_list.borrow_mut().on_comments_selected({
            let state = state.clone();
            move |article, view_stack| {
                state.borrow_mut().selected_article = Some(article);
                let comments = Rc::new(RefCell::new(Comments::new(state.clone())));
                comments.borrow_mut().load_comments();
                view_stack.push(comments);
            }
        });

        hubs_list.borrow_mut().on_hub_selected({
            let articles_list = articles_list.clone();
            move |_selected_hub, view_stack| {
                articles_list.borrow_mut().get_articles();
                view_stack.push(articles_list.clone());
            }
        });

        hubs_list.borrow_mut().get_hubs();
        view_stack.push(hubs_list.clone());

        Self { state, view_stack }
    }
}

#[derive(Debug)]
pub struct HabreState {
    pub selected_hub: Option<Hub>,
    pub selected_article: Option<ArticleData>,

    pub settings: Rc<RefCell<Settings>>,
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
