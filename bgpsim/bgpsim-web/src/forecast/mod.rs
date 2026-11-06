// Forecast store: the probability distribution over the external routing
// inputs. The routing input of each external network is its AS-path length
// (1, 2 or 3). The distribution is stored as unnormalised weights over the
// 3x3 joint matrix (len of E1 x len of E2); the normalised distribution is
// computed on demand. Editing a marginal or a single cell rescales the
// weights such that the edited quantity matches the new value after
// normalisation.

use std::collections::BTreeMap;
use std::rc::Rc;

use bgpsim::types::RouterId;
use yewdux::prelude::*;

pub mod forecast_tab;
pub mod simulate;
pub mod spec;
pub mod spec_tab;
pub mod states_tab;

pub use forecast_tab::ForecastTab;
pub use spec_tab::SpecTab;
#[allow(unused_imports)]
use simulate::RoutingStateOutcome;

use crate::net::Net;

/// The two views of the distribution. Toggling between them resets the
/// distribution to uniform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DistView {
    #[default]
    Marginals,
    Joint,
}

/// Labels of the three routing inputs, indexed like the matrix dimensions:
/// no route, AS-path length 1, AS-path length 3.
pub const INPUT_LABELS: [&str; 3] = ["no route", "len 1", "len 3"];

impl DistView {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Marginals => "Marginals",
            Self::Joint => "Joint",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Store)]
pub struct Forecast {
    weights: [[f64; 3]; 3],
    view: DistView,
    outcomes: Rc<Vec<RoutingStateOutcome>>,
    /// The routing-input combination currently applied to the visible
    /// network (set when clicking a routing state in the Routing States tab).
    selected: Option<(usize, usize)>,
    /// The configured link bandwidths (MB/s), keyed by the undirected link
    /// (min, max router id). Missing links use the default.
    mlu_bandwidth: Rc<BTreeMap<(RouterId, RouterId), f64>>,
    /// The configured traffic volumes (MB/s) from each internal router
    /// towards the destination. Missing routers use the default.
    mlu_traffic: Rc<BTreeMap<RouterId, f64>>,
    /// The MLU distribution computed by [Forecast::recompute_mlu], if the
    /// user pressed the recompute button. While `None`, the MLU plot shows
    /// the distribution for the default parameters.
    mlu_values: Option<Rc<Vec<(f64, f64)>>>,
}

impl Default for Forecast {
    fn default() -> Self {
        Self {
            weights: [[1.0; 3]; 3],
            view: DistView::default(),
            outcomes: Rc::new(Vec::new()),
            selected: None,
            mlu_bandwidth: Rc::new(BTreeMap::new()),
            mlu_traffic: Rc::new(BTreeMap::new()),
            mlu_values: None,
        }
    }
}

/// Lower bound on the weights so that every outcome keeps a non-zero
/// probability and scaling never divides by zero.
const MIN_WEIGHT: f64 = 1e-6;

impl Forecast {
    pub fn view(&self) -> DistView {
        self.view
    }

    /// Switch the view. Toggling resets the distribution to uniform.
    pub fn set_view(&mut self, view: DistView) {
        if view != self.view {
            self.view = view;
            self.reset();
        }
    }

    /// Reset the distribution to uniform.
    pub fn reset(&mut self) {
        self.weights = [[1.0; 3]; 3];
    }

    /// Set the distribution to a random one: a number between 0 and 100 is
    /// drawn for every cell (normalized on demand).
    pub fn randomize(&mut self) {
        let mut weights = [[0.0; 3]; 3];
        for row in weights.iter_mut() {
            for w in row.iter_mut() {
                *w = (js_sys::Math::random() * 100.0).max(MIN_WEIGHT);
            }
        }
        self.weights = weights;
    }

    /// The distribution over routing states. Computed by [Forecast::run_forecast].
    #[allow(dead_code)]
    pub fn outcomes(&self) -> &[RoutingStateOutcome] {
        &self.outcomes
    }

    /// The routing-input combination applied to the visible network, if any.
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selected
    }

    /// Select a routing state (by one of its routing-input combinations), or
    /// deselect by passing `None`.
    pub fn set_selected(&mut self, selected: Option<(usize, usize)>) {
        self.selected = selected;
    }

    /// Compute the routing-state distribution from the *current* network and
    /// the forecast matrix. Triggered by the "Run forecast" button. Also
    /// opens the bottom panel on the Routing States tab.
    pub fn run_forecast(&mut self) {
        let net = Dispatch::<Net>::new().get();
        let matrix = self.matrix();
        self.outcomes = simulate::compute_outcomes(&net, matrix);
        self.selected = None;
        // the MLU distribution depends on the outcomes: recompute it with
        // the configured parameters (or show the defaults until the user
        // pressed recompute at least once)
        if self.mlu_values.is_some() {
            self.recompute_mlu();
        }
        Dispatch::<crate::state::State>::new().reduce_mut(|s| {
            s.set_panel_tab(crate::state::PanelTab::States);
        });
    }

    /// Set the bandwidth (MB/s) of the undirected link (a, b).
    pub fn set_mlu_bandwidth(&mut self, a: RouterId, b: RouterId, value: f64) {
        let key = if a < b { (a, b) } else { (b, a) };
        let mut m = (*self.mlu_bandwidth).clone();
        m.insert(key, value);
        self.mlu_bandwidth = Rc::new(m);
    }

    /// Set the traffic volume (MB/s) from the given internal router towards
    /// the destination.
    pub fn set_mlu_traffic(&mut self, router: RouterId, value: f64) {
        let mut m = (*self.mlu_traffic).clone();
        m.insert(router, value);
        self.mlu_traffic = Rc::new(m);
    }

    pub fn mlu_bandwidth(&self) -> &BTreeMap<(RouterId, RouterId), f64> {
        &self.mlu_bandwidth
    }

    pub fn mlu_traffic(&self) -> &BTreeMap<RouterId, f64> {
        &self.mlu_traffic
    }

    /// The MLU distribution computed by [Forecast::recompute_mlu], if any.
    pub fn mlu_values(&self) -> Option<&Rc<Vec<(f64, f64)>>> {
        self.mlu_values.as_ref()
    }

    /// Recompute the normalized MLU distribution from the current outcomes
    /// and the configured bandwidths and traffic volumes. Triggered by the
    /// "Recompute" button in the Specification tab.
    pub fn recompute_mlu(&mut self) {
        let mut values: Vec<(f64, f64)> = Vec::new();
        let mut total = 0.0;
        for o in self.outcomes.iter() {
            let mlu = {
                let raw = spec::mlu_with(o, &self.mlu_bandwidth, &self.mlu_traffic);
                (raw * 1000.0).round() / 1000.0
            };
            let p = o.probability;
            total += p;
            match values.iter_mut().find(|(v, _)| (*v - mlu).abs() < 1e-6) {
                Some((_, acc)) => *acc += p,
                None => values.push((mlu, p)),
            }
        }
        if total > 0.0 {
            for (_, p) in values.iter_mut() {
                *p /= total;
            }
        }
        values.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        self.mlu_values = Some(Rc::new(values));
    }

    /// The normalised joint distribution P(len1 = i, len2 = j).
    pub fn matrix(&self) -> [[f64; 3]; 3] {
        let total: f64 = self.weights.iter().flatten().copied().sum();
        if total <= 0.0 {
            return [[1.0 / 9.0; 3]; 3];
        }
        let mut m = [[0.0; 3]; 3];
        for (i, row) in self.weights.iter().enumerate() {
            for (j, w) in row.iter().enumerate() {
                m[i][j] = w / total;
            }
        }
        m
    }

    /// Marginal P(len1 = i) of external router 1 (rows of the matrix).
    pub fn marginal1(&self) -> [f64; 3] {
        let m = self.matrix();
        let mut p = [0.0; 3];
        for (i, row) in m.iter().enumerate() {
            p[i] = row.iter().sum();
        }
        p
    }

    /// Marginal P(len2 = j) of external router 2 (columns of the matrix).
    pub fn marginal2(&self) -> [f64; 3] {
        let m = self.matrix();
        let mut p = [0.0; 3];
        for j in 0..3 {
            p[j] = m[0][j] + m[1][j] + m[2][j];
        }
        p
    }

    /// Set P(len1) of external router 1 to `target` (rescales row `len`).
    pub fn set_marginal1(&mut self, len: usize, target: f64) {
        self.scale(|i, _| i == len, target);
    }

    /// Set P(len2) of external router 2 to `target` (rescales column `len`).
    pub fn set_marginal2(&mut self, len: usize, target: f64) {
        self.scale(|_, j| j == len, target);
    }

    /// Set P(len1 = i, len2 = j) to `target` (rescales the single cell).
    pub fn set_cell(&mut self, i: usize, j: usize, target: f64) {
        self.scale(move |a, b| a == i && b == j, target);
    }

    /// Scale the weights of the selected cells such that, after
    /// normalisation, their total probability equals `target`. The weights
    /// within the selection keep their relative proportions, and so do the
    /// weights outside the selection.
    fn scale(&mut self, select: impl Fn(usize, usize) -> bool, target: f64) {
        let target = target.clamp(0.001, 0.999);
        let mut total = 0.0;
        let mut current = 0.0;
        for (i, row) in self.weights.iter().enumerate() {
            for (j, w) in row.iter().enumerate() {
                total += w;
                if select(i, j) {
                    current += w;
                }
            }
        }
        if total <= 0.0 || current <= 0.0 {
            return;
        }
        let s = target * (total - current) / (current * (1.0 - target));
        for (i, row) in self.weights.iter_mut().enumerate() {
            for (j, w) in row.iter_mut().enumerate() {
                if select(i, j) {
                    *w = (*w * s).max(MIN_WEIGHT);
                }
            }
        }
    }
}

