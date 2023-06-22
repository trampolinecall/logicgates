use std::{collections::HashMap, marker::PhantomData};

use sfml::graphics::{Shape, Transformable};

use crate::{
    simulation::{self, hierarchy, logic, Gate, GateKey, NodeKey, NodeMap, Simulation},
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
    },
};

const VERTICAL_VALUE_SPACING: f32 = 20.0;
const GATE_EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
const GATE_WIDTH: f32 = 50.0;

pub(crate) struct SimulationWidgetState {
    cur_gate_drag: Option<(simulation::GateKey, sfml::system::Vector2f, (f32, f32))>,
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
struct SimulationViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original SimulationView<Data, StateLens, SimulationLens>,
    widget_size: sfml::system::Vector2f,

    gates: Vec<GateViewLayout<'original, Data, StateLens, SimulationLens>>,
    nodes: Vec<NodeViewLayout<'original, Data, StateLens, SimulationLens>>,
    connections: Vec<ConnectionViewLayout<'original, Data, StateLens, SimulationLens>>,
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
struct GateViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original GateView<Data, StateLens, SimulationLens>,
    widget_size: sfml::system::Vector2f,
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
    color: sfml::graphics::Color,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}
struct NodeViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original NodeView<Data, StateLens, SimulationLens>,
    widget_size: sfml::system::Vector2f,
}
struct ConnectionView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    // state_lens: StateLens,
    // simulation_lens: SimulationLens,

    // node1: NodeKey,
    // node2: NodeKey,
    pos1: NodeViewPos,
    pos2: NodeViewPos,
    color: sfml::graphics::Color,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}
struct ConnectionViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original ConnectionView<Data, StateLens, SimulationLens>,
    widget_size: sfml::system::Vector2f,
}

pub(crate) fn simulation<Data>(
    id_maker: &mut ViewIdMaker,
    state_lens: impl Lens<Data, SimulationWidgetState> + Copy,
    simulation_lens: impl Lens<Data, Simulation> + Copy,
    data: &Data,
) -> impl ViewWithoutLayout<Data> {
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
                    being_dragged: if let Some((cur_gate_drag, _, _)) = cur_gate_drag { cur_gate_drag == gate } else { false },
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

impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> ViewWithoutLayout<Data> for SimulationView<Data, StateLens, SimulationLens> {
    type WithLayout<'without_layout> = SimulationViewLayout<'without_layout, Data, StateLens, SimulationLens> where Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        SimulationViewLayout {
            view: self,
            widget_size: sc.max,
            gates: self.gates.iter().map(|gate| gate.layout(sc)).collect(),
            nodes: self.nodes.iter().map(|node| node.layout(sc)).collect(),
            connections: self.connections.iter().map(|connection| connection.layout(sc)).collect(),
        }
    }
}
impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> View<Data> for SimulationViewLayout<'_, Data, StateLens, SimulationLens> {
    fn draw_inner(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(top_left, self.widget_size);

        let mut widget_shape = sfml::graphics::RectangleShape::from_rect(widget_rect);
        widget_shape.set_fill_color(Theme::DEFAULT.simulation_bg_color);
        target.draw(&widget_shape);

        for connection in &self.connections {
            connection.draw(app, target, top_left, hover);
        }
        for gate in &self.gates {
            gate.draw(app, target, top_left, hover);
        }
        for node in &self.nodes {
            node.draw(app, target, top_left, hover);
        }
    }

    fn find_hover(&self, top_left: sfml::system::Vector2f, mouse: sfml::system::Vector2f) -> Option<ViewId> {
        // reverse to go in z order from highest to lowest
        for node in self.nodes.iter().rev() {
            if let hover @ Some(_) = node.find_hover(top_left, mouse) {
                return hover;
            }
        }
        for gate in self.gates.iter().rev() {
            if let hover @ Some(_) = gate.find_hover(top_left, mouse) {
                return hover;
            }
        }
        for connection in self.connections.iter().rev() {
            if let hover @ Some(_) = connection.find_hover(top_left, mouse) {
                return hover;
            }
        }
        if sfml::graphics::FloatRect::from_vecs(top_left, self.widget_size).contains(mouse) {
            return Some(self.view.id);
        }

        None
    }

    fn size(&self) -> sfml::system::Vector2f {
        self.widget_size
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.view.id {
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

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, app: &crate::App, data: &mut Data, event: GeneralEvent) {
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

impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> ViewWithoutLayout<Data> for GateView<Data, StateLens, SimulationLens> {
    type WithLayout<'without_layout> = GateViewLayout<'without_layout, Data, StateLens, SimulationLens>
    where
        Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        GateViewLayout { view: self, widget_size: sc.max }
    }
}
impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> View<Data> for GateViewLayout<'_, Data, StateLens, SimulationLens> {
    fn draw(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let gate_rect = gate_rect(widget_rect, self.view.gate_location, self.view.num_inputs, self.view.num_outputs);

        if Some(self.view.id) == hover {
            // expand by hover distance, this is the "stroke weight"
            // TODO: test to see if stroke works well
            let hover_rect = sfml::graphics::FloatRect::new(
                gate_rect.left - Theme::DEFAULT.gate_hover_dist,
                gate_rect.top - Theme::DEFAULT.gate_hover_dist,
                gate_rect.width + Theme::DEFAULT.gate_hover_dist * 2.0,
                gate_rect.height + Theme::DEFAULT.gate_hover_dist * 2.0,
            );
            let mut hover_shape = sfml::graphics::RectangleShape::from_rect(hover_rect);
            hover_shape.set_fill_color(Theme::DEFAULT.gate_hover_color);
            target.draw(&hover_shape);
        }

        let mut gate_shape = sfml::graphics::RectangleShape::from_rect(gate_rect);
        gate_shape.set_fill_color(Theme::DEFAULT.gate_color);
        target.draw(&gate_shape);

        // target.text(&self.view.name).xy(gate_rect.xy()).wh(gate_rect.wh()).center_justify().align_text_middle_y().color(Theme::DEFAULT.gate_text_color); TODO
    }

    fn find_hover(&self, widget_top_left: sfml::system::Vector2f, mouse_pos: sfml::system::Vector2f) -> Option<ViewId> {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let rect = gate_rect(widget_rect, self.view.gate_location, self.view.num_inputs, self.view.num_outputs);
        if rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self.view.id);
        }

        None
    }

    fn size(&self) -> sfml::system::Vector2f {
        sfml::system::Vector2f::new(0.0, 0.0) // does not participate in layout
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if self.view.id == target {
            self.targeted_event(app, data, event);
        }
        // no other chilren to go through
    }

    fn targeted_event(&self, _: &crate::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown(mouse_pos) => {
                let cur_gate_pos = self.view.simulation_lens.with(data, |simulation| {
                    let location = Gate::location(&simulation.circuits, &simulation.gates, self.view.gate_key);
                    (location.x, location.y)
                });
                self.view.state_lens.with_mut(data, |state| state.cur_gate_drag = Some((self.view.gate_key, mouse_pos, (cur_gate_pos.0, cur_gate_pos.1))));
            }
        }
    }

    fn general_event(&self, _: &crate::App, data: &mut Data, event: GeneralEvent) {
        if self.view.being_dragged {
            match event {
                GeneralEvent::MouseMoved(mouse_pos) => {
                    if let Some((gate, mouse_start, gate_start)) = self.view.state_lens.with(data, |state| state.cur_gate_drag) {
                        // TODO: zooming and panning
                        self.view.simulation_lens.with_mut(data, |simulation| {
                            let mouse_diff = mouse_pos - mouse_start;
                            let loc = simulation::Gate::location_mut(&mut simulation.circuits, &mut simulation.gates, gate);
                            loc.x = gate_start.0 + mouse_diff.x;
                            loc.y = gate_start.1 + mouse_diff.y;
                        });
                    }
                }
                GeneralEvent::LeftMouseUp => self.view.state_lens.with_mut(data, |state| {
                    state.cur_gate_drag = None;
                }),
            }
        } else {
            // dont care
        }
    }
}
impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, simulation::Simulation>> ViewWithoutLayout<Data> for NodeView<Data, StateLens, SimulationLens> {
    type WithLayout<'without_layout> = NodeViewLayout<'without_layout, Data, StateLens, SimulationLens>
    where
        Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        NodeViewLayout { view: self, widget_size: sc.max }
    }
}
impl<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, simulation::Simulation>> View<Data> for NodeViewLayout<'_, Data, StateLens, SimulationLens> {
    fn draw(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos = node_pos(widget_rect, self.view.pos);
        if Some(self.view.id) == hover {
            let hover_rad = Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist;
            let mut hover_shape = sfml::graphics::CircleShape::new(hover_rad, 30); // TODO: put point count in theme
            hover_shape.set_origin((hover_rad, hover_rad));
            hover_shape.set_position(pos);
            hover_shape.set_fill_color(Theme::DEFAULT.node_hover_color);
            target.draw(&hover_shape);
        }

        let mut node_shape = sfml::graphics::CircleShape::new(Theme::DEFAULT.node_rad, 30); // TODO: put point count in theme
        node_shape.set_origin((Theme::DEFAULT.node_rad, Theme::DEFAULT.node_rad));
        node_shape.set_position(pos);
        node_shape.set_fill_color(self.view.color);
        target.draw(&node_shape);
    }

    fn find_hover(&self, widget_top_left: sfml::system::Vector2f, mouse_pos: sfml::system::Vector2f) -> Option<ViewId> {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos = node_pos(widget_rect, self.view.pos);
        if vector_dist(pos, mouse_pos) < Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist {
            return Some(self.view.id);
        }
        None
    }

    fn size(&self) -> sfml::system::Vector2f {
        sfml::system::Vector2f::new(0.0, 0.0)
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.view.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &crate::App, _: &mut Data, _: GeneralEvent) {}
}
impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> ViewWithoutLayout<Data> for ConnectionView<Data, StateLens, SimulationLens> {
    type WithLayout<'without_layout> = ConnectionViewLayout<'without_layout, Data, StateLens, SimulationLens>
    where
        Self: 'without_layout;

    fn layout(&self, sc: SizeConstraints) -> Self::WithLayout<'_> {
        ConnectionViewLayout { view: self, widget_size: sc.max }
    }
}
impl<Data, SimulationLens: Lens<Data, simulation::Simulation>, StateLens: Lens<Data, SimulationWidgetState>> View<Data> for ConnectionViewLayout<'_, Data, StateLens, SimulationLens> {
    fn draw(&self, app: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn sfml::graphics::RenderTarget, widget_top_left: sfml::system::Vector2f, hover: Option<ViewId>) {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos1 = node_pos(widget_rect, self.view.pos1);
        let pos2 = node_pos(widget_rect, self.view.pos2);
        let line_weight = if Some(self.view.id) == hover { Theme::DEFAULT.connection_width + Theme::DEFAULT.connection_hover_dist } else { Theme::DEFAULT.connection_width };

        let dist = vector_dist(pos1, pos2);
        let mut connection_shape = sfml::graphics::RectangleShape::new();

        connection_shape.set_size((dist, line_weight));
        connection_shape.set_origin((0.0, line_weight / 2.0));
        connection_shape.set_position(pos1);
        connection_shape.set_rotation(f32::atan2(pos2.y - pos1.y, pos2.x - pos1.x).to_degrees());
        connection_shape.set_fill_color(self.view.color);

        target.draw(&connection_shape);
    }

    fn find_hover(&self, widget_top_left: sfml::system::Vector2f, mouse_pos: sfml::system::Vector2f) -> Option<ViewId> {
        let widget_rect = sfml::graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos1 = node_pos(widget_rect, self.view.pos1);
        let pos2 = node_pos(widget_rect, self.view.pos2);
        if min_dist_to_line_squared((pos1, pos2), mouse_pos) < Theme::DEFAULT.connection_hover_dist.powf(2.0) {
            Some(self.view.id)
        } else {
            None
        }
    }

    fn size(&self) -> sfml::system::Vector2f {
        sfml::system::Vector2f::new(0.0, 0.0) // does not participate in layout
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.view.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &crate::App, _: &mut Data, _: GeneralEvent) {}
}

// TODO: reorganize all of these functions
fn gate_rect(widget_rect: sfml::graphics::FloatRect, (x, y): (f32, f32), num_inputs: usize, num_outputs: usize) -> sfml::graphics::FloatRect {
    let wh = gate_display_size(num_inputs, num_outputs);
    sfml::graphics::FloatRect::new(widget_rect.left + widget_rect.width / 2.0 + x - wh.x / 2.0, widget_rect.top + widget_rect.height / 2.0 + y - wh.y / 2.0, wh.x, wh.y)
}

fn gate_display_size(num_inputs: usize, num_outputs: usize) -> sfml::system::Vector2f {
    let gate_height = (std::cmp::max(num_inputs, num_outputs) - 1) as f32 * VERTICAL_VALUE_SPACING + GATE_EXTRA_VERTICAL_HEIGHT;
    sfml::system::Vector2f::new(GATE_WIDTH, gate_height)
}

fn y_centered_around(center_y: f32, total: usize, index: usize) -> f32 {
    let box_height: f32 = ((total - 1) as f32) * VERTICAL_VALUE_SPACING;
    let box_start_y = center_y - (box_height / 2.0);
    box_start_y + (index as f32) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(widget_rect: sfml::graphics::FloatRect, num_inputs: usize, num_outputs: usize, index: usize) -> sfml::system::Vector2f {
    sfml::system::Vector2f::new(widget_rect.left, y_centered_around(0.0, num_inputs, index))
}
fn circuit_output_pos(widget_rect: sfml::graphics::FloatRect, num_inputs: usize, num_outputs: usize, index: usize) -> sfml::system::Vector2f {
    sfml::system::Vector2f::new(widget_rect.left + widget_rect.width, y_centered_around(0.0, num_outputs, index))
}

fn gate_input_pos(widget_rect: sfml::graphics::FloatRect, gate_location: (f32, f32), num_inputs: usize, num_outputs: usize, idx: usize) -> sfml::system::Vector2f {
    let rect = gate_rect(widget_rect, gate_location, num_inputs, num_outputs);
    sfml::system::Vector2f::new(rect.left, y_centered_around(rect.top + rect.height / 2.0, num_inputs, idx))
}
fn gate_output_pos(widget_rect: sfml::graphics::FloatRect, gate_location: (f32, f32), num_inputs: usize, num_outputs: usize, idx: usize) -> sfml::system::Vector2f {
    let rect = gate_rect(widget_rect, gate_location, num_inputs, num_outputs);
    sfml::system::Vector2f::new(rect.left + rect.width, y_centered_around(rect.top + rect.height / 2.0, num_outputs, idx))
}

fn node_pos(widget_rect: sfml::graphics::FloatRect, pos: NodeViewPos) -> sfml::system::Vector2f {
    match pos {
        NodeViewPos::FarLeftEdge(i, inputs, outputs) => circuit_input_pos(widget_rect, inputs, outputs, i),
        NodeViewPos::FarRightEdge(i, inputs, outputs) => circuit_output_pos(widget_rect, inputs, outputs, i),
        NodeViewPos::LeftOfGate(gate_pos, i, inputs, outputs) => gate_input_pos(widget_rect, gate_pos, inputs, outputs, i),
        NodeViewPos::RightOfGate(gate_pos, i, inputs, outputs) => gate_output_pos(widget_rect, gate_pos, inputs, outputs, i),
    }
}

fn vector_dist_squared(a: sfml::system::Vector2f, b: sfml::system::Vector2f) -> f32 {
    (b.x - a.x).powf(2.0) + (b.y - a.y).powf(2.0)
}
fn vector_dist(a: sfml::system::Vector2f, b: sfml::system::Vector2f) -> f32 {
    vector_dist_squared(a, b).sqrt()
}

fn min_dist_to_line_squared(line_segment: (sfml::system::Vector2f, sfml::system::Vector2f), point: sfml::system::Vector2f) -> f32 {
    fn lerp(a: sfml::system::Vector2f, b: sfml::system::Vector2f, t: f32) -> sfml::system::Vector2f {
        a + (b - a) * t
    }

    let (a, b) = line_segment;

    let len_squared = vector_dist_squared(a, b);
    if len_squared == 0.0 {
        vector_dist_squared(point, a)
    } else {
        // project point onto line segment and return distance to that projected point
        let t = (point - a).dot(b - a) / len_squared;
        let t_clamped = t.clamp(0.0, 1.0);
        let projected = lerp(a, b, t_clamped);
        vector_dist_squared(point, projected)
    }
}

fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> sfml::graphics::Color {
    fn value_to_color(v: logic::Value) -> sfml::graphics::Color {
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
