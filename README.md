# Interactive Forecasting Demo (HotNets'26)

**This is a demonstration artifact, not a production tool.** It is an
interactive web demo built to accompany the paper *"Forecasting routing"*
(HotNets 2026): the user draws a probability distribution over the routing
inputs of two external networks, computes the resulting distribution over
BGP routing states, and evaluates simple specifications (fully connected,
load balanced, maximum link utilization) on that distribution.

## What it is based off of

The demo is a **modified fork of
[bgpsim-web](https://github.com/nsg-ethz/bgpsim)**, the web frontend of
[BGPsim](https://github.com/nsg-ethz/bgpsim) — a BGP/OSPF network control
plane simulator written in Rust by Tibor Schneider (NSG, ETH Zürich). The
simulation engine (`bgpsim` crate, included in `bgpsim/`) is used unmodified;
the web UI (`bgpsim-web/`) was stripped down and extended for the demo. If
you use bgpsim for research, please cite the original work:

```bibtex
@INPROCEEDINGS{schneider2021snowcap,
  author    = {Schneider, Tibor and Birkner, R{\"u}diger and Vanbever, Laurent},
  booktitle = {Proceedings of the 2021 ACM SIGCOMM Conference},
  title     = {Snowcap: Synthesizing Network-Wide Configuration Updates},
  year      = {2021},
  pages     = {33--49},
  doi       = {10.1145/3452296.3472916}
}
```

## What the demo shows

- A fixed demo network: three internal routers in a triangle (full-mesh
  iBGP) connected to two external networks; both advertise the prefix
  `129.132.57.30/32`.
- **Forecast** (right sidebar, bottom half): draw the joint probability
  distribution over the routing inputs of the two external networks
  (no route, AS-path length 1, AS-path length 3) — as independent marginals
  or as a joint 3×3 matrix. Drag the bars/cells to edit.
- **Run forecast**: compute the distribution over routing states (full
  forwarding paths) from the *current* network configuration and the drawn
  distribution. States are depicted as mini versions of the network and
  form the ordered probability histogram in the **Routing States** tab
  (clicking a state highlights it on the canvas).
- **Specification** tab: evaluate simple specifications on the state
  distribution (fully connected; load balanced; maximum link utilization
  with 1 MB/s demands and 3 MB/s link capacities).
- The network configuration (link weights, BGP sessions, **route maps**,
  neighbors) remains editable in the sidebar, and feeds into the forecast.

## Running

Development (live reload):

```sh
cd bgpsim/bgpsim-web
trunk serve
```

Production build (fully static, client-side, no server needed):

```sh
cd bgpsim/bgpsim-web
trunk build --release   # output in dist/
```

The `dist/` folder must be served over HTTP (any static file server —
browsers refuse ES modules and `fetch()` from `file://`):

```sh
cd bgpsim/bgpsim-web/dist
python3 -m http.server 8000   # http://localhost:8000
```

### Deploying to GitHub Pages

The build is deployment-safe at any base path: all asset references are
relative (`public_url = "./"` in `Trunk.toml`) and a `.nojekyll` file is
included (guards against GitHub Pages' Jekyll step). Publish the *contents*
of `dist/` (e.g. to a `gh-pages` branch, or via a workflow using
`actions/upload-pages-artifact` with `path: bgpsim/bgpsim-web/dist`) and it
works both at `user.github.io` and at a project site
(`user.github.io/repo-name/`).

A GitHub Actions workflow is included (`.github/workflows/deploy.yml`): it
builds the demo with the pinned toolchain on every push to `main` and
deploys it to GitHub Pages. To enable it: in the repository settings, set
*Pages → Source* to **GitHub Actions**. The workflow can also be triggered
manually (*workflow_dispatch*).

The development repository is hosted on GitLab (ETH). To publish via
GitHub, add GitHub as a second remote and use the `publish-github.sh`
helper, which squashes the current state of `main` into a single commit on
the `github-publish` branch and force-pushes it to GitHub's `main` (the
workflow then builds and deploys):

```sh
git remote add github git@github.com:<user>/<repo>.git   # once
./publish-github.sh                                      # whenever you want to update
```

## License

This demo is distributed under the **GNU General Public License v3.0 or
later** (see `LICENSE`). The reason for this choice:

- The demo is built on top of `bgpsim-web`, whose source files carry
  **GPL-2.0-or-later** headers (© Tibor Schneider, NSG ETH Zürich).
- The `bgpsim` simulator crate itself is **Apache-2.0** (© Tibor Schneider,
  NSG ETH Zürich), which is compatible with GPL-3.0 but *not* with
  GPL-2.0-only — hence GPL-3.0-or-later is the license that satisfies both
  parts of the combined work.
- The Apache-2.0 license terms and copyright notices of the bgpsim crates
  are retained in the repository (`bgpsim/bgpsim/LICENSE` and the source
  file headers), as required by Apache-2.0.
- All modifications made for this demo are © 2026 Jean Mégret (NSG, ETH
  Zürich), licensed under GPL-3.0-or-later.

Requirements: Rust with the toolchain pinned in
`bgpsim/bgpsim-web/rust-toolchain.toml` (`nightly-2024-12-01`; newer rustc
versions reject the `anymap` dependency of yewdux with E0804, which is why
the toolchain is pinned to the project era), the
`wasm32-unknown-unknown` target, `trunk`, and `tailwindcss` (v3) on the PATH
for the pre-build hook. The MSRV-aware resolver
(`.cargo/config.toml`) resolves dependency versions compatible with the
pinned toolchain; `Cargo.lock` is committed for reproducibility.

## Repository layout

```
bgpsim/                the bgpsim repository (flattened fork)
  bgpsim/              the simulator crate (unmodified)
  bgpsim-macros/       the net! macro crate (unmodified)
  bgpsim-web/          the web frontend (modified for the demo)
    src/forecast/      demo code: forecast store, simulation engine,
                       routing states and specification tabs
    src/net/demo.rs    the demo network bootstrap
    src/panel.rs       the collapsible bottom panel
figures/               Paul's faces for the tour
publish-github.sh      squash + publish helper (see above)
```Also generated locally (gitignored): `target/`, `dist/`, `tailwind.css`.
