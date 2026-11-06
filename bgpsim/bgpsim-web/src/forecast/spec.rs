// Specifications: map each routing state to the outcome of a specification,
// then aggregate the probabilities of all states leading to the same outcome.

use bgpsim::types::RouterId;

use super::simulate::RoutingStateOutcome;

/// The available specifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Spec {
    #[default]
    FullyConnected,
    LoadBalanced,
    Mlu,
}

impl Spec {
    pub fn label(&self) -> &'static str {
        match self {
            Self::FullyConnected => "Fully connected",
            Self::LoadBalanced => "Load balanced",
            Self::Mlu => "MLU",
        }
    }

    /// A short description of the specification, shown next to the plot.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FullyConnected => {
                "True if and only if every internal router has a forwarding \
                 path towards the destination prefix."
            }
            Self::LoadBalanced => {
                "True if and only if R0 and R2 forward towards different \
                 egress routers of the network."
            }
            Self::Mlu => {
                "Each internal router sends 1 MB/s towards the destination \
                 along its forwarding path; every link of the network \
                 (including the egress links) has a capacity of 3 MB/s. The \
                 outcome is the utilization of the most loaded link, \
                 normalized to [0, 1]."
            }
        }
    }
}

/// Fully connected: true iff every internal router has a forwarding path
/// towards the destination.
pub fn fully_connected(outcome: &RoutingStateOutcome) -> bool {
    outcome.state.paths.values().all(|paths| !paths.is_empty())
}

/// The egress router (last element of the first forwarding path) of the
/// given router, if it has one.
pub fn egress_of(outcome: &RoutingStateOutcome, router: &RouterId) -> Option<RouterId> {
    outcome
        .state
        .paths
        .get(router)
        .and_then(|paths| paths.first())
        .and_then(|path| path.last())
        .copied()
}

/// Load balanced: true iff R0 and R2 forward towards different egresses.
/// False if either of them has no route.
pub fn load_balanced(outcome: &RoutingStateOutcome, r0: &RouterId, r2: &RouterId) -> bool {
    match (egress_of(outcome, r0), egress_of(outcome, r2)) {
        (Some(e0), Some(e2)) => e0 != e2,
        _ => false,
    }
}

/// The default link bandwidth (MB/s), used when no bandwidth is configured
/// for a link.
pub const DEFAULT_BANDWIDTH: f64 = 3.0;

/// The default traffic volume (MB/s) from each internal router towards the
/// destination.
pub const DEFAULT_TRAFFIC: f64 = 1.0;

/// The maximum link utilization for the given routing state: each internal
/// router `r` sends `traffic[r]` (default: 1 MB/s) towards the destination
/// along its first forwarding path (no ECMP splitting); every link carries
/// the traffic of all paths over it, and has the configured bandwidth
/// (default: 3 MB/s). Returns the utilization of the most loaded link, in
/// [0, 1] (values above 1 are possible if the traffic exceeds the
/// capacity). States without any route have an MLU of 0.
pub fn mlu_with(
    outcome: &RoutingStateOutcome,
    bandwidth: &std::collections::BTreeMap<(RouterId, RouterId), f64>,
    traffic: &std::collections::BTreeMap<RouterId, f64>,
) -> f64 {
    // accumulate the load per directed link
    let mut load: Vec<((RouterId, RouterId), f64)> = Vec::new();
    for (router, paths) in outcome.state.paths.iter() {
        let Some(path) = paths.first() else { continue };
        let volume = traffic.get(router).copied().unwrap_or(DEFAULT_TRAFFIC);
        for link in path.windows(2) {
            let key = (link[0], link[1]);
            match load.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => *v += volume,
                None => load.push((key, volume)),
            }
        }
    }
    // the utilization of a link is the load of the more loaded direction,
    // divided by the (undirected) link bandwidth
    load.iter()
        .map(|((a, b), v)| {
            let key = if a < b { (*a, *b) } else { (*b, *a) };
            let bw = bandwidth.get(&key).copied().unwrap_or(DEFAULT_BANDWIDTH);
            if bw <= 0.0 {
                0.0
            } else {
                v / bw
            }
        })
        .fold(0.0, f64::max)
}

/// The MLU with the default parameters (1 MB/s per router, 3 MB/s per link).
pub fn mlu(outcome: &RoutingStateOutcome) -> f64 {
    mlu_with(outcome, &Default::default(), &Default::default())
}
