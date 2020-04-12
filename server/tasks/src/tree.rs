use yew::{
    *,
};
use crate::task::{
    *,
};

#[derive(Clone, Debug)]
pub enum Msg {
    ToggleExpand,
    SetParentMessenger(Callback<Self>),
    ChildMessage(usize, Box<Msg>),
}

#[derive(Properties, Clone, Debug)]
pub struct TreeData<P>
    where P: Properties
{
    pub element: P,
    pub expanded: bool,
    pub children: Vec<TreeData<P>>,
    pub message_parent: Option<Callback<Msg>>,
}

impl<P> TreeData<P>
    where P: Properties
{
    fn update(&mut self, msg: Msg) {
        match msg.clone() {
            Msg::ToggleExpand => {
                console!(log, format!("Toggle"));
                self.expanded = !self.expanded;
            },
            Msg::SetParentMessenger(callback) => {
                console!(log, format!("Toggle"));
                self.message_parent = Some(callback);
            },
            Msg::ChildMessage(child_index, child_msg) => {
                self.children[child_index].update(*child_msg);
            },
        }
    }
}

pub struct TreeView<C>
    where C: Component + Preview,
          <C as Component>::Properties: std::fmt::Debug,
{
    props: TreeData<<C as Component>::Properties>,
    link: ComponentLink<Self>,
}

impl<C> TreeView<C>
    where C: Component + Preview,
          <C as Component>::Properties: std::fmt::Debug,
{
    pub fn toggle_expand(&self) -> Callback<ClickEvent> {
        self.link.callback(|_| {
            Msg::ToggleExpand
        })
    }
    pub fn child_messenger(&self, child_index: usize)  -> Callback<Msg>{
        self.link.callback(move |msg| {
            Msg::ChildMessage(child_index, Box::new(msg))
        })
    }
}

impl<C> Component for TreeView<C>
    where C: Component + Preview,
          <C as Component>::Properties: std::fmt::Debug,
{
    type Message = Msg;
    type Properties = TreeData<<C as Component>::Properties>;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Self {
            props,
            link,
        };
        s.props.children = s.props.children
            .iter()
            .cloned()
            .enumerate()
            .map(|(child_index, mut child)| {
                let callback = s.child_messenger(child_index);
                child.message_parent = Some(callback.clone());
                callback.emit(Msg::SetParentMessenger(callback.clone()));
                child
            })
            .collect();
        s
    }
    fn view(&self) -> Html {
        console!(log, format!("{:#?}\n-------------------\n", self.props));
        let props = self.props.element.clone();
        html!{
            <div class="tree-node">
                <div class="tree-header" onclick=&self.toggle_expand()>
                    { // icon
                        html!{
                            <div class="tree-icon">
                                <ion-icon
                                    name={
                                        if self.props.expanded {
                                            "caret-down-outline"
                                        } else {
                                            "caret-forward-outline"
                                        }
                                    }
                                ></ion-icon>
                            </div>
                        }
                    }
                    <div class="tree-preview">{
                        C::preview(props.clone())
                    }</div>
                </div>
                { // item
                    if self.props.expanded {
                        html!{
                            <div class="tree-expanded">
                                <div class="tree-content">
                                    <div class="tree-line"/>
                                    <div class="tree-element">
                                        <C with props/>
                                    </div>
                                </div>
                                <div class="tree-children">
                                    { // item
                                        for self.props
                                        .children
                                        .iter()
                                        .cloned()
                                        .map(|props| html! {
                                                <div class="tree-child">
                                                    <div class="tree-line"/>
                                                    <TreeView<C> with props/>
                                                </div>
                                        })
                                    }
                                </div>
                            </div>
                        }
                    } else { html!{} }
                }
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        if let Some(message_parent) = &self.props.message_parent {
            // self is child
            message_parent.emit(msg.clone());
            false
        } else {
            // self is root
            self.props.update(msg);
            true
        }
    }
}