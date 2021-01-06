use components::{
    Component,
    Init,
    Viewable,
    Editor,
    editor,
};
pub mod chart;
use seed::{
    prelude::*,
    *,
};
use std::collections::HashMap;
use shared::{
    subscriptions::{
        Response,
        PriceSubscription,
        Route,
    },
};
use database_table::{
    Entry,
    RemoteTable,
};
#[allow(unused)]
use tracing::{
    instrument,
    debug,
    info,
};
use rql::*;

#[derive(Debug)]
pub struct Subscriptions {
    subscriptions: HashMap<Id<PriceSubscription>, chart::SubscriptionChart>,
    editor: Option<Editor<PriceSubscription>>,
    server_msg_sub: SubHandle,
    update_list: bool,
}
impl Subscriptions {
    fn subscription_editor(&self) -> Node<Msg> {
        if let Some(editor) = &self.editor {
            editor.view().map_msg(Msg::Editor)
        } else {
            self.add_subscription_button()
        }
    }
    fn add_subscription_button(&self) -> Node<Msg> {
        button![
            ev(Ev::Click, |_| Msg::OpenEditor),
            "Add Subscription", 
        ]
    }
}
impl Init<Route> for Subscriptions {
    fn init(_: Route, orders: &mut impl Orders<Msg>) -> Self {
        // TODO add components for list and entry
        orders.send_msg(Msg::GetList);
        Self {
            server_msg_sub: orders.subscribe_with_handle(|msg: Response| {
                debug!("Received Subscription Response");
                match msg {
                    Response::SubscriptionList(list) => Some(Msg::SetList(list)),
                    Response::PriceHistory(id, history) => Some(Msg::Subscription(id, chart::Msg::AppendCandles(history.candles))),
                    _ => None,
                }
            }),
            subscriptions: HashMap::new(),
            editor: Some(Editor::default()),
            update_list: true,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Msg {
    GetList,
    OpenEditor,
    Editor(<Editor<PriceSubscription> as Component>::Msg),
    SetList(Vec<Entry<PriceSubscription>>),
    Subscription(Id<PriceSubscription>, chart::Msg),
}
impl Component for Subscriptions {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        debug!("subscriptions::Msg");
        //self.update_list = false;
        match msg {
            Msg::OpenEditor => {
                debug!("Msg::OpenEditor...");
                self.editor = Some(Editor::default());
            }
            Msg::GetList => {
                debug!("Msg::GetList...");
                orders.perform_cmd(async move {
                    debug!("Calling command..");
                    PriceSubscription::get_all().await
                        .map(|list: Vec<Entry<PriceSubscription>>| {
                            Msg::SetList(list)
                        }).expect("Failed to get SubscriptionList")
                });
            },
            Msg::SetList(list) => {
                debug!("Setting SubscriptionList");
                self.subscriptions = list.into_iter().map(|entry| {
                    let id = entry.id.clone();
                    (
                        id.clone(),
                        chart::SubscriptionChart::init(
                            id.clone(),
                            &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg))
                        )
                    )
                })
                .collect();
                self.update_list = true;
            },
            Msg::Editor(msg) => {
                debug!("Editor Msg {:#?}", msg);
                if let Some(ed) = &mut self.editor {
                    let new = match msg {
                        editor::Msg::Cancel => Some(None),
                        editor::Msg::Submit => Some(None),
                        _ => None,
                    };
                    ed.update(msg, &mut orders.proxy(Msg::Editor));
                    if let Some(new) = new {
                        self.editor = new;
                    }
                }
            },
            Msg::Subscription(id, msg) => {
                if let Some(subscription) = self.subscriptions.get_mut(&id) {
                    subscription.update(msg, &mut orders.proxy(move |msg| Msg::Subscription(id.clone(), msg)));
                } else {
                    error!("Subscription {} not found!", id);
                }
                self.update_list = true;
            },
        }
    }
}
impl Viewable for Subscriptions {
    fn view(&self) -> Node<Msg> {
        let _list = if self.update_list {
            let list = self.subscriptions.iter()
                    .map(move |(id, chart)| {
                        (id.clone(), chart.view())
                    });
            div![
                list.map(move |(id, chart)| {
                    chart.map_msg(move |msg| Msg::Subscription(id.clone(), msg))
                })
            ]
        } else {
            div![Node::NoChange]
        };
        div![
            self.subscription_editor(),
            //list,
        ]
    }
}
