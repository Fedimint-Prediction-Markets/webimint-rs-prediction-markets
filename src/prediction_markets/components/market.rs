use anyhow::anyhow;
use fedimint_core::OutPoint;
use fedimint_prediction_markets_common::config::GeneralConsensus;
use fedimint_prediction_markets_common::Outcome;
use leptos::*;

use crate::context::ClientContext;
use crate::prediction_markets::components::CandlestickChart;
use crate::utils::empty_view;

#[component]
pub fn Market(cx: Scope, market_outpoint: Memo<OutPoint>) -> impl IntoView {
    let ClientContext { client, .. } = expect_context::<ClientContext>(cx);
    let general_consensus = expect_context::<GeneralConsensus>(cx);

    let get_market_resource = create_resource(
        cx,
        move || market_outpoint.get(),
        move |market: OutPoint| async move { client.get_value().get_market(market, false).await },
    );
    let get_market_result = move || match get_market_resource.read(cx) {
        Some(Ok(Some(m))) => Ok(m),
        Some(Ok(None)) => Err(anyhow!("market does not exist")),
        Some(Err(_)) => Err(anyhow!("issue getting market")),
        None => Err(anyhow!("action has not run")),
    };
    let market = move || get_market_result().ok();

    let outcome = create_rw_signal(cx, Outcome::from(0));

    let candlestick_interval = create_rw_signal(
        cx,
        general_consensus
            .candlestick_intervals
            .get(0)
            .unwrap()
            .to_owned(),
    );

    view! { cx,
        <Show
            when=move || matches!{market(), Some(_)}
            fallback=|_| empty_view()
        >
            <h1 class="text-2xl">{market().map(|m| m.information.title)}</h1>

            <table class="p-2 bor">
                <thead>
                    <th>"Payout Control Public Key"</th>
                    <th>"Weight"</th>
                </thead>
                {move || market().map(|m| {
                    m.payout_controls_weights
                        .into_iter()
                        .map(move |(k, v)| view! {
                            cx,
                            <tr class="border-b">
                                <td>{k.to_string()}</td>
                                <td>{v}</td>
                            </tr>
                        })
                        .collect_view(cx)
                })}
            </table>

            <div class="flex">
                {move || market().map(|m| {
                    m.information.outcome_titles.into_iter().enumerate().map(|(i, outcome_title)| {
                        view! {
                            cx,
                            <div on:click=move |_| {outcome.set(i as Outcome)} class="p-4 border-1">{outcome_title}</div>
                        }
                    }).collect_view(cx)
                })}
            </div>

            <div class="flex">
                {general_consensus.candlestick_intervals.iter().map(|ci| {
                        let ci = ci.to_owned();

                        view! {
                            cx,
                            <div on:click=move |_| {candlestick_interval.set(ci)} class="p-4 border-1">{ci}s</div>
                        }
                    }).collect_view(cx)}
            </div>

            <CandlestickChart market_outpoint=market_outpoint outcome=outcome candlestick_interval=candlestick_interval/>
        </Show>
    }
}
