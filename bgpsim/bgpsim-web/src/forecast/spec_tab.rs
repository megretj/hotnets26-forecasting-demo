// The Specification tab: the user selects a specification (fully connected,
// load balanced, or MLU); each routing state is mapped to the outcome of the
// specification, and the probabilities of all states leading to the same
// outcome are added up. The result is displayed as a histogram over the
// outcome values (for MLU: an x-axis from 0 to 1 with a red line at the
// expected value).

use bgpsim::types::RouterId;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::Net;

use super::simulate::RoutingStateOutcome;
use super::spec::{self, Spec};

/// The two views of a boolean specification: the aggregated outcome histogram
/// (True / False), or the routing states coloured by the property value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoolView {
    Outcomes,
    States,
}

#[function_component]
pub fn SpecTab() -> Html {
    let outcomes = use_selector(|f: &super::Forecast| f.outcomes.clone());
    let mlu_values = use_selector(|f: &super::Forecast| f.mlu_values().cloned());
    let (net, _) = use_store::<Net>();
    let spec = use_selector(|s: &crate::state::State| s.spec());
    let bool_view = use_state(|| BoolView::Outcomes);

    let (r0, r2) = internal_pair(&net);

    let spec_button = |s: Spec| {
        let active = *spec == s;
        let class = classes!(
            "px-3",
            "py-1",
            "rounded-md",
            "text-sm",
            "transition",
            "duration-150",
            "ease-in-out",
            if active {
                "bg-blue text-base-1"
            } else {
                "text-main hover:bg-base-2"
            },
        );
        let onclick =
            Dispatch::<crate::state::State>::new().reduce_mut_callback(move |st| st.set_spec(s));
        html! {
            <button class={class} {onclick}>{ s.label() }</button>
        }
    };

    if outcomes.is_empty() {
        return html! {
            <div class="w-full h-full flex flex-col">
                <div class="flex items-center gap-2 px-4 py-2 border-b border-base-3 shrink-0">
                    <div class="flex bg-base-2 rounded-md p-0.5">
                        { spec_button(Spec::FullyConnected) }
                        { spec_button(Spec::LoadBalanced) }
                        { spec_button(Spec::Mlu) }
                    </div>
                    <p class="text-sm text-main-ia">
                        { "P(outcome) of the selected specification over the routing states." }
                    </p>
                </div>
                <div class="flex-1 min-h-0 flex">
                    <div class="flex-1 min-w-0 flex items-center justify-center">
                        <p class="text-main-ia italic text-sm">
                            { "Press \"Run forecast\" in the sidebar to compute the routing states." }
                        </p>
                    </div>
                    <div class="w-64 shrink-0 p-3">
                        <div class="w-full h-full bg-base-2 rounded-md p-3 overflow-y-auto flex flex-col" id="spec-description">
                            <p class="text-sm font-semibold mb-1">{ spec.label() }</p>
                            <p class="text-sm text-main">{ spec.description() }</p>
                        </div>
                    </div>
                </div>
            </div>
        };
    }

    let bool_view_button = |v: BoolView, label: &'static str| {
        let active = *bool_view == v;
        let class = classes!(
            "px-3",
            "py-1",
            "rounded-md",
            "text-sm",
            "transition",
            "duration-150",
            "ease-in-out",
            if active {
                "bg-blue text-base-1"
            } else {
                "text-main hover:bg-base-2"
            },
        );
        let onclick = {
            let bool_view = bool_view.clone();
            Callback::from(move |_| bool_view.set(v))
        };
        html! {
            <button class={class} {onclick}>{ label }</button>
        }
    };

    let content = match *spec {
        Spec::FullyConnected | Spec::LoadBalanced => {
            let property = |o: &RoutingStateOutcome| match *spec {
                Spec::FullyConnected => spec::fully_connected(o),
                _ => spec::load_balanced(o, &r0, &r2),
            };
            match *bool_view {
                BoolView::Outcomes => {
                    let p_true: f64 = outcomes
                        .iter()
                        .filter(|o| property(o))
                        .map(|o| o.probability)
                        .sum();
                    html! {
                        <BoolHistogram p_true={p_true} />
                    }
                }
                BoolView::States => {
                    // the routing states, coloured green (property holds) or
                    // red (property does not hold)
                    let mut sorted = (*outcomes).as_ref().clone();
                    sorted.sort_by(|a, b| {
                        b.probability
                            .partial_cmp(&a.probability)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    let bar_classes: Vec<Classes> = sorted
                        .iter()
                        .map(|o| {
                            if property(o) {
                                Classes::from("bg-green")
                            } else {
                                Classes::from("bg-red")
                            }
                        })
                        .collect();
                    let topo = super::states_tab::topology(&net);
                    html! {
                        <super::states_tab::StateHistogram
                            outcomes={sorted}
                            topology={topo.nodes}
                            links={topo.links}
                            bar_classes={bar_classes}
                        />
                    }
                }
            }
        }
        Spec::Mlu => {
            // use the distribution computed by the "Recompute" button, or the
            // distribution for the default parameters
            let values = match mlu_values.as_ref() {
                Some(v) => (**v).clone(),
                None => default_mlu_values(&outcomes),
            };
            html! {
                <MluHistogram {values} />
            }
        }
    };

    let bool_view_toggle = |show: bool| {
        if !show {
            return html!();
        }
        html! {
            <div class="flex bg-base-2 rounded-md p-0.5 ml-2">
                { bool_view_button(BoolView::Outcomes, "Outcomes") }
                { bool_view_button(BoolView::States, "Routing states") }
            </div>
        }
    };

    html! {
        <div class="w-full h-full flex flex-col">
            <div class="flex items-center gap-2 px-4 py-2 border-b border-base-3 shrink-0">
                <div class="flex bg-base-2 rounded-md p-0.5">
                    { spec_button(Spec::FullyConnected) }
                    { spec_button(Spec::LoadBalanced) }
                    { spec_button(Spec::Mlu) }
                </div>
                { bool_view_toggle(matches!(*spec, Spec::FullyConnected | Spec::LoadBalanced)) }
                <p class="text-sm text-main-ia">
                    { "P(outcome) of the selected specification over the routing states." }
                </p>
            </div>
            <div class="flex-1 min-h-0 flex">
                <div class="flex-1 min-w-0">
                    { content }
                </div>
                <div class="w-64 shrink-0 p-3">
                    <div class="w-full h-full bg-base-2 rounded-md p-3 overflow-y-auto flex flex-col" id="spec-description">
                        <p class="text-sm font-semibold mb-1">{ spec.label() }</p>
                        <p class="text-sm text-main">{ spec.description() }</p>
                        if *spec == Spec::Mlu {
                            <MluParams net_snapshot={topology_snapshot(&net)} />
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}

fn internal_pair(net: &Net) -> (RouterId, RouterId) {
    let n = net.net();
    let r0 = n.get_router_id("R0").ok();
    let r2 = n.get_router_id("R2").ok();
    match (r0, r2) {
        (Some(a), Some(b)) => (a, b),
        _ => (RouterId::from(0), RouterId::from(0)),
    }
}

#[derive(Properties, PartialEq)]
struct BoolHistogramProps {
    p_true: f64,
}

/// Histogram over a boolean outcome: two bars (True, False).
#[function_component]
fn BoolHistogram(props: &BoolHistogramProps) -> Html {
    let p_true = props.p_true.clamp(0.0, 1.0);
    let p_false = 1.0 - p_true;
    let bar = |label: &'static str, p: f64, color: &'static str| {
        let height = (p * 100.0).max(0.0);
        html! {
            <div class="flex flex-col items-center gap-1 h-full">
                <div class="relative w-24 h-full flex items-end bg-base-2 rounded">
                    <div class={classes!("w-full", "rounded", color)} style={format!("height: {height}%")} />
                    <span class="absolute inset-x-0 top-1 text-center text-sm font-semibold">
                        { format!("{:.0}%", p * 100.0) }
                    </span>
                </div>
                <span class="text-sm text-main font-medium">{ label }</span>
            </div>
        }
    };
    html! {
        <div class="w-full h-full flex items-stretch justify-center gap-16 p-4">
            { bar("True", p_true, "bg-green") }
            { bar("False", p_false, "bg-red") }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct MluHistogramProps {
    /// The distinct MLU values with their probabilities.
    values: Vec<(f64, f64)>,
}

/// Histogram over the (normalized) MLU: bars at the distinct MLU values,
/// with a red vertical line at the expected value. The x-axis spans at least
/// [0, 1] and extends (in half steps) when MLU values exceed 1.
#[function_component]
fn MluHistogram(props: &MluHistogramProps) -> Html {
    let values = &props.values;
    let expected: f64 = values.iter().map(|(v, p)| v * p).sum();

    // the x-axis maximum: at least 1, rounded up to a half step above the
    // largest MLU value (so that values above 1 fit into the plot)
    let max_value = values
        .iter()
        .map(|(v, _)| *v)
        .fold(expected, f64::max)
        .max(1.0);
    let axis_max = (max_value * 2.0).ceil() / 2.0;
    let scale = |v: f64| v / axis_max * 100.0;

    let bar_width = 7.0; // percent of the axis
    let bars = values.iter().map(|(v, p)| {
        let left = (scale(*v) - bar_width / 2.0).clamp(0.0, 100.0 - bar_width);
        let height = (p * 100.0).clamp(0.0, 100.0);
        html! {
            <div class="absolute bottom-0 bg-blue rounded-t"
                 style={format!("left: {left}%; width: {bar_width}%; height: {height}%")} />
        }
    }).collect::<Html>();

    let y_tick = |label: &'static str| {
        html! {
            <span class="text-xs text-main-ia">{ label }</span>
        }
    };

    // format with as few decimals as needed (one for integers/half steps,
    // two otherwise)
    let fmt = |v: f64| {
        if (v * 10.0).round() == v * 10.0 {
            format!("{v:.1}")
        } else {
            format!("{v:.2}")
        }
    };

    let x_tick = |v: f64| {
        html! {
            <span class="absolute text-xs text-main-ia" style={format!("left: {}%; transform: translateX(-50%)", scale(v))}>
                { fmt(v) }
            </span>
        }
    };

    html! {
        <div class="w-full h-full flex flex-col items-center justify-center px-8">
            <div class="w-full max-w-3xl flex">
                <div class="flex flex-col justify-between items-end h-40 pr-1 text-xs text-main-ia shrink-0" style="width: 2.5rem;">
                    { y_tick("100%") }
                    { y_tick("50%") }
                    { y_tick("0%") }
                </div>
                <div class="relative flex-1 h-40 border-b-2 border-l-2 border-base-4">
                    { bars }
                    <div class="absolute left-0 right-0 border-t border-base-3" style="bottom: 50%;" />
                    <div class="absolute bottom-0 bg-red"
                         style={format!("left: {}%; width: 2px; height: 100%;", scale(expected).clamp(0.0, 100.0))}
                         title="Expected value" />
                </div>
            </div>
            <div class="w-full max-w-3xl flex">
                <div class="shrink-0" style="width: 2.5rem;" />
                <div class="relative flex-1 h-5 mt-1">
                    { x_tick(0.0) }
                    { x_tick(axis_max / 2.0) }
                    { x_tick(axis_max) }
                </div>
            </div>
            <p class="text-center text-sm text-main mt-2">
                    { format!("normalized MLU (expected value: {expected:.2})") }
                </p>
        </div>
    }
}

/// The MLU distribution for the default parameters (aggregated from the
/// outcomes).
fn default_mlu_values(outcomes: &[RoutingStateOutcome]) -> Vec<(f64, f64)> {
    let mut values: Vec<(f64, f64)> = Vec::new();
    let mut total = 0.0;
    for o in outcomes.iter() {
        let mlu = (spec::mlu(o) * 1000.0).round() / 1000.0;
        total += o.probability;
        match values.iter_mut().find(|(v, _)| (*v - mlu).abs() < 1e-6) {
            Some((_, p)) => *p += o.probability,
            None => values.push((mlu, o.probability)),
        }
    }
    if total > 0.0 {
        for (_, p) in values.iter_mut() {
            *p /= total;
        }
    }
    values
}

/// A link or router of the network, with a human-readable name.
#[derive(Clone, PartialEq)]
struct NetElement {
    /// For a link: (router_a, router_b); for a router: (router, router).
    key: (RouterId, RouterId),
    name: String,
    is_link: bool,
}

/// Snapshot of the links (undirected) and internal routers, used for the MLU
/// parameter fields.
fn topology_snapshot(net: &Net) -> Vec<NetElement> {
    use std::collections::BTreeSet;
    let n = net.net();
    let name = |id: RouterId| {
        n.get_router(id)
            .map(|r| r.name().to_string())
            .unwrap_or_else(|_| format!("r{}", id.index()))
    };
    let mut links: BTreeSet<(RouterId, RouterId)> = BTreeSet::new();
    for e in n.ospf_network().edges() {
        let (a, b) = (e.src(), e.dst());
        links.insert(if a < b { (a, b) } else { (b, a) });
    }
    for (a, b, ty, _) in net.get_bgp_sessions() {
        if ty == bgpsim::bgp::BgpSessionType::EBgp {
            links.insert(if a < b { (a, b) } else { (b, a) });
        }
    }
    let mut elements: Vec<NetElement> = links
        .into_iter()
        .map(|(a, b)| NetElement {
            key: (a, b),
            name: format!("{}–{}", name(a), name(b)),
            is_link: true,
        })
        .collect();
    for id in n.indices() {
        if n.get_router(id).is_ok_and(|r| r.asn() == crate::net::demo::DEMO_ASN) {
            elements.push(NetElement {
                key: (id, id),
                name: name(id),
                is_link: false,
            });
        }
    }
    elements
}

#[derive(Properties, PartialEq)]
struct MluParamsProps {
    net_snapshot: Vec<NetElement>,
}

/// The editable MLU parameters (link bandwidths and traffic volumes) and the
/// recompute button, shown below the MLU description.
#[function_component]
fn MluParams(props: &MluParamsProps) -> Html {
    let bandwidth = use_selector(|f: &super::Forecast| f.mlu_bandwidth().clone());
    let traffic = use_selector(|f: &super::Forecast| f.mlu_traffic().clone());
    let recompute = Dispatch::<super::Forecast>::new()
        .reduce_mut_callback(|f| f.recompute_mlu());

    let input_class = "w-16 px-1 py-0.5 text-sm bg-base-1 border border-solid border-base-5 rounded focus:outline-none focus:border-blue text-right";

    let field = |name: String, value: f64, on_input: Callback<f64>| {
        let display = if value == value.trunc() {
            format!("{}", value)
        } else {
            format!("{value}")
        };
        html! {
            <div class="flex items-center justify-between gap-2 h-7">
                <span class="text-sm text-main truncate">{ name }</span>
                <input type="number" step="0.1" min="0" class={input_class}
                       value={display}
                       oninput={move |e: InputEvent| {
                           let v = e.target()
                               .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                               .map(|i| i.value())
                               .unwrap_or_default();
                           if let Ok(v) = v.parse::<f64>() {
                               on_input.emit(v);
                           }
                       }} />
            </div>
        }
    };

    let links = props.net_snapshot.iter().filter(|e| e.is_link).map(|e| {
        let (a, b) = e.key;
        let value = bandwidth.get(&e.key).copied().unwrap_or(spec::DEFAULT_BANDWIDTH);
        let on_input = Dispatch::<super::Forecast>::new()
            .reduce_mut_callback_with(move |f: &mut super::Forecast, v: f64| f.set_mlu_bandwidth(a, b, v));
        field(e.name.clone(), value, on_input)
    }).collect::<Html>();

    let routers = props.net_snapshot.iter().filter(|e| !e.is_link).map(|e| {
        let (r, _) = e.key;
        let value = traffic.get(&r).copied().unwrap_or(spec::DEFAULT_TRAFFIC);
        let on_input = Dispatch::<super::Forecast>::new()
            .reduce_mut_callback_with(move |f: &mut super::Forecast, v: f64| f.set_mlu_traffic(r, v));
        field(e.name.clone(), value, on_input)
    }).collect::<Html>();

    html! {
        <div class="mt-3 pt-2 border-t border-base-4 flex flex-col gap-1">
            <p class="text-sm font-semibold">{"Link bandwidth (MB/s)"}</p>
            { links }
            <p class="text-sm font-semibold mt-2">{"Traffic (MB/s)"}</p>
            { routers }
            <button class="mt-3 w-full py-1.5 rounded-md text-sm font-semibold bg-blue text-base-1 hover:bg-blue-dark transition duration-150 ease-in-out"
                    onclick={recompute}>
                { "Recompute" }
            </button>
        </div>
    }
}
