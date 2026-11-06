// Simulation engine: for every combination of AS-path lengths (i, j) announced
// by the two external networks, compute the resulting routing state (the full
// forwarding paths of all internal routers towards the demo prefix), and
// aggregate the states into the distribution over outcomes.

use std::collections::BTreeMap;
use std::rc::Rc;

use bgpsim::network::Network;
use bgpsim::ospf::LocalOspf;
use bgpsim::types::{ASN, RouterId};

use crate::net::demo::{DEMO_ASN, DEMO_PREFIX};
use crate::net::{Net, Pfx, Queue};

/// ASN used as a filler for paths longer than 1 (the own AS of the external
/// network is prepended automatically by bgpsim).
pub const FILLER_ASN: ASN = ASN(65000);

/// The routing input of one external network, indexed 0 to 2 in the forecast
/// matrix: no advertisement, AS-path of length 1, or AS-path of length 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    NoRoute,
    Len1,
    Len3,
}

impl Input {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::NoRoute,
            1 => Self::Len1,
            _ => Self::Len3,
        }
    }

    /// The AS-path announced to the internal network (without the own AS,
    /// which bgpsim prepends over eBGP). Empty for [Input::Len1].
    pub fn as_path(&self) -> Vec<ASN> {
        match self {
            Self::NoRoute => vec![],
            Self::Len1 => vec![],
            Self::Len3 => vec![FILLER_ASN, FILLER_ASN],
        }
    }
}

/// The routing state of the network for one combination of AS-path lengths:
/// the forwarding paths (towards the demo prefix) of every internal router.
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingState {
    pub paths: BTreeMap<RouterId, Vec<Vec<RouterId>>>,
    /// Canonical string identifying the state, used for deduplication.
    pub key: String,
}

/// One distinct routing state, together with the combinations of AS-path
/// lengths that lead to it and their total probability.
#[derive(Clone, Debug, PartialEq)]
pub struct RoutingStateOutcome {
    pub state: RoutingState,
    /// The (len1, len2) combinations that lead to this state.
    pub combinations: Vec<(usize, usize)>,
    /// Total probability of all combinations leading to this state.
    pub probability: f64,
}

/// Compute the distribution over routing states for the given joint matrix
/// `matrix[i][j] = P(input1 = i, input2 = j)`, where the inputs are indexed
/// as [Input]: 0 = no route, 1 = AS-path length 1, 2 = AS-path length 3.
/// States are ordered by the first combination (in (i, j) order) that leads
/// to them.
pub fn compute_outcomes(net: &Net, matrix: [[f64; 3]; 3]) -> Rc<Vec<RoutingStateOutcome>> {
    let mut outcomes: Vec<RoutingStateOutcome> = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            let state = match compute_state(net, i, j) {
                Ok(state) => state,
                Err(e) => {
                    log::error!("Failed to compute the routing state for ({i}, {j}): {e}");
                    continue;
                }
            };
            let p = matrix[i][j];
            match outcomes.iter_mut().find(|o| o.state.key == state.key) {
                Some(o) => {
                    o.combinations.push((i, j));
                    o.probability += p;
                }
                None => outcomes.push(RoutingStateOutcome {
                    state,
                    combinations: vec![(i, j)],
                    probability: p,
                }),
            }
        }
    }
    log::info!(
        "Computed {} routing state(s): {}",
        outcomes.len(),
        outcomes
            .iter()
            .map(|o| format!(
                "{:?} -> {:.3}",
                o.combinations.iter().copied().collect::<Vec<_>>(),
                o.probability
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Rc::new(outcomes)
}

/// Compute the forwarding paths of all internal routers for the given inputs
/// of the two external networks. The base network is not modified: it is
/// cloned via serialization.
fn compute_state(net: &Net, input1: usize, input2: usize) -> Result<RoutingState, String> {
    let (e1, e2) = external_routers(net);
    let in1 = Input::from_index(input1);
    let in2 = Input::from_index(input2);

    // clone the network via serialization (Network does not implement Clone)
    let bytes = serde_json::to_vec(&*net.net()).map_err(|e| format!("serialize: {e}"))?;
    let mut net: Network<Pfx, Queue, LocalOspf> =
        serde_json::from_slice(&bytes).map_err(|e| format!("deserialize: {e}"))?;

    // apply the routing inputs: withdraw or advertise with the given AS-path
    // (bgpsim prepends the own AS over eBGP, so an empty AS-path yields an
    // AS-path length of 1)
    let prefix: Pfx = DEMO_PREFIX.parse().unwrap();
    apply_input(&mut net, e1, in1, prefix).map_err(|e| format!("apply input on E1: {e}"))?;
    apply_input(&mut net, e2, in2, prefix).map_err(|e| format!("apply input on E2: {e}"))?;

    // extract the forwarding paths of all internal routers; a router without
    // a route for the prefix has no forwarding paths (NoPath error)
    let mut fw = net.get_forwarding_state();
    let mut paths = BTreeMap::new();
    let mut key = String::new();
    for r in net.indices() {
        let is_internal = net.get_router(r).map(|x| x.asn()) == Ok(DEMO_ASN);
        if !is_internal {
            continue;
        }
        let router_paths = fw.get_paths(r, prefix).unwrap_or_default();
        key.push_str(&format!("{:?};", router_paths));
        paths.insert(r, router_paths);
    }
    Ok(RoutingState { paths, key })
}

/// Withdraw the route (for [Input::NoRoute]) or advertise the given AS-path.
pub(crate) fn apply_input(
    net: &mut Network<Pfx, Queue, LocalOspf>,
    router: RouterId,
    input: Input,
    prefix: Pfx,
) -> Result<(), bgpsim::types::NetworkError> {
    match input {
        Input::NoRoute => net.withdraw_route(router, prefix),
        _ => net.advertise_route(router, prefix, input.as_path(), None, Vec::new()),
    }
}

/// Find the two external routers. Prefer the routers named E1 / E2; fall back
/// to the two routers outside the demo AS, ordered by their index.
pub(crate) fn external_routers(net: &Net) -> (RouterId, RouterId) {
    if let (Ok(e1), Ok(e2)) = (net.net().get_router_id("E1"), net.net().get_router_id("E2")) {
        return (e1, e2);
    }
    let mut external: Vec<RouterId> = net
        .net()
        .indices()
        .filter(|&r| net.net().get_router(r).map(|x| x.asn()) != Ok(DEMO_ASN))
        .collect();
    external.sort();
    match (external.first(), external.get(1)) {
        (Some(&e1), Some(&e2)) => (e1, e2),
        _ => {
            log::error!("The demo network does not have two external routers!");
            (RouterId::from(0), RouterId::from(0))
        }
    }
}
