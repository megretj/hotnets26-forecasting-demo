// The Forecast tab of the bottom panel: edit the probability distribution
// over the AS-path lengths announced by the two external networks. Two
// views: independent marginals (histograms, joint computed by product) and
// the joint 3x3 matrix. Both are drag-editable; switching views resets the
// distribution to uniform.

use yew::prelude::*;
use yewdux::prelude::*;

use crate::forecast::{DistView, Forecast, INPUT_LABELS};

#[function_component]
pub fn ForecastTab() -> Html {
    let s = use_selector(|f: &Forecast| (f.view(), f.matrix(), f.marginal1(), f.marginal2()));
    let (view, matrix, marginal1, marginal2) = (s.0, s.1, s.2, s.3);

    let reset = Dispatch::<Forecast>::new().reduce_mut_callback(|f| f.reset());
    let run = Dispatch::<Forecast>::new().reduce_mut_callback(|f| f.run_forecast());
    let run_button_class = classes!(
        "flex",
        "items-center",
        "gap-2",
        "px-3",
        "py-1",
        "rounded-md",
        "text-sm",
        "font-semibold",
        "bg-blue",
        "text-base-1",
        "hover:bg-blue-dark",
        "transition",
        "duration-150",
        "ease-in-out",
    );

    let view_button = |v: DistView| {
        let active = v == view;
        let class = classes!(
            "px-3",
            "py-1",
            "rounded-md",
            "text-xs",
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
            Dispatch::<Forecast>::new().reduce_mut_callback(move |f| f.set_view(v));
        html! {
            <button class={class} {onclick}>{ v.label() }</button>
        }
    };

    let randomize = Dispatch::<Forecast>::new().reduce_mut_callback(|f| f.randomize());
    let randomize_button = |show: bool| {
        if !show {
            return html!();
        }
        html! {
            <button class="w-full py-1.5 rounded-md text-sm font-semibold bg-blue text-base-1 hover:bg-blue-dark transition duration-150 ease-in-out flex items-center justify-center gap-2"
                    onclick={randomize}
                    title="Draw a random number between 0 and 100 for every cell and normalize">
                <yew_lucide::Dices class="w-4 h-4" />
                { "Generate random distribution" }
            </button>
        }
    };

    let content = match view {
        DistView::Marginals => html! {
            <div class="w-full h-full flex flex-col gap-2 p-2 min-h-0">
                <Histogram title={"External network 1 (E1)"} external={1} values={marginal1} />
                <Histogram title={"External network 2 (E2)"} external={2} values={marginal2} />
            </div>
        },
        DistView::Joint => html! {
            <div class="w-full h-full flex flex-col p-2 gap-2 min-h-0">
                <div class="flex-1 min-h-0">
                    <JointMatrix {matrix} />
                </div>
                { randomize_button(true) }
            </div>
        },
    };

    html! {
        <div class="w-full h-full flex flex-col">
            <div class="flex items-center gap-2 px-4 py-2 border-b border-base-3 shrink-0 flex-wrap">
                <button class={run_button_class} id="run-forecast" onclick={run} title="Compute the routing states and the specification outcomes">
                    <yew_lucide::Play class="w-4 h-4" />
                    { "Run forecast" }
                </button>
                <div class="flex bg-base-2 rounded-md p-0.5">
                    { view_button(DistView::Marginals) }
                    { view_button(DistView::Joint) }
                </div>
                <p class="text-sm text-main-ia whitespace-nowrap overflow-hidden text-ellipsis">
                    { "Drag the bars/cells to edit." }
                </p>
                <div class="flex-1" />
                <button class="text-main-ia hover:text-blue transition duration-150 ease-in-out"
                        onclick={reset}
                        title="Reset to uniform">
                    <yew_lucide::RotateCcw class="w-4 h-4" />
                </button>
            </div>
            <div class="flex-1 min-h-0">
                { content }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct HistogramProps {
    title: &'static str,
    /// Which external network this histogram belongs to (1 or 2).
    external: usize,
    values: [f64; 3],
}

#[function_component]
fn Histogram(props: &HistogramProps) -> Html {
    let bars = props.values.iter().enumerate().map(|(i, &p)| {
        let external = props.external;
        let on_change = Dispatch::<Forecast>::new()
            .reduce_mut_callback_with(move |f: &mut Forecast, target: f64| match external {
                1 => f.set_marginal1(i, target),
                _ => f.set_marginal2(i, target),
            });
        html! {
            <div class="flex flex-col items-center gap-0.5 flex-1 min-h-0">
                <DragValue value={p} class="w-full h-full" fill_class="bg-blue" {on_change}>
                    <span class="text-sm font-semibold">{ format!("{:.0}%", p * 100.0) }</span>
                </DragValue>
                <span class="text-sm text-main font-medium">
                    { crate::forecast::INPUT_LABELS[i] }
                </span>
            </div>
        }
    }).collect::<Html>();

    html! {
        <div class="flex flex-col gap-1 flex-1 min-h-0">
            <p class="text-sm font-semibold">{ props.title }</p>
            <div class="flex gap-3 flex-1 min-h-0">{ bars }</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct JointMatrixProps {
    matrix: [[f64; 3]; 3],
}

#[function_component]
fn JointMatrix(props: &JointMatrixProps) -> Html {
    let header_cell = |text: String| {
        html! {
            <div class="flex items-center justify-center text-sm text-main font-medium">
                { text }
            </div>
        }
    };

    let rows = (0..3).map(|i| {
        let cells = (0..3).map(|j| {
            let p = props.matrix[i][j];
            let on_change = Dispatch::<Forecast>::new()
                .reduce_mut_callback_with(move |f: &mut Forecast, target: f64| {
                    f.set_cell(i, j, target)
                });
            html! {
                <DragValue value={p} class="w-full h-full rounded" fill_class="bg-blue" {on_change}>
                    <span class="text-base font-semibold">{ format!("{:.0}%", p * 100.0) }</span>
                </DragValue>
            }
        }).collect::<Html>();
        html! {
            <>
                { header_cell(INPUT_LABELS[i].into()) }
                { cells }
            </>
        }
    }).collect::<Html>();

    html! {
        <div class="w-full h-full flex flex-col gap-2">
            <p class="text-center text-base font-semibold">
                { "len of E2 →" }
            </p>
            <div class="flex-1 min-h-0 grid gap-2"
                 style="grid-template-columns: 1.75rem 3.5rem repeat(3, 1fr); grid-template-rows: 1.75rem repeat(3, 1fr);">
                <div></div>
                <div></div>
                { header_cell(INPUT_LABELS[0].into()) }
                { header_cell(INPUT_LABELS[1].into()) }
                { header_cell(INPUT_LABELS[2].into()) }
                <div class="flex items-center justify-center text-base font-semibold"
                     style="grid-row: 2 / span 3; writing-mode: vertical-rl; transform: rotate(180deg);">
                    { "len of E1" }
                </div>
                { rows }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct DragValueProps {
    /// The current probability (0 to 1); rendered as a bottom-anchored fill.
    value: f64,
    /// Called with the new value while dragging (0 to 1, bottom = 0).
    on_change: Callback<f64>,
    class: Classes,
    fill_class: Classes,
    children: Children,
}

/// A container whose probability can be edited by dragging the mouse
/// vertically (bottom = 0, top = 1). Renders a bottom-anchored fill of the
/// given height, with arbitrary children on top.
#[function_component]
fn DragValue(props: &DragValueProps) -> Html {
    // NOTE: yew 0.20 uses event delegation, so the event's current_target is
    // not this element. We use a NodeRef to get the geometry of this div.
    let div_ref = use_node_ref();
    let div_ref_handler = div_ref.clone();
    let on_change = props.on_change.clone();
    let update_from_event = move |e: &MouseEvent| {
        let Some(el) = div_ref_handler.cast::<web_sys::HtmlElement>() else {
            return;
        };
        let rect = el.get_bounding_client_rect();
        if rect.height() <= 0.0 {
            return;
        }
        let frac = 1.0 - (e.client_y() as f64 - rect.top()) / rect.height();
        on_change.emit(frac.clamp(0.0, 1.0));
    };
    let onmousedown = {
        let update = update_from_event.clone();
        move |e: MouseEvent| update(&e)
    };
    let onmousemove = move |e: MouseEvent| {
        if e.buttons() & 1 != 0 {
            update_from_event(&e);
        }
    };

    html! {
        <div ref={div_ref}
             class={classes!("relative", "overflow-hidden", "bg-base-2", "select-none", "cursor-ns-resize", props.class.clone())}
             {onmousedown} {onmousemove}>
            <div class={classes!("absolute", "bottom-0", "left-0", "right-0", props.fill_class.clone())}
                 style={format!("height: {}%", props.value * 100.0)} />
            <div class="absolute inset-0 flex items-center justify-center">
                { for props.children.iter() }
            </div>
        </div>
    }
}
