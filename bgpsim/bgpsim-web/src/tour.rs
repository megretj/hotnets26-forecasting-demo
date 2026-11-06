// The intro tour: a sequence of steps that walk the user through the demo.
// Each step dims the whole page (a grey overlay with mix-blend-multiply)
// except for the highlighted element (a blurred white rectangle on top of
// the overlay), with a popup that explains the highlighted area.

use std::rc::Rc;

use wasm_bindgen::JsCast;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::forecast::spec::Spec;
use crate::net::Net;
use crate::state::{PanelTab, Selected, State};

/// One step of the tour.
struct TourStep {
    /// CSS selector of the element to highlight. If `None`, the popup is
    /// centred on the screen.
    target: Option<&'static str>,
    /// The face of Paul shown next to the text bubble (relative URL).
    face: &'static str,
    /// Extra padding around the highlighted element.
    padding: f64,
    title: &'static str,
    text: Html,
    /// Action to perform when the step is entered (e.g., select a router, or
    /// switch the bottom panel tab).
    on_enter: fn(),
}

fn select_router(name: &'static str) {
    Dispatch::<Net>::new().reduce_mut(move |n| {
        if let Ok(id) = n.net().get_router_id(name) {
            let asn = n.net().get_router(id).map(|r| r.asn()).unwrap_or(crate::net::demo::DEMO_ASN);
            Dispatch::<crate::state::State>::new()
                .reduce_mut(move |s| s.set_selected(Selected::Router(id, asn)));
        }
    });
}

fn open_panel_tab(tab: PanelTab) {
    Dispatch::<crate::state::State>::new().reduce_mut(move |s| {
        s.set_panel_tab(tab);
    });
}

fn no_action() {}

/// The steps of the tour, in order.
fn steps() -> Vec<TourStep> {
    vec![
        TourStep {
            face: "./figures/happy.png",
            target: None,
            padding: 0.0,
            title: "Welcome to the probabilistic verification demo!",
            text: html! {
                <p>
                    { "Hi! I'm " }
                    <b> { "Paul, a network engineer" } </b>
                    { ", and I will talk you through this demo. In the following steps, I show you \
                       the network, its configuration, the routing input forecast, and the specifications. Press " }
                    <b> { "Next" } </b>
                    { " to continue, or " }
                    <b> { "Skip tour" } </b>
                    { " to explore on your own." }
                </p>
            },
            on_enter: no_action,
        },
        // 1: the network
        TourStep {
            face: "./figures/talk2.png",
            target: Some("#canvas"),
            padding: 12.0,
            title: "The canvas",
            text: html! {
                <p>
                    { "This is the network of the demo: three internal routers connected in a triangle, \
                       and two external networks E1 and E2, each attached to one of the routers. " }
                    { "The grey shaded areas are autonomous systems (ASes)." }
                    { " You can drag the routers around, \
                       and add internal nodes with the button above. Everything in this demo concerns a \
                       single prefix, " }
                    <b> { "129.132.57.30/32" } </b>
                    { ", which both external networks advertise." }
                </p>
            },
            on_enter: no_action,
        },
        // 1b: the views (layer selection) and the prefix
        TourStep {
            face: "./figures/talk3.png",
            target: Some("#layer-selection"),
            padding: 8.0,
            title: "Views of the network",
            text: html! {
                <p>
                    { "With the buttons above, you can switch between different views of the network: \
                       the data plane (forwarding state), the control plane (route propagation), \
                       the OSPF state, and the BGP configuration. Next to it, the prefix used \
                       throughout the demo is displayed — for this demo, you cannot change it." }
                </p>
            },
            on_enter: no_action,
        },
        // 2.1: right-click to add links / sessions
        TourStep {
            face: "./figures/talk5.png",
            target: Some("#canvas"),
            padding: 12.0,
            title: "Editing the topology",
            text: html! {
                <p>
                    { "The topology itself is editable: " }
                    <b> { "right-click on a router" } </b>
                    { " to add a physical link or a BGP session to another router, or to delete the \
                       router. Links and sessions appear in the sidebar of the affected routers, \
                       where they can also be removed. You may add additional internal routers \
                       but no external routers. You may also add internal routers with the + button above" }
                </p>
            },
            on_enter: no_action,
        },
        // 2: the router configuration in the sidebar
        TourStep {
            face: "./figures/talk4.png",
            target: Some("#sidebar-config"),
            padding: 8.0,
            title: "Router configuration",
            text: html! {
                <p>
                    { "By " }
                    <b> { "clicking on a router" } </b>
                    { ", its configuration is shown here, in the top half of the right sidebar: \
                       its name and AS number, the OSPF links with their weights, and its BGP sessions. \
                       We selected router R0 for you — try clicking a different router!\
                       Also, by scrolling down for every BGP session you can add " }
                    <b> { "incoming and outgoing route maps." } </b>
                </p>
            },
            on_enter: || select_router("R0"),
        },
        // 3: the forecast
        TourStep {
            face: "./figures/talk2.png",
            target: Some("#forecast"),
            padding: 8.0,
            title: "The forecast",
            text: html! {
                <p>
                    { "Below the configuration, you can set the " }
                    <b> { "routing forecast" } </b>
                    { ": the probability distribution over the routing inputs of the external networks. \
                       For this demo, only two external networks are considered, and the routing input \
                       of each is whether it advertises the prefix at all, and with which AS-path length \
                       (no route, length 1, or length 3). You can edit the two marginal distributions as \
                       histograms (the joint distribution assumes independence), or switch to \
                       the joint 3×3 matrix view." }
                </p>
            },
            on_enter: no_action,
        },
        // 4: run forecast + routing states
        TourStep {
            face: "./figures/talk3.png",
            target: Some("#run-forecast"),
            padding: 8.0,
            title: "Run the forecast",
            text: html! {
                <p>
                    { "Press " }
                    <b> { "Run forecast" } </b>
                    { " to compute the distribution over the routing states: for every combination of \
                       routing inputs, the simulator computes the resulting forwarding state of the \
                       network.\
                       Try it now!" }
                </p>
            },
            on_enter: no_action,
        },
        TourStep {
            face: "./figures/talk4.png",
            target: Some("#bottom-panel"),
            padding: 8.0,
            title: "The routing states",
            text: html! {
                <p>
                    { "Here you see all predicted routing states and their associated probability, \
                       ordered from the most likely (left) to the least likely (right). Each state is \
                       depicted as a mini version of the network: green arrows show the forwarding \
                       neighbour, and a red cross means that the router has no route. \
                       Click on a state to highlight it on the canvas, and use the arrows to browse \
                       through all states if there are more than six." }
                </p>
            },
            on_enter: || open_panel_tab(PanelTab::States),
        },
        // 5: the specifications (three steps)
        TourStep {
            face: "./figures/talk5.png",
            target: Some("#bottom-panel"),
            padding: 8.0,
            title: "Specifications",
            text: html! {
                <p>
                    { "In the " }
                    <b> { "Specification" } </b>
                    { " tab, each routing state is mapped to the outcome of a specification, and the \
                       probabilities of all states with the same outcome are added up. There are three \
                       specifications — I'll walk you through each of them." }
                </p>
            },
            on_enter: || open_panel_tab(PanelTab::Spec),
        },
        TourStep {
            face: "./figures/talk1.png",
            target: Some("#spec-description"),
            padding: 8.0,
            title: "1 — Fully connected",
            text: html! {
                <p>
                    { "The " }
                    <b> { "fully connected" } </b>
                    { " specification is true if and only if every internal router has a forwarding \
                       path towards the destination. The plot shows the probability that the property \
                       holds; in the routing-states view, each state is coloured green (true) or \
                       red (false)." }
                </p>
            },
            on_enter: || {
                open_panel_tab(PanelTab::Spec);
                Dispatch::<crate::state::State>::new()
                    .reduce_mut(|s| s.set_spec(Spec::FullyConnected));
            },
        },
        TourStep {
            face: "./figures/talk2.png",
            target: Some("#spec-description"),
            padding: 8.0,
            title: "2 — Load balanced",
            text: html! {
                <p>
                    { "The " }
                    <b> { "load balanced" } </b>
                    { " specification is true if and only if the two egress-adjacent routers R0 and R2 \
                       forward towards different egress routers." }
                </p>
            },
            on_enter: || {
                open_panel_tab(PanelTab::Spec);
                Dispatch::<crate::state::State>::new()
                    .reduce_mut(|s| s.set_spec(Spec::LoadBalanced));
            },
        },
        TourStep {
            face: "./figures/talk3.png",
            target: Some("#spec-description"),
            padding: 8.0,
            title: "3 — Maximum link utilization",
            text: html! {
                <p>
                    { "The " }
                    <b> { "MLU" } </b>
                    { " specification maps each routing state to its maximum link utilization: every \
                       internal router sends traffic along its forwarding path, and the outcome is the \
                       load of the most loaded link, normalized by the link bandwidth. The bandwidths \
                       and the traffic volumes are editable below the description, and the " }
                    <b> { "Recompute" } </b>
                    { " button applies them. The red line marks the expected value." }
                </p>
            },
            on_enter: || {
                open_panel_tab(PanelTab::Spec);
                Dispatch::<crate::state::State>::new()
                    .reduce_mut(|s| s.set_spec(Spec::Mlu));
            },
        },
        // the end
        TourStep {
            face: "./figures/exhausted.png",
            target: None,
            padding: 0.0,
            title: "You are ready! (and I'm exhausted)",
            text: html! {
                <p>
                    { "That's it! Edit the network configuration and the forecast, press " }
                    <b> { "Run forecast" } </b>
                    { ", and explore the routing states and the specifications. \
                       You can restart this tour at any time from the menu (top left). \
                       Note that the simulator behind this demo is fully interactive: every change \
                       you make is applied immediately." }
                </p>
            },
            on_enter: no_action,
        },
    ]
}

pub struct Tour {
    step: usize,
    /// Incremented on every tick, to recompute the highlight/popup positions.
    tick: u64,
    tour_complete: bool,
    _state_dispatch: Dispatch<State>,
}

pub enum Msg {
    State(Rc<State>),
    Tick,
    Next,
    Prev,
    Skip,
}

impl Component for Tour {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        let link = ctx.link().clone();
        // periodically re-measure the highlighted element, so that the
        // highlight follows layout changes (panel animations, scrolling, ...)
        let interval = gloo_timers::callback::Interval::new(250, move || {
            link.send_message(Msg::Tick);
        });
        interval.forget();
        let mut this = Self {
            step: 0,
            tick: 0,
            tour_complete: true,
            _state_dispatch,
        };
        this.enter_step(0);
        this
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                let changed = self.tour_complete != state.is_tour_complete();
                self.tour_complete = state.is_tour_complete();
                if changed && !self.tour_complete {
                    // the tour was (re-)started
                    self.enter_step(0);
                }
                changed
            }
            Msg::Tick => {
                self.tick += 1;
                true
            }
            Msg::Next => {
                if self.step + 1 >= steps().len() {
                    Dispatch::<State>::new().reduce_mut(|s| s.set_tour_complete());
                    true
                } else {
                    self.enter_step(self.step + 1);
                    true
                }
            }
            Msg::Prev => {
                self.enter_step(self.step.saturating_sub(1));
                true
            }
            Msg::Skip => {
                Dispatch::<State>::new().reduce_mut(|s| s.set_tour_complete());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.tour_complete {
            return html! {};
        }
        let steps = steps();
        let step = &steps[self.step.min(steps.len() - 1)];
        let is_last = self.step + 1 >= steps.len();

        // compute the highlight rectangle and the position of the speech
        // bubble group (Paul's face + the bubble)
        let (highlight_style, group_style) = match step.target {
            Some(selector) => {
                let target = gloo_utils::document()
                    .query_selector(selector)
                    .ok()
                    .flatten();
                match target.and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok()) {
                    Some(el) => {
                        let rect = el.get_bounding_client_rect();
                        let (win_w, win_h) = window_size();
                        // clamp to the window (full-area targets start at the
                        // very edge, so no margin here)
                        let x = (rect.left() - step.padding).max(0.0);
                        let y = (rect.top() - step.padding).max(0.0);
                        let w = (rect.width() + 2.0 * step.padding).min(win_w - x);
                        let h = (rect.height() + 2.0 * step.padding).min(win_h - y);
                        let highlight = format!("left: {x}px; top: {y}px; width: {w}px; height: {h}px;");
                        // place the group (Paul + bubble) below the target if
                        // there is space, otherwise above it; centred on the
                        // target, clamped to the window
                        let group_w = FACE_SIZE + GAP + BUBBLE_WIDTH;
                        let group_h_estimate = 260.0;
                        let cx = x + w / 2.0;
                        let left = (cx - group_w / 2.0 + FACE_SIZE / 2.0)
                            .clamp(8.0, win_w - group_w - 8.0);
                        let below = y + h + 16.0 + group_h_estimate < win_h;
                        let top = if below {
                            y + h + 16.0
                        } else {
                            (y - 16.0 - group_h_estimate).max(8.0)
                        };
                        let group = format!("left: {left}px; top: {top}px;");
                        (highlight, group)
                    }
                    None => (String::new(), centred_group()),
                }
            }
            None => (String::new(), centred_group()),
        };

        let _ = step.padding;

        html! {
            <>
                // the spotlight: a rectangle at the target position whose huge
                // box-shadow dims everything around it. The rectangle itself is
                // fully transparent, so the highlighted area keeps its normal
                // brightness and everything on the page stays clickable.
                if !highlight_style.is_empty() {
                    <div class="absolute z-30 rounded-xl pointer-events-none"
                         style={format!("{highlight_style} box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.55);")} />
                } else {
                    <div class="absolute z-30 pointer-events-none"
                         style="left: 0; top: 0; width: 100vw; height: 100vh; background: rgba(0, 0, 0, 0.55);" />
                }
                <div class="absolute z-30 flex items-start gap-3" style={group_style}>
                    <img class="shrink-0 select-none pointer-events-none drop-shadow-lg"
                         src={step.face} alt="Paul, the network engineer"
                         style={format!("width: {FACE_SIZE}px;")} />
                    <div class="relative shadow-md bg-base-1 p-6 text-main flex flex-col gap-4"
                         style={format!("width: {BUBBLE_WIDTH}px; border-radius: 1rem;")}>
                        // the triangle of the speech bubble, pointing to Paul's mouth
                        <div class="absolute w-0 h-0"
                             style="left: -14px; top: 88px; width: 0; height: 0; border-top: 14px solid transparent; border-bottom: 14px solid transparent; border-right: 14px solid rgb(var(--color-base1));" />
                        <p class="text-lg font-semibold">{ step.title }</p>
                        <div class="text-sm">{ step.text.clone() }</div>
                    <div class="flex flex-row items-center gap-2">
                        if self.step > 0 {
                            <button class="rounded-md py-2 px-4 border border-base-4 bg-base-2 hover:bg-base-3 transition duration-150 ease-in-out"
                                    onclick={ctx.link().callback(|_| Msg::Prev)}>
                                { "Back" }
                            </button>
                        }
                        <div class="flex-1" />
                        <button class="text-main-ia hover:text-main transition duration-150 ease-in-out mr-2"
                                onclick={ctx.link().callback(|_| Msg::Skip)}>
                            { "Skip tour" }
                        </button>
                        <button class="rounded-md py-2 px-4 shadow-md border border-base-4 bg-base-2 hover:bg-base-3 transition duration-150 ease-in-out"
                                onclick={ctx.link().callback(|_| Msg::Next)}>
                            if is_last {
                                { "Finish" }
                            } else {
                                { "Next" }
                            }
                        </button>
                    </div>
                    <p class="text-xs text-main-ia text-right">
                        { format!("{} / {}", self.step + 1, steps.len()) }
                    </p>
                    </div>
                </div>
            </>
        }
    }
}

impl Tour {
    /// Enter the given step and run its action.
    fn enter_step(&mut self, step: usize) {
        self.step = step;
        let steps = steps();
        if let Some(step) = steps.get(step) {
            (step.on_enter)();
        }
    }
}

fn window_size() -> (f64, f64) {
    let w = gloo_utils::window();
    (
        w.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1200.0),
        w.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(800.0),
    )
}

/// The size of Paul's face in the tour (approximately half the size of the
/// first text bubble).
const FACE_SIZE: f64 = 160.0;
/// The width of the speech bubble.
const BUBBLE_WIDTH: f64 = 448.0;
/// The gap between Paul's face and the speech bubble.
const GAP: f64 = 12.0;

fn centred_group() -> String {
    let (win_w, win_h) = window_size();
    let group_w = FACE_SIZE + GAP + BUBBLE_WIDTH;
    let left = (win_w - group_w) / 2.0;
    let top = (win_h - 280.0) / 2.0;
    format!("left: {left}px; top: {top}px;")
}
