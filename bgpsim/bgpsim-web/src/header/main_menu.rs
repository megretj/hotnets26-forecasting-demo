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

use yew::prelude::*;
use yewdux::prelude::*;

use crate::{callback, sidebar::Toggle, state::{Hover, State}};

#[derive(Properties, PartialEq)]
pub struct Properties {
    pub node_ref: NodeRef,
}

#[function_component]
pub fn MainMenu(props: &Properties) -> Html {
    let state_dispatch = Dispatch::<State>::new();

    // define the state
    let shown = use_state(|| false);
    let s = use_selector(|state: &State| (state.is_theme_forced(), state.is_dark_mode()));
    let (forced_theme, dark_mode) = (s.0, s.1);

    // define all callbacks
    let show = callback!(shown -> move |_| shown.set(true));
    let hide = callback!(shown -> move |_| shown.set(false));
    let restart_tour = callback!(shown, state_dispatch -> move |_| {
        shown.set(false);
        state_dispatch.reduce_mut(|s| s.reset_tour_complete());
    });
    let launch_tour = callback!(state_dispatch -> move |_| {
        state_dispatch.reduce_mut(|s| s.reset_tour_complete());
    });
    let show_tour_tooltip = state_dispatch.reduce_mut_callback(|s| {
        s.set_hover(Hover::Help(html! { "launch the tour" }));
    });
    let hide_tour_tooltip = state_dispatch.reduce_mut_callback(|s| s.clear_hover());
    let reset_demo = callback!(shown -> move |_| {
        shown.set(false);
        crate::net::demo::load_demo_network();
    });

    let on_dark_mode_toggle = state_dispatch.reduce_mut_callback(|s| s.toggle_dark_mode());

    let dark_mode_symbol = if dark_mode {
        html! {<yew_lucide::Sun />}
    } else {
        html! {<yew_lucide::Moon />}
    };

    let button_class = "absolute z-10 rounded-full mt-4 ml-4 p-2 drop-shadow bg-blue text-base-1 hover:bg-blue-dark focus:bg-blue active:bg-blue-darker transition duration-150 ease-in-out";
    let tour_button_class = "absolute z-10 rounded-full mt-[50px] ml-4 p-1.5 transition duration-150 ease-in-out overflow-hidden";
    let bg_class = "absolute z-20 h-screen w-screen bg-black bg-opacity-0 peer-checked:bg-opacity-30 pointer-events-none peer-checked:pointer-events-auto cursor-default focus:outline-none transition duration-300 ease-in-out";
    let sidebar_class = "absolute z-20 h-screen -left-96 w-96 bg-base-1 peer-checked:opacity-100 pointer-events-none peer-checked:pointer-events-auto peer-checked:translate-x-full transition duration-300 ease-in-out overflow-auto";
    let link_class = "border-b border-base-4 hover:border-blue-dark hover:text-blue-dark transition duration-150 ease-in-out";
    let element_class = "w-full flex items-center py-4 px-6 h-12 overflow-hidden text-main text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-2 transition duration-200 ease-in-out cursor-pointer active:ring-none";
    let target = "_blank";

    html! {
        <>
            <input type="checkbox" value="" class="sr-only peer" checked={*shown}/>
            <button class={button_class} onclick={show} ref={props.node_ref.clone()}> <yew_lucide::Menu class="w-6 h-6" /> </button>
            <button class={tour_button_class} title="launch the tour" onclick={launch_tour}
                    onmouseenter={show_tour_tooltip} onmouseleave={hide_tour_tooltip}>
                <img class="w-9 h-9 object-contain" src="./figures/happy.png" alt="Paul, the network engineer" draggable="false" />
            </button>
            <button class={bg_class} onclick={hide}> </button>
            <div class={sidebar_class}>
                <div class="flex-1 flex justify-end">
                    if forced_theme {
                        <div class="m-2 text-base-1">{ dark_mode_symbol }</div>
                    } else {
                        <div class="cursor-pointer m-2" onclick={on_dark_mode_toggle}>{ dark_mode_symbol }</div>
                    }
                </div>
                <div class="flex-1 flex flex-col items-center justify-center pt-2 pb-10 px-4">
                    <a class="text-lg font-semibold text-main hover:text-blue transition duration-150 ease-in-out text-center"
                       href="https://nsg.ee.ethz.ch/publications/2026-11-16-forecasting-routing-inputs-20-500-11850-xxxxxx/" {target}>
                        { "Forecasting routing inputs demo" }
                    </a>
                    <p class="text-sm mt-2">
                        { "by " }
                        <a class={link_class} href="https://nsg.ee.ethz.ch/people/jean-megret/" {target}>{ "Jean Mégret @ NSG" }</a>
                    </p>
                    <p class="text-sm text-main-ia mt-6 text-center">
                        { "built with " }
                        <a class={link_class} href="https://bgpsim.github.io/" {target}>{ "bgpsim" }</a>
                        { " by " }
                        <a class={link_class} href="https://nsg.ee.ethz.ch/people/tibor-schneider/" {target}>{ "Tibor Schneider @ NSG" }</a>
                    </p>
                </div>
                <div class="p-2 flex flex-col space-y-2">
                    <button class={element_class} onclick={reset_demo}>
                        <yew_lucide::RotateCcw class="h-6 mr-4" />
                        {"Reset demo network"}
                    </button>
                    <button class={element_class} onclick={restart_tour}>
                        <yew_lucide::HelpCircle class="h-6 mr-4" />
                        {"Restart the tour"}
                    </button>
                    <FeatureSettings main_class={element_class} />
                </div>
            </div>
        </>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct FeatureSettingsProps {
    main_class: &'static str,
}

#[function_component(FeatureSettings)]
fn feature_settings(props: &FeatureSettingsProps) -> Html {
    let visible = use_state(|| false);
    let toggle_show = {
        let visible = visible.clone();
        Callback::from(move |_| visible.set(!*visible))
    };
    let element_class = "w-full flex items-center py-4 px-6 h-8 overflow-hidden text-main text-ellipsis whitespace-nowrap rounded hover:text-blue hover:bg-base-3 transition duration-200 ease-in-out cursor-pointer active:ring-none";

    let (state, dispatch) = use_store::<State>();

    let toggle_ospf = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().ospf;
        s.features_mut().ospf = !old_val;
    });

    let toggle_bgp = dispatch.reduce_mut_callback(|s| {
        let old_val = s.features().bgp;
        s.features_mut().bgp = !old_val;
    });

    html! {
        <>
            <button class={props.main_class} onclick={toggle_show}>
                if *visible {
                    <yew_lucide::ChevronDown class="h-6 mr-4" />
                } else {
                    <yew_lucide::Wrench class="h-6 mr-4" />
                }
                {"Settings"}
            </button>
            if *visible {
                <div class= "w-full flex flex-col py-2 px-2 rounded bg-base-2">
                    <button class={element_class} onclick={toggle_ospf}>
                        <p class="flex-1 text-left ml-8">{"OSPF"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().ospf}/>
                        </div>
                    </button>
                    <button class={element_class} onclick={toggle_bgp}>
                        <p class="flex-1 text-left ml-8">{"BGP"}</p>
                        <div class="pointer-events-none flex flex-row-reverse mt-2">
                            <Toggle text={""} on_click={Callback::from(|_| ())} checked={state.features().bgp}/>
                        </div>
                    </button>
                </div>
            }
        </>

    }
}
