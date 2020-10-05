use crate::{
    entry,
    preview,
    Component,
    Init,
    Viewable,
};
use database_table::{
    Entry,
    TableItem,
};
use seed::{
    prelude::*,
    *,
};
use std::fmt::Debug;
use std::result::Result;

#[derive(Debug, Clone, Default)]
pub struct Model<T: TableItem + Component + std::fmt::Debug> {
    previews: Vec<preview::Model<T>>,
}
impl<T: Component + TableItem + Default + std::fmt::Debug + std::fmt::Debug + std::fmt::Debug>
    Init<Msg<T>> for Model<T>
{
    fn init(msg: Msg<T>, orders: &mut impl Orders<Msg<T>>) -> Model<T> {
        orders.send_msg(msg);
        Model::default()
    }
}
impl<T: Component + TableItem + std::fmt::Debug + std::fmt::Debug> From<Vec<Entry<T>>>
    for Model<T>
{
    fn from(entries: Vec<Entry<T>>) -> Self {
        Self {
            previews: init_previews(entries),
        }
    }
}
fn init_previews<T: Component + TableItem + Debug>(
    entries: Vec<Entry<T>>,
) -> Vec<preview::Model<T>> {
    entries.iter().cloned().map(preview::Model::from).collect()
}
#[derive(Clone, Debug)]
pub enum Msg<T: Component + TableItem + std::fmt::Debug> {
    GetAll,
    All(Result<Vec<Entry<T>>, String>),

    Preview(usize, preview::Msg<T>),
}
impl<T: Component + TableItem + std::fmt::Debug + std::fmt::Debug + std::fmt::Debug> Component
    for Model<T>
{
    type Msg = Msg<T>;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Msg<T>>) {
        match msg {
            Msg::GetAll => {
                //orders.perform_cmd(
                //    T::get_all()
                //        .map(|res| Msg::All(res))
                //);
            }
            Msg::All(res) => {
                match res {
                    Ok(entries) => self.previews = init_previews(entries),
                    Err(e) => {
                        seed::log(e);
                    }
                }
            }
            Msg::Preview(index, msg) => {
                self.previews[index].update(
                    msg.clone(),
                    &mut orders.proxy(move |msg| Msg::Preview(index.clone(), msg)),
                );
                if let preview::Msg::Entry(entry::Msg::Deleted(_)) = msg {
                    self.previews.remove(index);
                }
            }
        }
    }
}
impl<T: Component + preview::Preview + TableItem + std::fmt::Debug + std::fmt::Debug> Viewable
    for Model<T>
{
    fn view(&self) -> Node<Msg<T>> {
        div![ul![self.previews.iter().enumerate().map(|(i, preview)| {
            li![preview
                .view()
                .map_msg(move |msg| Msg::Preview(i.clone(), msg))]
        })]]
    }
}
