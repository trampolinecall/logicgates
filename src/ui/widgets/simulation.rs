use std::{collections::HashMap, marker::PhantomData};

use nannou::prelude::*;

use crate::{
    simulation::{self, hierarchy, logic, Gate, GateKey, NodeKey, NodeMap, Simulation},
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, SizeConstraints, TargetedEvent, View,
    },
};

const VERTICAL_VALUE_SPACING: f32 = 20.0;
const GATE_EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
const GATE_WIDTH: f32 = 50.0;

pub(crate) struct SimulationWidgetState {
    cur_gate_drag: Option<simulation::GateKey>,
    view: Option<simulation::CircuitKey>,
}

impl SimulationWidgetState {
    pub(crate) fn new() -> SimulationWidgetState {
        SimulationWidgetState { cur_gate_drag: None, view: None }
    }
}

struct SimulationView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    gates: Vec<GateView<Data, StateLens, SimulationLens>>,
    nodes: Vec<NodeView<Data, StateLens, SimulationLens>>,
    connections: Vec<ConnectionView<Data, StateLens, SimulationLens>>,
}

struct GateView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    state_lens: StateLens,
    simulation_lens: SimulationLens,

    gate_key: GateKey,
    name: String,
    gate_location: (f32, f32),
    num_inputs: usize,
    num_outputs: usize,

    being_dragged: bool,

    _phantom: PhantomData<fn(&Data)>,
}
#[derive(Copy, Clone)]
enum NodeViewPos {
    FarLeftEdge(usize, usize, usize),
    FarRightEdge(usize, usize, usize),
    LeftOfGate((f32, f32), usize, usize, usize),
    RightOfGate((f32, f32), usize, usize, usize),
}
struct NodeView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    // state_lens: StateLens,
    // simulation_lens: SimulationLens,

    // key: NodeKey,
    pos: NodeViewPos,
    color: nannou::color::Srgb<u8>,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}
struct ConnectionView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    // state_lens: StateLens,
    // simulation_lens: SimulationLens,

    // node1: NodeKey,
    // node2: NodeKey,
    pos1: NodeViewPos,
    pos2: NodeViewPos,
    color: nannou::color::Srgb<u8>,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}

pub(crate) fn simulation<Data>(
    id_maker: &mut ViewIdMaker,
    state_lens: impl Lens<Data, SimulationWidgetState> + Copy,
    simulation_lens: impl Lens<Data, Simulation> + Copy,
    data: &Data,
) -> impl View<Data> {
    // TODO: show currently viewing at top of widget
    let (current_view, cur_gate_drag) = state_lens.with(data, |state| (state.view, state.cur_gate_drag));
    let (gates, nodes, connections) = simulation_lens.with(data, |simulation| {
        let gates_currently_viewing = match current_view {
            Some(ck) => &simulation.circuits[ck].gates,
            None => &simulation.toplevel_gates,
        };

        let gates = gates_currently_viewing.iter().copied();
        let nodes = gates_currently_viewing
            .iter()
            .flat_map(|gate| simulation::Gate::inputs(&simulation.circuits, &simulation.gates, *gate).iter().chain(simulation::Gate::outputs(&simulation.circuits, &simulation.gates, *gate)))
            .copied();

        let gate_views = gates
            .into_iter()
            .map(|gate| {
                let gate_location = Gate::location(&simulation.circuits, &simulation.gates, gate);
                let num_inputs = Gate::num_inputs(&simulation.circuits, &simulation.gates, gate);
                let num_outputs = Gate::num_outputs(&simulation.circuits, &simulation.gates, gate);
                let gate_name = simulation.gates[gate].name(&simulation.circuits).to_string();

                GateView {
                    id: id_maker.next_id(),
                    state_lens,
                    simulation_lens,
                    gate_key: gate,
                    name: gate_name,
                    gate_location: (gate_location.x, gate_location.y),
                    num_inputs,
                    num_outputs,
                    being_dragged: Some(gate) == cur_gate_drag,
                    _phantom: PhantomData,
                }
            })
            .collect();

        let node_positions_and_colors: HashMap<_, _> = nodes
            .into_iter()
            .map(|node| {
                let pos = match simulation.nodes[node].parent.kind() {
                    hierarchy::NodeParentKind::CircuitIn(c, i) if Some(c) == current_view => {
                        let circuit = &simulation.circuits[c];
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        NodeViewPos::FarLeftEdge(i, num_inputs, num_outputs)
                    }
                    hierarchy::NodeParentKind::CircuitOut(c, i) if Some(c) == current_view => {
                        let circuit = &simulation.circuits[c];
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        NodeViewPos::FarRightEdge(i, num_inputs, num_outputs)
                    }
                    hierarchy::NodeParentKind::CircuitIn(c, i) => {
                        let circuit = &simulation.circuits[c];
                        let location = &circuit.location;
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        NodeViewPos::LeftOfGate((location.x, location.y), i, num_inputs, num_outputs)
                    }
                    hierarchy::NodeParentKind::CircuitOut(c, i) => {
                        let circuit = &simulation.circuits[c];
                        let location = &circuit.location;
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        NodeViewPos::RightOfGate((location.x, location.y), i, num_inputs, num_outputs)
                    }
                    hierarchy::NodeParentKind::GateIn(g, i) => {
                        let location = &simulation::Gate::location(&simulation.circuits, &simulation.gates, g);
                        let num_inputs = simulation::Gate::num_inputs(&simulation.circuits, &simulation.gates, g);
                        let num_outputs = simulation::Gate::num_outputs(&simulation.circuits, &simulation.gates, g);
                        NodeViewPos::LeftOfGate((location.x, location.y), i, num_inputs, num_outputs)
                    }
                    hierarchy::NodeParentKind::GateOut(g, i) => {
                        let location = &simulation::Gate::location(&simulation.circuits, &simulation.gates, g);
                        let num_inputs = simulation::Gate::num_inputs(&simulation.circuits, &simulation.gates, g);
                        let num_outputs = simulation::Gate::num_outputs(&simulation.circuits, &simulation.gates, g);
                        NodeViewPos::RightOfGate((location.x, location.y), i, num_inputs, num_outputs)
                    }
                };
                let color = node_color(&simulation.nodes, node, true);

                (node, (pos, color))
            })
            .collect();
        let connection_vews: Vec<_> = simulation
            .connections
            .iter()
            .filter_map(|(a, b)| {
                Some(ConnectionView {
                    id: id_maker.next_id(),
                    pos1: node_positions_and_colors.get(a)?.0,
                    pos2: node_positions_and_colors.get(b)?.0,
                    color: node_positions_and_colors.get(a)?.1,
                    _phantom: PhantomData,
                    _phantom2: PhantomData,
                    _phantom3: PhantomData,
                    // state_lens,
                    // simulation_lens,
                    // node1: *a,
                    // node2: *b,
                })
            })
            .collect();
        let node_views = node_positions_and_colors
            .into_iter()
            .map(|(_, (pos, color))| NodeView {
                id: id_maker.next_id(),
                pos,
                color,
                _phantom: PhantomData,
                _phantom2: PhantomData,
                _phantom3: PhantomData, /* state_lens, simulation_lens, key: node */
            })
            .collect();

        (gate_views, node_views, connection_vews)
    });

    SimulationView { id: id_maker.next_id(), gates, nodes, connections }
}

impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> View<Data> for SimulationView<Data, StateLens, SimulationLens> {
    fn draw(&self, app: &nannou::App, draw: &nannou::Draw, center: Vec2, sc: SizeConstraints, hover: Option<ViewId>) {
        let widget_rect = widget_rect(center, sc);
        draw.rect().xy(widget_rect.xy()).wh(widget_rect.wh()).color(Theme::DEFAULT.simulation_bg_color);

        for connection in &self.connections {
            connection.draw(app, draw, center, sc, hover);
        }
        for gate in &self.gates {
            gate.draw(app, draw, center, sc, hover);
        }
        for node in &self.nodes {
            node.draw(app, draw, center, sc, hover);
        }
    }

    fn find_hover(&self, center: Vec2, sc: SizeConstraints, mouse: Vec2) -> Option<ViewId> {
        // reverse to go in z order from highest to lowest
        for node in self.nodes.iter().rev() {
            if let hover @ Some(_) = node.find_hover(center, sc, mouse) {
                return hover;
            }
        }
        for gate in self.gates.iter().rev() {
            if let hover @ Some(_) = gate.find_hover(center, sc, mouse) {
                return hover;
            }
        }
        for connection in self.connections.iter().rev() {
            if let hover @ Some(_) = connection.find_hover(center, sc, mouse) {
                return hover;
            }
        }
        if widget_rect(center, sc).contains(mouse) {
            return Some(self.id);
        }

        None
    }

    fn size(&self, sc: SizeConstraints) -> Vec2 {
        sc.max
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
        for node in &self.nodes {
            node.send_targeted_event(app, data, target, event);
        }
        for gate in &self.gates {
            gate.send_targeted_event(app, data, target, event);
        }
        for connection in &self.connections {
            connection.send_targeted_event(app, data, target, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &nannou::App, data: &mut Data, event: GeneralEvent) {
        for node in &self.nodes {
            node.general_event(app, data, event);
        }
        for gate in &self.gates {
            gate.general_event(app, data, event);
        }
        for connection in &self.connections {
            connection.general_event(app, data, event);
        }
    }
}

fn widget_rect(center: Vec2, sc: SizeConstraints) -> nannou::geom::Rect {
    nannou::geom::Rect::from_xy_wh(center, sc.max)
}

impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> View<Data> for GateView<Data, StateLens, SimulationLens> {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, widget_center: Vec2, widget_sc: SizeConstraints, hover: Option<ViewId>) {
        // TODO: cache?
        let widget_rect = widget_rect(widget_center, widget_sc);
        let rect = gate_rect(widget_rect, self.gate_location, self.num_inputs, self.num_outputs);

        if Some(self.id) == hover {
            let hover_rect =
                rect.pad_left(-Theme::DEFAULT.gate_hover_dist).pad_top(-Theme::DEFAULT.gate_hover_dist).pad_right(-Theme::DEFAULT.gate_hover_dist).pad_bottom(-Theme::DEFAULT.gate_hover_dist); // expand by hover distance, this is the "stroke weight"
            draw.rect().xy(hover_rect.xy()).wh(hover_rect.wh()).color(Theme::DEFAULT.gate_hover_color);
        }

        draw.rect().xy(rect.xy()).wh(rect.wh()).color(Theme::DEFAULT.gate_color);

        draw.text(&self.name).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y().color(Theme::DEFAULT.gate_text_color);
    }

    fn find_hover(&self, widget_center: Vec2, widget_sc: SizeConstraints, mouse_pos: Vec2) -> Option<ViewId> {
        // TODO: also cache?
        let widget_rect = widget_rect(widget_center, widget_sc);
        let rect = gate_rect(widget_rect, self.gate_location, self.num_inputs, self.num_outputs);
        if rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self.id);
        }

        None
    }

    fn size(&self, _: SizeConstraints) -> Vec2 {
        Vec2::ZERO // does not participate in layout
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if self.id == target {
            self.targeted_event(app, data, event);
        }
        // no other chilren to go through
    }

    fn targeted_event(&self, _: &nannou::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown => self.state_lens.with_mut(data, |state| state.cur_gate_drag = Some(self.gate_key)),
        }
    }

    fn general_event(&self, _: &nannou::App, data: &mut Data, event: GeneralEvent) {
        if self.being_dragged {
            match event {
                GeneralEvent::MouseMoved(mouse_pos) => {
                    // TODO: zooming and panning, also fix dragging when simulation widget is not at center of screen
                    self.simulation_lens.with_mut(data, |simulation| {
                        let loc = simulation::Gate::location_mut(&mut simulation.circuits, &mut simulation.gates, self.gate_key);
                        loc.x = mouse_pos.x;
                        loc.y = mouse_pos.y;
                    });
                }
                GeneralEvent::LeftMouseUp => self.state_lens.with_mut(data, |state| {
                    state.cur_gate_drag = None;
                }),
            }
        } else {
            // dont care
        }
    }
}
impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, simulation::Simulation>> View<Data> for NodeView<Data, StateLens, SimulationLens> {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, widget_center: Vec2, widget_sc: SizeConstraints, hover: Option<ViewId>) {
        let widget_rect = widget_rect(widget_center, widget_sc);
        let pos = node_pos(widget_rect, self.pos);
        if Some(self.id) == hover {
            draw.ellipse().xy(pos).radius(Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist).color(Theme::DEFAULT.node_hover_color);
        }

        draw.ellipse().xy(pos).radius(Theme::DEFAULT.node_rad).color(self.color);
    }

    fn find_hover(&self, widget_center: Vec2, widget_sc: SizeConstraints, mouse_pos: Vec2) -> Option<ViewId> {
        let widget_rect = widget_rect(widget_center, widget_sc);
        let pos = node_pos(widget_rect, self.pos);
        if pos.distance(mouse_pos) < Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist {
            return Some(self.id);
        }
        None
    }

    fn size(&self, _: SizeConstraints) -> Vec2 {
        Vec2::ZERO
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &nannou::App, _: &mut Data, _: GeneralEvent) {}
}
impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> View<Data> for ConnectionView<Data, StateLens, SimulationLens> {
    fn draw(&self, _: &nannou::App, draw: &nannou::Draw, widget_center: Vec2, widget_sc: SizeConstraints, hover: Option<ViewId>) {
        let widget_rect = widget_rect(widget_center, widget_sc);
        let pos1 = node_pos(widget_rect, self.pos1);
        let pos2 = node_pos(widget_rect, self.pos2);
        let mut line = draw.line().start(pos1).end(pos2).weight(Theme::DEFAULT.connection_width).color(self.color);

        if Some(self.id) == hover {
            line = line.weight(Theme::DEFAULT.connection_width + Theme::DEFAULT.connection_hover_dist);
        }

        line.finish();
    }

    fn find_hover(&self, widget_center: Vec2, widget_sc: SizeConstraints, mouse_pos: Vec2) -> Option<ViewId> {
        let widget_rect = widget_rect(widget_center, widget_sc);
        let pos1 = node_pos(widget_rect, self.pos1);
        let pos2 = node_pos(widget_rect, self.pos2);
        if min_dist_to_line_squared((pos1, pos2), mouse_pos) < Theme::DEFAULT.connection_hover_dist.powf(2.0) {
            Some(self.id)
        } else {
            None
        }
    }

    fn size(&self, _: SizeConstraints) -> Vec2 {
        Vec2::ZERO // does not participate in layout
    }

    fn send_targeted_event(&self, app: &nannou::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &nannou::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &nannou::App, _: &mut Data, _: GeneralEvent) {}
}

// TODO: reorganize all of these functions
fn gate_rect(widget_rect: Rect, (x, y): (f32, f32), num_inputs: usize, num_outputs: usize) -> Rect {
    let wh = gate_display_size(num_inputs, num_outputs);
    Rect::from_x_y_w_h(widget_rect.x() + x, widget_rect.y() + y, wh.x, wh.y)
}

fn gate_display_size(num_inputs: usize, num_outputs: usize) -> Vec2 {
    let gate_height = (std::cmp::max(num_inputs, num_outputs) - 1) as f32 * VERTICAL_VALUE_SPACING + GATE_EXTRA_VERTICAL_HEIGHT;
    pt2(GATE_WIDTH, gate_height)
}

fn y_centered_around(center_y: f32, total: usize, index: usize) -> f32 {
    let box_height: f32 = ((total - 1) as f32) * VERTICAL_VALUE_SPACING;
    let box_start_y = center_y + (box_height / 2.0);
    box_start_y - (index as f32) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(widget_rect: Rect, num_inputs: usize, num_outputs: usize, index: usize) -> Vec2 {
    pt2(widget_rect.x.start, y_centered_around(0.0, num_inputs, index))
}
fn circuit_output_pos(widget_rect: Rect, num_inputs: usize, num_outputs: usize, index: usize) -> Vec2 {
    pt2(widget_rect.x.end, y_centered_around(0.0, num_outputs, index))
}

fn gate_input_pos(widget_rect: Rect, gate_location: (f32, f32), num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(widget_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.left(), y_centered_around(rect.y(), num_inputs, idx))
}
fn gate_output_pos(widget_rect: Rect, gate_location: (f32, f32), num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(widget_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.right(), y_centered_around(rect.y(), num_outputs, idx))
}

fn node_pos(widget_rect: nannou::geom::Rect, pos: NodeViewPos) -> Vec2 {
    match pos {
        NodeViewPos::FarLeftEdge(i, inputs, outputs) => circuit_input_pos(widget_rect, inputs, outputs, i),
        NodeViewPos::FarRightEdge(i, inputs, outputs) => circuit_output_pos(widget_rect, inputs, outputs, i),
        NodeViewPos::LeftOfGate(gate_pos, i, inputs, outputs) => gate_input_pos(widget_rect, gate_pos, inputs, outputs, i),
        NodeViewPos::RightOfGate(gate_pos, i, inputs, outputs) => gate_output_pos(widget_rect, gate_pos, inputs, outputs, i),
    }
}

fn min_dist_to_line_squared(line_segment: (Vec2, Vec2), point: Vec2) -> f32 {
    let (a, b) = line_segment;

    let len_squared = a.distance_squared(b);
    if len_squared == 0.0 {
        point.distance_squared(a)
    } else {
        // project point onto line segment and return distance to that projected point
        let t = (point - a).dot(b - a) / len_squared;
        let t_clamped = t.clamp(0.0, 1.0);
        let projected = a.lerp(b, t_clamped);
        point.distance_squared(projected)
    }
}

fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb<u8> {
    fn value_to_color(v: logic::Value) -> Rgb<u8> {
        match v {
            logic::Value::H => Theme::DEFAULT.on_color,
            logic::Value::L => Theme::DEFAULT.off_color,
            logic::Value::Z => Theme::DEFAULT.high_impedance_color,
            logic::Value::X => Theme::DEFAULT.err_color,
        }
    }
    if use_production {
        if let Some(v) = logic::get_node_production(nodes, node) {
            value_to_color(v)
        } else {
            value_to_color(logic::get_node_value(nodes, node))
        }
    } else {
        value_to_color(logic::get_node_value(nodes, node))
    }
}
