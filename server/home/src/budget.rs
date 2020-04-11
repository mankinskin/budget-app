pub use plans::{
    *,
    currency::*,
};
use yew::{
    *,
};

use crate::{
    *,
    transactions::*,
};

pub enum Msg {
    Update,
}
pub struct BudgetView<C: 'static + Currency> {
    link: ComponentLink<Self>,
    model: Budget<C>,
}

impl<C: 'static + Currency> Component for BudgetView<C> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut b = Budget::create("My Budget", 0);
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        Self {
            link,
            model: b,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="budget-container">
                <div class="budget-header">
                    <div class="budget-name">{
                        self.model.name()
                    }</div>
                </div>
                {TransactionsView::from(self.model.transactions.clone()).view()}
            </div>
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update => {
                true
            },
            _ => false
        }
    }
}
