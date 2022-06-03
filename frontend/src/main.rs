use yew::prelude::*;
use yew_router::prelude::*;

mod components;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/property/:id")]
    Property,
    #[at("/properties")]
    PropertyList,
    #[at("/property/new")]
    AddProperty,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <h1 class="btn">{ "Hello World!" }</h1>
    }
}

fn main() {
    yew::start_app::<App>();
}
