// Demo network for the forecasting demo.
//
// Builds the fixed demo topology used in the HotNets'26 forecasting demo:
// a triangle of three internal routers (R0, R1, R2) with full-mesh iBGP,
// connected to two external networks E1 (at R0) and E2 (at R2). Both
// external networks advertise the same demo prefix.

use std::collections::HashMap;

use bgpsim::network::Network;
use bgpsim::ospf::LocalOspf;
use bgpsim::types::{ASN, RouterId};
use yewdux::prelude::Dispatch;

use super::{Net, Pfx, Queue, Replay};
use crate::point::Point;

/// The prefix advertised by both external networks.
pub const DEMO_PREFIX: &str = "129.132.57.30/32";

/// The AS number of the internal network.
pub const DEMO_ASN: ASN = ASN(100);

/// ASN of the first external network (attached to R0).
pub const DEMO_E1_ASN: ASN = ASN(1);

/// ASN of the second external network (attached to R2).
pub const DEMO_E2_ASN: ASN = ASN(2);

/// Build the demo network and return it together with the initial router
/// positions.
pub fn build_demo_network() -> (Network<Pfx, Queue, LocalOspf>, HashMap<RouterId, Point>) {
    let mut net = Network::new(Queue::new());

    // internal routers (triangle)
    let r0 = net.add_router("R0", DEMO_ASN);
    let r1 = net.add_router("R1", DEMO_ASN);
    let r2 = net.add_router("R2", DEMO_ASN);

    // external networks
    let e1 = net.add_router("E1", DEMO_E1_ASN);
    let e2 = net.add_router("E2", DEMO_E2_ASN);

    // physical links: triangle + one external link on R0 and R2. External
    // links have no OSPF weight, so only internal links get a weight.
    let links = [(r0, r1), (r1, r2), (r2, r0), (e1, r0), (e2, r2)];
    for (a, b) in links {
        net.add_link(a, b).unwrap();
    }
    for (a, b) in [(r0, r1), (r1, r2), (r2, r0)] {
        net.set_link_weight(a, b, 1.0).unwrap();
        net.set_link_weight(b, a, 1.0).unwrap();
    }

    // iBGP full mesh among the internal routers (`false`: no route-reflector
    // client relationship; the eBGP/iBGP distinction is derived from the ASNs)
    for (a, b) in [(r0, r1), (r1, r2), (r0, r2)] {
        net.set_bgp_session(a, b, Some(false)).unwrap();
    }

    // eBGP sessions to the external networks
    net.set_bgp_session(e1, r0, Some(false)).unwrap();
    net.set_bgp_session(e2, r2, Some(false)).unwrap();

    // initial advertisements: E1 announces AS-path length 1, E2 announces
    // AS-path length 3. Remember that the own AS is prepended when the route
    // is advertised over eBGP. Hence, an empty AS-path yields a length of 1.
    let prefix: Pfx = DEMO_PREFIX.parse().unwrap();
    net.advertise_route(e1, prefix, Vec::<ASN>::new(), None, Vec::new()).unwrap();
    net.advertise_route(
        e2,
        prefix,
        vec![crate::forecast::simulate::FILLER_ASN, crate::forecast::simulate::FILLER_ASN],
        None,
        Vec::new(),
    )
    .unwrap();

    // fixed initial positions: R1 on top, R0 and R2 at the bottom, the
    // external routers further out. The positions are normalized to fit the
    // screen when loaded.
    let pos: HashMap<RouterId, Point> = [
        (r0, Point::new(300.0, 400.0)),
        (r1, Point::new(550.0, 150.0)),
        (r2, Point::new(800.0, 400.0)),
        (e1, Point::new(100.0, 580.0)),
        (e2, Point::new(1000.0, 580.0)),
    ]
    .into_iter()
    .collect();

    (net, pos)
}

/// Load the demo network into the application state.
pub fn load_demo_network() {
    let (net, pos) = build_demo_network();
    Dispatch::<Net>::new().reduce_mut(|n| {
        *n.net.borrow_mut() = net;
        n.spec.borrow_mut().clear();
        *n.replay.borrow_mut() = Replay::default();
        n.topology_zoo = None;
        n.last_asn = DEMO_ASN;
        *n.pos.borrow_mut() = pos;
        n.normalize_pos();
    });
    // reset the selection state
    Dispatch::<crate::State>::new().reduce_mut(|s| {
        s.set_selected(crate::state::Selected::None);
        s.clear_hover();
        s.set_prefix(None);
    });
}
