// BgpSim: BGP Network Simulator written in Rust
// Copyright (C) 2022-2023 Tibor Schneider <sctibor@ethz.ch>
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::{collections::HashSet, rc::Rc};

use bgpsim::{
    formatter::NetworkFormatter,
    route_map::{
        RouteMap, RouteMapBuilder,
        RouteMapDirection::{self, Incoming, Outgoing},
    },
    types::{RouterId, ASN},
};
use wasm_bindgen::JsCast;
use web_sys::{Element as WebElement, HtmlInputElement, InputEvent, KeyboardEvent};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::{
    net::{Net, Pfx},
    state::{Flash, State},
};

use super::super::{Divider, Element, ExpandableDivider, Select};
use super::route_map_item_cfg::RouteMapCfg;

pub struct RouteMapsCfg {
    net: Rc<Net>,
    net_dispatch: Dispatch<Net>,
    state: Rc<State>,
    _state_dispatch: Dispatch<State>,
    router: RouterId,
    /// The text typed into the "New route map" order fields, and whether the
    /// entered order is valid (parses to i16 and is not used yet). These are
    /// managed here (not via the shared TextField component), so that
    /// unrelated re-renders cannot reset the validity.
    order_in: String,
    order_out: String,
    order_in_correct: bool,
    order_out_correct: bool,
    rm_neighbor: Option<RouterId>,
    flash_changed: bool,
    in_ref: NodeRef,
    out_ref: NodeRef,
}

pub enum Msg {
    StateNet(Rc<Net>),
    State(Rc<State>),
    ChooseRMNeighbor(RouterId),
    UpdateRM(RouterId, i16, Option<RouteMap<Pfx>>, RouteMapDirection),
    SetOrderIn(String),
    SetOrderOut(String),
    AddRM(RouterId, RouteMapDirection),
}

#[derive(Properties, PartialEq, Eq)]
pub struct Properties {
    pub router: RouterId,
    pub asn: ASN,
    pub bgp_peers: Vec<(RouterId, String)>,
    pub disabled: Option<bool>,
}

impl Component for RouteMapsCfg {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let net_dispatch = Dispatch::<Net>::subscribe(ctx.link().callback(Msg::StateNet));
        let _state_dispatch = Dispatch::<State>::subscribe(ctx.link().callback(Msg::State));
        RouteMapsCfg {
            net: Default::default(),
            net_dispatch,
            state: Default::default(),
            _state_dispatch,
            router: ctx.props().router,
            order_in: String::new(),
            order_out: String::new(),
            order_in_correct: false,
            order_out_correct: false,
            rm_neighbor: None,
            flash_changed: false,
            in_ref: NodeRef::default(),
            out_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let router = ctx.props().router;
        let asn = ctx.props().asn;
        let disabled = ctx.props().disabled.unwrap_or(false);
        let n = &self.net.net();
        let r = if let Ok(r) = n.get_router(router) {
            r
        } else {
            return html! {};
        };

        let bgp_peers = ctx.props().bgp_peers.clone();

        if let Some(neighbor) = self
            .rm_neighbor
            .or_else(|| bgp_peers.first().map(|(x, _)| *x))
        {
            let on_in_order_input = ctx
                .link()
                .callback(|e: InputEvent| Msg::SetOrderIn(input_value(&e)));
            let on_in_route_map_add = ctx.link().callback(move |_| Msg::AddRM(neighbor, Incoming));
            let incoming_rms: Vec<(i16, RouteMap<Pfx>)> = r
                .bgp
                .get_route_maps(neighbor, Incoming)
                .iter()
                .map(|r| (r.order, r.clone()))
                .collect();
            let incoming_existing: Rc<HashSet<i16>> =
                Rc::new(incoming_rms.iter().map(|(o, _)| *o).collect());

            let on_out_order_input = ctx
                .link()
                .callback(|e: InputEvent| Msg::SetOrderOut(input_value(&e)));
            let on_out_route_map_add = ctx.link().callback(move |_| Msg::AddRM(neighbor, Outgoing));
            let outgoing_rms: Vec<(i16, RouteMap<Pfx>)> = r
                .bgp
                .get_route_maps(neighbor, Outgoing)
                .iter()
                .map(|r| (r.order, r.clone()))
                .collect();
            let outgoing_existing: Rc<HashSet<i16>> =
                Rc::new(outgoing_rms.iter().map(|(o, _)| *o).collect());

            let help = html! {<p>{"Route maps are configured per neighbor. Select a neighbor to configure route-maps from that neighbor."}</p>};

            // Handle the flashing

            let (in_ref, out_ref) = (self.in_ref.clone(), self.out_ref.clone());
            // the route-map sections are expanded by default, so that the
            // "New route map" field is immediately visible; the flash logic
            // (Some(..)) is kept for scroll-into-view after adding a route
            // map from the data-plane view.
            let default_shown = Some(true);
            let (shown_in, shown_out) =
                if let Some(Flash::RouteMap(src, _, dir)) = self.state.get_flash() {
                    if src == ctx.props().router {
                        match dir {
                            Incoming => (default_shown, None),
                            Outgoing => (None, default_shown),
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

            let flash_class = "ring-4 ring-blue ring-offset-4 ring-offset-base-2";
            let normal_class = "w-full p-0 m-0 transition duration-300 ease-in-out rounded-lg";
            let in_box_class = if shown_in == Some(true) {
                classes!(normal_class, flash_class)
            } else {
                classes!(normal_class)
            };
            let out_box_class = if shown_out == Some(true) {
                classes!(normal_class, flash_class)
            } else {
                classes!(normal_class)
            };

            let input_class = "flex-1 w-16 px-3 text-base font-normal bg-base-1 bg-clip-padding border border-solid border-base-5 rounded transition ease-in-out m-0 focus:outline-none focus:border-blue";
            let add_btn_class = |correct: bool| {
                if correct && !disabled {
                    classes! {"ml-2", "px-2", "flex-none", "text-main", "rounded", "shadow-md", "hover:shadow-lg", "transition", "ease-in-out", "border", "border-base-5", "focus:border-blue", "focus:outline-none", "bg-base-2"}
                } else {
                    classes! {"ml-2", "px-2", "flex-none", "rounded", "bg-base-1", "transition", "ease-in-out", "border", "focus:outline-none", "text-base-4", "border-base-2", "cursor-not-allowed"}
                }
            };

            html! {
                <>
                    <Divider text={"BGP Route-Maps"} />
                    <div class="w-full space-y-2 bg-base-2 p-4 rounded-md shadow-md" id="route-maps">
                        <Element text={"Neighbor"} {help}>
                            <Select<RouterId> text={neighbor.fmt(n).to_string()} options={bgp_peers} on_select={ctx.link().callback(Msg::ChooseRMNeighbor)} />
                        </Element>
                    <ExpandableDivider text={String::from("Incoming Route Map")} padding_top={false} shown={shown_in.or(default_shown)} >
                            <Element text={"New route map"} >
                                <div class="w-full flex">
                                    <input type="text" class={input_class} placeholder="order"
                                           value={self.order_in.clone()}
                                           oninput={on_in_order_input}
                                           onkeypress={ctx.link().batch_callback(move |e: KeyboardEvent| (e.code() == "Enter").then_some(Msg::AddRM(neighbor, Incoming)))}
                                           {disabled} />
                                    <button class={add_btn_class(self.order_in_correct)}
                                            disabled={!self.order_in_correct || disabled}
                                            onclick={on_in_route_map_add}>
                                        { "Add" }
                                    </button>
                                </div>
                            </Element>
                            <div class={in_box_class} ref={in_ref}>
                                {
                                    incoming_rms.into_iter().map(|(order, map)|  {
                                        let on_update = ctx.link().callback(move |(order, map)| Msg::UpdateRM(neighbor, order, Some(map), Incoming));
                                        let on_remove = ctx.link().callback(move |order| Msg::UpdateRM(neighbor, order, None, Incoming));
                                        html!{ <RouteMapCfg {router} {asn} {neighbor} {order} {map} existing={incoming_existing.clone()} {on_update} {on_remove} {disabled}/> }
                                    }).collect::<Html>()
                                }
                            </div>
                        </ExpandableDivider>
                    <ExpandableDivider text={String::from("Outgoing Route Map")} shown={shown_out.or(default_shown)}>
                            <Element text={"New route map"} >
                                <div class="w-full flex">
                                    <input type="text" class={input_class} placeholder="order"
                                           value={self.order_out.clone()}
                                           oninput={on_out_order_input}
                                           onkeypress={ctx.link().batch_callback(move |e: KeyboardEvent| (e.code() == "Enter").then_some(Msg::AddRM(neighbor, Outgoing)))}
                                           {disabled} />
                                    <button class={add_btn_class(self.order_out_correct)}
                                            disabled={!self.order_out_correct || disabled}
                                            onclick={on_out_route_map_add}>
                                        { "Add" }
                                    </button>
                                </div>
                            </Element>
                            <div class={out_box_class} ref={out_ref}>
                                {
                                    outgoing_rms.into_iter().map(|(order, map)| {
                                        let on_update = ctx.link().callback(move |(order, map)| Msg::UpdateRM(neighbor, order, Some(map), Outgoing));
                                        let on_remove = ctx.link().callback(move |order| Msg::UpdateRM(neighbor, order, None, Outgoing));
                                        html!{ <RouteMapCfg {router} {asn} {neighbor} {order} {map} existing={outgoing_existing.clone()} {on_update} {on_remove} {disabled}/> }
                                    }).collect::<Html>()
                                }
                            </div>
                        </ExpandableDivider>
                    </div>
                </>
            }
        } else {
            html!()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let router = ctx.props().router;
        match msg {
            Msg::StateNet(n) => {
                self.net = n;
                true
            }
            Msg::State(state) => {
                self.state = state;
                if let Some(Flash::RouteMap(src, dst, dir)) = self.state.get_flash() {
                    if (!self.flash_changed) && router == src {
                        self.flash_changed = true;
                        self.rm_neighbor = Some(dst);
                        if let Some(node_ref) = match dir {
                            Incoming => self.in_ref.cast::<WebElement>(),
                            Outgoing => self.out_ref.cast::<WebElement>(),
                        } {
                            node_ref.scroll_into_view();
                        }
                    }
                } else {
                    self.flash_changed = false;
                }
                true
            }
            Msg::ChooseRMNeighbor(neighbor) => {
                self.rm_neighbor = Some(neighbor);
                // reset the order fields when switching neighbors
                self.order_in.clear();
                self.order_out.clear();
                self.order_in_correct = false;
                self.order_out_correct = false;
                true
            }
            Msg::SetOrderIn(o) => {
                self.order_in = o;
                self.order_in_correct = self.order_valid(Incoming);
                true
            }
            Msg::SetOrderOut(o) => {
                self.order_out = o;
                self.order_out_correct = self.order_valid(Outgoing);
                true
            }
            Msg::AddRM(neighbor, direction) => {
                let text = match direction {
                    Incoming => self.order_in.clone(),
                    Outgoing => self.order_out.clone(),
                };
                let Ok(o) = text.parse::<i16>() else {
                    return false;
                };
                if self.net.net().get_router(router).is_ok_and(|r| {
                    r.bgp.get_route_map(neighbor, direction, o).is_some()
                }) {
                    return false;
                }
                let rm = RouteMapBuilder::new().order_sgn(o).allow().build();
                self.net_dispatch.reduce_mut(move |n| {
                    n.net_mut()
                        .set_bgp_route_map(router, neighbor, direction, rm)
                        .unwrap()
                });
                // clear the field
                match direction {
                    Incoming => {
                        self.order_in.clear();
                        self.order_in_correct = false;
                    }
                    Outgoing => {
                        self.order_out.clear();
                        self.order_out_correct = false;
                    }
                }
                true
            }
            Msg::UpdateRM(neighbor, order, map, direction) => {
                self.net_dispatch.reduce_mut(move |n| {
                    if let Some(map) = map {
                        if order != map.order {
                            n.net_mut()
                                .remove_bgp_route_map(router, neighbor, direction, order)
                                .unwrap();
                        }
                        n.net_mut()
                            .set_bgp_route_map(router, neighbor, direction, map)
                            .unwrap();
                    } else {
                        n.net_mut()
                            .remove_bgp_route_map(router, neighbor, direction, order)
                            .unwrap();
                    }
                });
                false
            }
        }
    }
}

impl RouteMapsCfg {
    fn current_neighbor(&self) -> Option<RouterId> {
        self.rm_neighbor.or_else(|| {
            self.net
                .net()
                .get_router(self.router)
                .ok()
                .and_then(|r| {
                    r.bgp.get_sessions().keys().copied().min_by_key(|id| {
                        self.net
                            .net()
                            .get_router(*id)
                            .map(|x| x.name().to_string())
                            .unwrap_or_default()
                    })
                })
        })
    }

    /// Whether the text typed into the order field of the given direction is
    /// a valid, not-yet-used order.
    fn order_valid(&self, direction: RouteMapDirection) -> bool {
        let Some(neighbor) = self.current_neighbor() else {
            return false;
        };
        let Ok(o) = (match direction {
            Incoming => self.order_in.clone(),
            Outgoing => self.order_out.clone(),
        })
        .parse::<i16>() else {
            return false;
        };
        self.net
            .net()
            .get_router(self.router)
            .is_ok_and(|r| r.bgp.get_route_map(neighbor, direction, o).is_none())
    }
}

/// Read the current value of the input element that triggered the event.
fn input_value(e: &InputEvent) -> String {
    e.target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .map(|i| i.value())
        .unwrap_or_default()
}
