mod table_models;
use crate::table_models::{KrakenTableTransaction, KrakenTransaction, ServerResponse};
use gloo_net::http::Request;
use yew::classes;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <KrakenTable />
    }
}

#[function_component]
fn KrakenTable() -> Html {
    let data = use_state(|| vec![]);
    let pagination = use_state(|| (0, 10));

    {
        let data = data.clone();
        let pagination = pagination.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let transactions: Vec<KrakenTableTransaction> =
                        Request::get("http://localhost:3000/api/v1/kraken-transaction")
                            .query([
                                ("page", pagination.0.to_string()),
                                ("rows", pagination.1.to_string()),
                            ])
                            .send()
                            .await
                            .unwrap()
                            .json::<ServerResponse<Vec<KrakenTransaction>>>()
                            .await
                            .unwrap()
                            .response
                            .unwrap_or_default()
                            .into_iter()
                            .map(KrakenTableTransaction::from)
                            .collect();

                    data.set(transactions);
                });
                || ()
            },
            (),
        );
    }

    let previous_pagination = pagination.clone();
    let next_pagination = pagination.clone();

    let previous_callback = Callback::from(move |_| {
        if previous_pagination.0 > 0 {
            previous_pagination.set((previous_pagination.0 - 1, previous_pagination.1))
        }
    });
    let next_callback = Callback::from(move |_| {
        next_pagination.set((next_pagination.0 + 1, next_pagination.1));
    });

    html! {
        <div class={classes!("table-container")}>
            <table>
                <thead>
                    { kraken_header_row(data.clone()) }
                </thead>
                <tbody>
                    { kraken_row_item(data.clone()) }
                </tbody>
            </table>

            <div class={classes!("page-button-container")}>
            <button onclick={previous_callback}>{"Previous"}</button>
            <button onclick={next_callback}>{"Next"}</button>
            </div>
        </div>
    }
}

fn kraken_header_row(_data: UseStateHandle<Vec<KrakenTableTransaction>>) -> Html {
    html! {
            <tr>
                <th>{"id"}</th>
                <th>{"asset"}</th>
                <th>{"amount"}</th>
                <th>{"Sure"}</th>
            </tr>
    }
}

fn kraken_row_item(data: UseStateHandle<Vec<KrakenTableTransaction>>) -> Vec<Html> {
    println!("data: {}", data.iter().len());
    data.iter()
        .map(|transaction| {
            html! {
                <tr>
                    <td>{&transaction.id}</td>
                    <td>{&transaction.asset}</td>
                    <td>{&transaction.amount}</td>
                    <td></td>
                </tr>
            }
        })
        .collect()
}

fn main() {
    yew::Renderer::<App>::new().render();
}

// // we use `flavor = "current_thread"` so this snippet can be tested in CI,
// // where tests are run in a WASM environment. You likely want to use
// // the (default) `multi_thread` favor as:
// #[tokio::main]
// #[tokio::main(flavor = "current_thread")]
// async fn no_main() {
//     let renderer = ServerRenderer::<App>::new();

//     let rendered = renderer.render().await;

//     // Prints: <div>Hello, World!</div>
//     println!("{}", rendered);
// }
