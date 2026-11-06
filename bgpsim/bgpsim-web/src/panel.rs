// Bottom panel of the forecasting demo: a collapsible panel below the
// network canvas with tabs (Routing States / Specification).

use yew::prelude::*;
use yewdux::prelude::*;

use crate::state::{PanelTab, State};

/// Height of the collapsed panel (just the bar), in Tailwind classes.
const COLLAPSED_CLASS: &str = "h-11";
/// Height of the open panel.
const OPEN_CLASS: &str = "h-[45vh]";

#[function_component]
pub fn BottomPanel() -> Html {
    let s = use_selector(|s: &State| (s.panel_open(), s.panel_tab()));
    let (panel_open, active_tab) = *s;

    let toggle = Dispatch::<State>::new().reduce_mut_callback(|s| s.toggle_panel());
    let tabs: Html = [PanelTab::States, PanelTab::Spec]
        .into_iter()
        .map(|tab| {
            let active = tab == active_tab;
            let class = classes!(
                "h-full",
                "px-4",
                "text-sm",
                "whitespace-nowrap",
                "transition",
                "duration-150",
                "ease-in-out",
                if active { "text-blue" } else { "text-main hover:text-blue" },
                if active {
                    "border-b-2 border-blue"
                } else {
                    "border-b-2 border-transparent hover:border-base-4"
                },
            );
            let onclick = Dispatch::<State>::new().reduce_mut_callback(move |s| s.set_panel_tab(tab));
            html! {
                <button class={class} {onclick}>
                    { tab.label() }
                </button>
            }
        })
        .collect::<Html>();

    let container_class = classes!(
        "relative",
        "w-full",
        "flex",
        "flex-col",
        "z-10",
        "bg-base-0",
        "border-t",
        "border-base-4",
        "rounded-t-lg",
        "shadow-2xl",
        "transition-all",
        "duration-300",
        "ease-in-out",
        if panel_open { OPEN_CLASS } else { COLLAPSED_CLASS },
    );

    let content: Html = match active_tab {
        PanelTab::States => html! {
            <crate::forecast::states_tab::StatesTab />
        },
        PanelTab::Spec => html! {
            <crate::forecast::SpecTab />
        },
    };

    html! {
        <div class={container_class} id="bottom-panel">
            <div class="flex items-stretch h-11 shrink-0 border-b border-base-3">
                { tabs }
                <div class="flex-1" />
                <button class="px-3 text-main hover:text-blue transition duration-150 ease-in-out" onclick={toggle}>
                    if panel_open {
                        <yew_lucide::ChevronDown class="w-5 h-5" />
                    } else {
                        <yew_lucide::ChevronUp class="w-5 h-5" />
                    }
                </button>
            </div>
            <div class="flex-1 min-h-0 overflow-auto">
                { content }
            </div>
        </div>
    }
}

