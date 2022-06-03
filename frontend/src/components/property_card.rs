use yew::prelude::*;

#[derive(PartialEq, Properties)]
struct PropertyCardProps {
    id: i32,
}

#[function_component(PropretyCard)]
fn propert_card(props: &PropertyCardProps) -> Html {
    let PropertyCardProps { id } = props;
    html!(
    <>
    </>
    )
}

