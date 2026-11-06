// The Routing States tab: the ordered PDF over routing states. Each routing
// state is depicted as a mini version of the network (unlabeled) with green
// arrows showing the forwarding neighbour of every router, or a red cross if
// the router has no forwarding neighbour. The states are ordered from most
// likely (left) to least likely (right), with a probability bar above each
// depiction.

use std::collections::BTreeMap;

use bgpsim::bgp::BgpSessionType;
use bgpsim::types::RouterId;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::net::demo::DEMO_ASN;
use crate::net::{Net, Pfx};
use crate::point::Point;

use super::simulate::RoutingStateOutcome;

/// A node of the mini network, with a fixed position in the mini coordinate
/// system (viewBox 0 0 100 66).
#[derive(Clone, PartialEq)]
pub(crate) struct MiniNode {
    pub(crate) id: RouterId,
    pub(crate) p: Point,
    pub(crate) external: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct MiniLink {
    pub(crate) a: RouterId,
    pub(crate) b: RouterId,
    pub(crate) external: bool,
}

/// Snapshot of the network topology used to draw the mini networks.
pub(crate) struct MiniTopology {
    pub(crate) nodes: Vec<MiniNode>,
    pub(crate) links: Vec<MiniLink>,
}

/// Fixed positions of the demo routers, by name. Unknown routers are placed
/// on a fallback row at the top.
fn position_of(name: &str, fallback_idx: usize) -> Point {
    match name {
        "R1" => Point::new(50.0, 12.0),
        "R0" => Point::new(27.0, 40.0),
        "R2" => Point::new(73.0, 40.0),
        "E1" => Point::new(6.0, 58.0),
        "E2" => Point::new(94.0, 58.0),
        _ => Point::new(8.0 + 12.0 * fallback_idx as f64, 2.0),
    }
}

pub(crate) fn topology(net: &Net) -> MiniTopology {
    let n = net.net();
    let mut nodes = Vec::new();
    let mut fallback_idx = 0;
    for id in n.indices() {
        let (name, external) = match n.get_router(id) {
            Ok(r) => (r.name().to_string(), r.asn() != DEMO_ASN),
            Err(_) => continue,
        };
        let p = position_of(&name, fallback_idx);
        if !["R0", "R1", "R2", "E1", "E2"].contains(&name.as_str()) {
            fallback_idx += 1;
        }
        nodes.push(MiniNode { id, p, external });
    }

    let mut links: Vec<MiniLink> = n
        .ospf_network()
        .edges()
        .map(|e| MiniLink { a: e.src(), b: e.dst(), external: false })
        .collect();
    for (a, b, ty, _) in net.get_bgp_sessions() {
        if ty == BgpSessionType::EBgp {
            links.push(MiniLink { a, b, external: true });
        }
    }

    MiniTopology { nodes, links }
}

/// How many routing states are shown per page.
pub(crate) const STATES_PER_PAGE: usize = 6;

#[function_component]
pub fn StatesTab() -> Html {
    let outcomes = use_selector(|f: &super::Forecast| f.outcomes.clone());
    let (net, net_dispatch) = use_store::<Net>();
    let selected = use_selector(|f: &super::Forecast| f.selected());

    // order the states from most likely to least likely
    let mut outcomes = (*outcomes).as_ref().clone();
    if outcomes.is_empty() {
        return html! {
            <EmptyHint text={"Press \"Run forecast\" in the sidebar to compute the routing states."} />
        };
    }
    outcomes.sort_by(|a, b| {
        b.probability
            .partial_cmp(&a.probability)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let topo = topology(&net);

    // clicking a card applies one routing-input combination that leads to
    // this state on the visible network, so that the data-plane layer
    // highlights the forwarding state; clicking it again deselects.
    let selected_combination: Option<(usize, usize)> = *selected;
    let on_click: Callback<usize> = {
        let net_dispatch = net_dispatch.clone();
        let outcomes: Vec<RoutingStateOutcome> = outcomes.clone();
        Callback::from(move |index| {
            let Some(o): Option<&super::simulate::RoutingStateOutcome> = outcomes.get(index) else { return };
            let combination: (usize, usize) = *o.combinations.first().unwrap_or(&(0, 0));
            let was_selected = selected_combination == Some(combination);
            let combination = if was_selected { None } else { Some(combination) };
            Dispatch::<super::Forecast>::new()
                .reduce_mut(move |f| f.set_selected(combination));
            if let Some((i, j)) = combination {
                let prefix: Pfx = crate::net::demo::DEMO_PREFIX.parse().unwrap();
                net_dispatch.reduce_mut(move |n| {
                    let (e1, e2) = super::simulate::external_routers(&n);
                    let in1 = super::simulate::Input::from_index(i);
                    let in2 = super::simulate::Input::from_index(j);
                    let _ = super::simulate::apply_input(&mut n.net_mut(), e1, in1, prefix);
                    let _ = super::simulate::apply_input(&mut n.net_mut(), e2, in2, prefix);
                });
                // show the forwarding state on the canvas
                Dispatch::<crate::state::State>::new().reduce_mut(move |s| {
                    s.set_layer(crate::state::Layer::FwState);
                    s.set_prefix(Some(prefix));
                });
            }
        })
    };
    let selected_index = selected_combination.and_then(|sel| {
        outcomes.iter().position(|o| o.combinations.first() == Some(&sel))
    });
    let bar_classes: Vec<Classes> = outcomes.iter().map(|_| Classes::from("bg-blue")).collect();

    html! {
        <StateHistogram {outcomes} topology={topo.nodes} links={topo.links} bar_classes={bar_classes} selected={selected_index} on_click={Some(on_click)} />
    }
}

/// The paginated histogram over routing states: an info box with the total
/// number of states, and the state cards (mini network + probability bar)
/// ordered from most likely to least likely, with arrows to browse between
/// pages. Shared between the Routing States tab and the Specification tab.
#[derive(Properties, PartialEq)]
pub(crate) struct StateHistogramProps {
    /// The routing states, already sorted from most likely to least likely.
    pub outcomes: Vec<RoutingStateOutcome>,
    pub topology: Vec<MiniNode>,
    pub links: Vec<MiniLink>,
    /// The bar colour class of each state (same length as `outcomes`).
    pub bar_classes: Vec<Classes>,
    /// Click callback (index into `outcomes`), if the cards are clickable.
    #[prop_or_default]
    pub on_click: Option<Callback<usize>>,
    /// Index of the highlighted (selected) card.
    #[prop_or_default]
    pub selected: Option<usize>,
}

#[function_component]
pub(crate) fn StateHistogram(props: &StateHistogramProps) -> Html {
    let outcomes = &props.outcomes;
    let page_handle = use_state(|| 0usize);

    let total = outcomes.len();
    let max_page = total.saturating_sub(1) / STATES_PER_PAGE;
    let page = (*page_handle).min(max_page);
    let start = page * STATES_PER_PAGE;
    let end = (start + STATES_PER_PAGE).min(total);

    let max_probability = outcomes
        .first()
        .map(|o| o.probability)
        .unwrap_or(1.0)
        .max(1e-9);

    let cards = outcomes[start..end].iter().enumerate().map(|(k, o)| {
        let index = start + k;
        let is_selected = props.selected == Some(index);
        let onclick = props.on_click.clone().map(|cb| {
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                cb.emit(index)
            })
        });
        let bar_class = props
            .bar_classes
            .get(index)
            .cloned()
            .unwrap_or_else(|| Classes::from("bg-blue"));
        html! {
            <StateCard
                outcome={o.clone()}
                max_probability={max_probability}
                topology={props.topology.clone()}
                links={props.links.clone()}
                selected={is_selected}
                bar_class={bar_class}
                onclick={onclick}
            />
        }
    }).collect::<Html>();

    // the info box: total number of states, and a note when not all are shown
    let s = if total == 1 { "" } else { "s" };
    let info_text = if total <= STATES_PER_PAGE {
        format!("{total} routing state{s}")
    } else {
        format!(
            "{total} routing states in total — showing {}–{} of {total}. \
             Only the top {STATES_PER_PAGE} routing states are shown per page; \
             use the arrows to browse all of them.",
            start + 1,
            end
        )
    };

    let prev_class = classes!(
        "p-2",
        "rounded-full",
        "shrink-0",
        "transition",
        "duration-150",
        "ease-in-out",
        if page > 0 {
            "text-main hover:text-blue hover:bg-base-2 cursor-pointer"
        } else {
            "text-base-4 cursor-not-allowed"
        },
    );
    let next_class = classes!(
        "p-2",
        "rounded-full",
        "shrink-0",
        "transition",
        "duration-150",
        "ease-in-out",
        if page < max_page {
            "text-main hover:text-blue hover:bg-base-2 cursor-pointer"
        } else {
            "text-base-4 cursor-not-allowed"
        },
    );
    let on_prev = {
        let page_handle = page_handle.clone();
        Callback::from(move |_| {
            let p = (*page_handle).min(max_page);
            page_handle.set(p.saturating_sub(1));
        })
    };
    let on_next = {
        let page_handle = page_handle.clone();
        Callback::from(move |_| {
            let p = (*page_handle).min(max_page);
            page_handle.set((p + 1).min(max_page));
        })
    };

    html! {
        <div class="w-full h-full flex flex-col">
            <div class="px-4 pt-2 shrink-0">
                <div class="inline-block bg-base-2 rounded-md px-3 py-1.5 text-sm text-main">
                    { info_text }
                </div>
            </div>
            <div class="flex-1 min-h-0 flex items-end gap-2 px-2 pb-3">
                <button class={prev_class} onclick={on_prev} disabled={page == 0}
                        title="Previous routing states">
                    <yew_lucide::ChevronLeft class="w-6 h-6" />
                </button>
                // the top padding must be inside the scroll container, so that
                // the probability labels of the tallest bars are not clipped
                <div class="flex-1 min-w-0 flex items-end justify-around gap-2 overflow-x-auto pt-8">
                    { cards }
                </div>
                <button class={next_class} onclick={on_next} disabled={page == max_page}
                        title="Next routing states">
                    <yew_lucide::ChevronRight class="w-6 h-6" />
                </button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct StateCardProps {
    outcome: RoutingStateOutcome,
    max_probability: f64,
    topology: Vec<MiniNode>,
    links: Vec<MiniLink>,
    selected: bool,
    bar_class: Classes,
    onclick: Option<Callback<MouseEvent>>,
}

#[function_component]
fn StateCard(props: &StateCardProps) -> Html {
    let outcome = &props.outcome;
    let height = outcome.probability / props.max_probability * 100.0;
    let pct = format!("{:.0}%", outcome.probability * 100.0);
    let card_class = classes!(
        "flex",
        "flex-col",
        "items-center",
        "gap-1",
        "shrink-0",
        "p-1",
        "rounded-lg",
        "transition",
        "duration-150",
        "ease-in-out",
        if props.selected {
            "bg-base-2 ring-2 ring-blue"
        } else if props.onclick.is_some() {
            "hover:bg-base-2 cursor-pointer"
        } else {
            ""
        },
    );

    html! {
        <div class={card_class} style="width: 8.5rem;" onclick={props.onclick.clone()}>
            <div class="w-full h-24 flex items-end">
                <div class={classes!("relative", "w-full", "rounded-t", props.bar_class.clone())} style={format!("height: {height}%")}>
                    <span class="absolute -top-5 left-0 right-0 text-center text-sm font-semibold">
                        { pct }
                    </span>
                </div>
            </div>
            <MiniNetwork paths={outcome.state.paths.clone()} topology={props.topology.clone()} links={props.links.clone()} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct MiniNetworkProps {
    pub(crate) paths: BTreeMap<RouterId, Vec<Vec<RouterId>>>,
    pub(crate) topology: Vec<MiniNode>,
    pub(crate) links: Vec<MiniLink>,
}

/// A mini version of the network (unlabeled): green arrows show the
/// forwarding neighbour of every router; a red cross marks routers without a
/// forwarding neighbour.
#[function_component]
pub(crate) fn MiniNetwork(props: &MiniNetworkProps) -> Html {
    let nodes = &props.topology;
    let pos = |id: &RouterId| nodes.iter().find(|n| &n.id == id).map(|n| n.p);

    // green arrows towards the forwarding neighbour(s)
    let arrows = props.paths.iter().filter_map(|(id, router_paths)| {
        if router_paths.is_empty() {
            return None; // no route: red cross instead
        }
        let from = pos(id)?;
        let next_hops: Vec<Point> = router_paths
            .iter()
            .filter_map(|path| path.get(1))
            .filter_map(pos)
            .collect();
        Some((from, next_hops))
    }).map(|(from, next_hops)| {
        next_hops.iter().map(|to| {
            let d = *to - from;
            let u = d.normalize();
            let start = from + u * 6.0;
            let end = *to - u * 7.5;
            html! {
                <line class="stroke-green" stroke-width="1.5" marker-end="url(#fw-arrow)"
                      x1={start.x()} y1={start.y()} x2={end.x()} y2={end.y()} />
            }
        }).collect::<Vec<Html>>()
    }).flatten().collect::<Html>();

    // red crosses on routers without a forwarding neighbour
    let crosses = props.paths.iter().filter(|(_, router_paths)| router_paths.is_empty())
        .filter_map(|(id, _)| pos(id))
        .map(|p| {
            html! {
                <>
                    <line class="stroke-red" stroke-width="1.5" x1={(p.x - 5.0).to_string()} y1={(p.y - 5.0).to_string()} x2={(p.x + 5.0).to_string()} y2={(p.y + 5.0).to_string()} />
                    <line class="stroke-red" stroke-width="1.5" x1={(p.x - 5.0).to_string()} y1={(p.y + 5.0).to_string()} x2={(p.x + 5.0).to_string()} y2={(p.y - 5.0).to_string()} />
                </>
            }
        }).collect::<Html>();

    let links = props.links.iter().map(|l| {
        let (Some(pa), Some(pb)) = (pos(&l.a), pos(&l.b)) else {
            return html!();
        };
        html! {
            <line class="stroke-base-4" stroke-width="1" x1={pa.x()} y1={pa.y()} x2={pb.x()} y2={pb.y()} />
        }
    }).collect::<Html>();

    let nodes = nodes.iter().map(|n| {
        if n.external {
            html! {
                <rect class="fill-base-3 stroke-main" stroke-width="1"
                      x={(n.p.x - 4.0).to_string()} y={(n.p.y - 4.0).to_string()} width="8" height="8" />
            }
        } else {
            html! {
                <circle class="fill-base-1 stroke-main" stroke-width="1" cx={n.p.x()} cy={n.p.y()} r="5" />
            }
        }
    }).collect::<Html>();

    html! {
        <svg class="w-full" viewBox="0 0 100 66">
            <defs>
                <marker id="fw-arrow" viewBox="0 0 10 10" refX="8" refY="5"
                        markerWidth="5" markerHeight="5" orient="auto-start-reverse">
                    <path d="M 0 0 L 10 5 L 0 10 z" class="fill-green" />
                </marker>
            </defs>
            { links }
            { arrows }
            { nodes }
            { crosses }
        </svg>
    }
}

#[derive(Properties, PartialEq)]
struct EmptyHintProps {
    text: &'static str,
}

#[function_component]
fn EmptyHint(props: &EmptyHintProps) -> Html {
    html! {
        <div class="w-full h-full flex items-center justify-center">
            <p class="text-main-ia italic text-sm">{ props.text }</p>
        </div>
    }
}
