use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use sfml::graphics::{Shape, Transformable};

use crate::{
    graphics::{self, CenterText, RectCenter},
    simulation::{self, hierarchy, logic, Gate, GateKey, NodeKey, NodeMap, Simulation},
    theme::Theme,
    view::{
        id::{ViewId, ViewIdMaker},
        lens::Lens,
        GeneralEvent, SizeConstraints, TargetedEvent, View, ViewWithoutLayout,
    },
};

const NODE_SPACING: f32 = 20.0;

pub(crate) struct SimulationWidgetState {
    cur_gate_drag: Option<(simulation::GateKey, graphics::Vector2f, (f32, f32))>,
    view_stack: Vec<simulation::CircuitKey>,
}

impl SimulationWidgetState {
    pub(crate) fn new() -> SimulationWidgetState {
        SimulationWidgetState { cur_gate_drag: None, view_stack: Vec::new() }
    }
}

struct SimulationView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    state_lens: StateLens,

    gates: Vec<GateView<Data, StateLens, SimulationLens>>,
    nodes: Vec<NodeView<Data, StateLens, SimulationLens>>,
    connections: Vec<ConnectionView<Data, StateLens, SimulationLens>>,
}
struct SimulationViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original SimulationView<Data, StateLens, SimulationLens>,
    widget_size: graphics::Vector2f,

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
    direction: simulation::GateDirection,
    num_inputs: usize,
    num_outputs: usize,

    kind: GateViewKind,

    being_dragged: bool,

    ck_to_zoom: Option<simulation::CircuitKey>,

    font: Rc<sfml::SfBox<graphics::Font>>,

    _phantom: PhantomData<fn(&Data)>,
}
enum GateViewKind {
    Normal,
    Button(graphics::Color),
}
struct GateViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original GateView<Data, StateLens, SimulationLens>,
    widget_size: graphics::Vector2f,
}
#[derive(Copy, Clone)]
enum NodeViewPos {
    FarLeftEdge { index: usize, num_inputs: usize, num_outputs: usize },
    FarRightEdge { index: usize, num_inputs: usize, num_outputs: usize },
    GateInput { gate_pos: (f32, f32), gate_direction: simulation::GateDirection, index: usize, num_inputs: usize, num_outputs: usize },
    GateOutput { gate_pos: (f32, f32), gate_direction: simulation::GateDirection, index: usize, num_inputs: usize, num_outputs: usize },
}
struct NodeView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    // state_lens: StateLens,
    // simulation_lens: SimulationLens,

    // key: NodeKey,
    pos: NodeViewPos,
    color: graphics::Color,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}
struct NodeViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original NodeView<Data, StateLens, SimulationLens>,
    widget_size: graphics::Vector2f,
}
struct ConnectionView<Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    id: ViewId,

    // state_lens: StateLens,
    // simulation_lens: SimulationLens,

    // node1: NodeKey,
    // node2: NodeKey,
    pos1: NodeViewPos,
    pos2: NodeViewPos,
    color: graphics::Color,

    _phantom: PhantomData<fn(&Data)>,
    _phantom2: PhantomData<StateLens>,
    _phantom3: PhantomData<SimulationLens>,
}
struct ConnectionViewLayout<'original, Data, StateLens: Lens<Data, SimulationWidgetState>, SimulationLens: Lens<Data, Simulation>> {
    view: &'original ConnectionView<Data, StateLens, SimulationLens>,
    widget_size: graphics::Vector2f,
}

pub(crate) fn simulation<Data>(
    id_maker: &mut ViewIdMaker,
    state_lens: impl Lens<Data, SimulationWidgetState> + Copy,
    simulation_lens: impl Lens<Data, Simulation> + Copy,
    font: &Rc<sfml::SfBox<graphics::Font>>,
    data: &Data,
) -> impl ViewWithoutLayout<Data> {
    // TODO: show currently viewing at top of widget
    let (current_view, cur_gate_drag) = state_lens.with(data, |state| (state.view_stack.last().copied(), state.cur_gate_drag));
    let (gates, nodes, connections) = simulation_lens.with(data, |simulation| {
        let gates_currently_viewing = match current_view {
            Some(ck) => &simulation.circuits[ck].gates,
            None => &simulation.toplevel_gates,
        };

        let extra_nodes = match current_view {
            Some(ck) => {
                let circuit = &simulation.circuits[ck];
                Some(circuit.nodes.inputs().iter().chain(circuit.nodes.outputs().iter()))
            }
            None => None,
        };
        let gates = gates_currently_viewing.iter().copied();
        let nodes = gates_currently_viewing
            .iter()
            .flat_map(|gate| simulation::Gate::inputs(&simulation.circuits, &simulation.gates, *gate).iter().chain(simulation::Gate::outputs(&simulation.circuits, &simulation.gates, *gate)))
            .chain(extra_nodes.into_iter().flatten())
            .copied();

        let gate_views = gates
            .into_iter()
            .map(|gate| {
                let gate_location = Gate::location(&simulation.circuits, &simulation.gates, gate);
                let num_inputs = Gate::num_inputs(&simulation.circuits, &simulation.gates, gate);
                let num_outputs = Gate::num_outputs(&simulation.circuits, &simulation.gates, gate);
                let direction = Gate::direction(&simulation.circuits, &simulation.gates, gate);
                let gate_name = simulation.gates[gate].name(&simulation.circuits).to_string();

                GateView {
                    id: id_maker.next_id(),
                    state_lens,
                    simulation_lens,
                    gate_key: gate,
                    name: gate_name,
                    gate_location: (gate_location.x, gate_location.y),
                    direction,
                    num_inputs,
                    num_outputs,
                    kind: match &simulation.gates[gate] {
                        Gate::Nand { logic: _, location: _, direction: _ }
                        | Gate::Const { logic: _, location: _, direction: _ }
                        | Gate::Unerror { logic: _, location: _, direction: _ }
                        | Gate::Custom(_) => GateViewKind::Normal,
                        Gate::Button { logic, location: _, direction: _ } => GateViewKind::Button(node_color(&simulation.nodes, logic.nodes.outputs()[0], true)),
                    },
                    being_dragged: if let Some((cur_gate_drag, _, _)) = cur_gate_drag { cur_gate_drag == gate } else { false },
                    ck_to_zoom: if let Gate::Custom(ck) = &simulation.gates[gate] { Some(*ck) } else { None },
                    font: font.clone(),
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
                        NodeViewPos::FarLeftEdge { index: i, num_inputs, num_outputs }
                    }
                    hierarchy::NodeParentKind::CircuitOut(c, i) if Some(c) == current_view => {
                        let circuit = &simulation.circuits[c];
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        NodeViewPos::FarRightEdge { index: i, num_inputs, num_outputs }
                    }
                    hierarchy::NodeParentKind::CircuitIn(c, i) => {
                        let circuit = &simulation.circuits[c];
                        let location = &circuit.location;
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        let direction = circuit.direction;
                        NodeViewPos::GateInput { gate_pos: (location.x, location.y), gate_direction: direction, index: i, num_inputs, num_outputs }
                    }
                    hierarchy::NodeParentKind::CircuitOut(c, i) => {
                        let circuit = &simulation.circuits[c];
                        let location = &circuit.location;
                        let num_inputs = circuit.nodes.inputs().len();
                        let num_outputs = circuit.nodes.outputs().len();
                        let direction = circuit.direction;
                        NodeViewPos::GateOutput { gate_pos: (location.x, location.y), gate_direction: direction, index: i, num_inputs, num_outputs }
                    }
                    hierarchy::NodeParentKind::GateIn(g, i) => {
                        let location = &simulation::Gate::location(&simulation.circuits, &simulation.gates, g);
                        let num_inputs = simulation::Gate::num_inputs(&simulation.circuits, &simulation.gates, g);
                        let num_outputs = simulation::Gate::num_outputs(&simulation.circuits, &simulation.gates, g);
                        let direction = simulation::Gate::direction(&simulation.circuits, &simulation.gates, g);
                        NodeViewPos::GateInput { gate_pos: (location.x, location.y), gate_direction: direction, index: i, num_inputs, num_outputs }
                    }
                    hierarchy::NodeParentKind::GateOut(g, i) => {
                        let location = &simulation::Gate::location(&simulation.circuits, &simulation.gates, g);
                        let num_inputs = simulation::Gate::num_inputs(&simulation.circuits, &simulation.gates, g);
                        let num_outputs = simulation::Gate::num_outputs(&simulation.circuits, &simulation.gates, g);
                        let direction = simulation::Gate::direction(&simulation.circuits, &simulation.gates, g);
                        NodeViewPos::GateOutput { gate_pos: (location.x, location.y), gate_direction: direction, index: i, num_inputs, num_outputs }
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

    SimulationView { id: id_maker.next_id(), state_lens, gates, nodes, connections }
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
    fn draw_inner(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, top_left: graphics::Vector2f, hover: Option<ViewId>) {
        let widget_rect = graphics::FloatRect::from_vecs(top_left, self.widget_size);

        let mut widget_shape = graphics::RectangleShape::from_rect(widget_rect);
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

    fn find_hover(&self, top_left: graphics::Vector2f, mouse: graphics::Vector2f) -> Option<ViewId> {
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
        if graphics::FloatRect::from_vecs(top_left, self.widget_size).contains(mouse) {
            return Some(self.view.id);
        }

        None
    }

    fn size(&self) -> graphics::Vector2f {
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

    fn targeted_event(&self, _: &crate::App, data: &mut Data, event: TargetedEvent) {
        match event {
            TargetedEvent::LeftMouseDown(_) => {}

            TargetedEvent::RightMouseDown(_) => {
                // TODO: find a better event for this (probably keyboard shortcut)
                self.view.state_lens.with_mut(data, |state| state.view_stack.pop());
            }
        }
    }
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
    fn draw(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let gate_rect = gate_rect(widget_rect, self.view.gate_location, self.view.direction, self.view.num_inputs, self.view.num_outputs);

        if Some(self.view.id) == hover {
            // expand by hover distance, this is the "stroke weight"
            // TODO: test to see if stroke works well
            let hover_rect = graphics::FloatRect::new(
                gate_rect.left - Theme::DEFAULT.gate_hover_dist,
                gate_rect.top - Theme::DEFAULT.gate_hover_dist,
                gate_rect.width + Theme::DEFAULT.gate_hover_dist * 2.0,
                gate_rect.height + Theme::DEFAULT.gate_hover_dist * 2.0,
            );
            let mut hover_shape = graphics::RectangleShape::from_rect(hover_rect);
            hover_shape.set_fill_color(Theme::DEFAULT.gate_hover_color);
            target.draw(&hover_shape);
        }

        let mut gate_shape = graphics::RectangleShape::from_rect(gate_rect);
        gate_shape.set_fill_color(Theme::DEFAULT.gate_color);
        target.draw(&gate_shape);

        match self.view.kind {
            GateViewKind::Normal => {
                let mut text = graphics::Text::new(&self.view.name, &self.view.font, 10); // TODO: put font size into theme
                text.set_fill_color(Theme::DEFAULT.gate_text_color);
                text.center();
                text.set_position(gate_rect.center());
                target.draw(&text);
            }
            GateViewKind::Button(color) => {
                // TODO: make this into a separate view
                let rad = f32::min(gate_rect.width / 2.0, gate_rect.height / 2.0);
                let mut ellipse = graphics::CircleShape::new(rad, 20); // TODO: also put point count into theme
                ellipse.set_fill_color(color);
                ellipse.set_origin((rad, rad));
                ellipse.set_position(gate_rect.center());
                target.draw(&ellipse);
            }
        }
    }

    fn find_hover(&self, widget_top_left: graphics::Vector2f, mouse_pos: graphics::Vector2f) -> Option<ViewId> {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let rect = gate_rect(widget_rect, self.view.gate_location, self.view.direction, self.view.num_inputs, self.view.num_outputs);
        if rect.contains(mouse_pos) {
            // TODO: hover distance
            return Some(self.view.id);
        }

        None
    }

    fn size(&self) -> graphics::Vector2f {
        graphics::Vector2f::new(0.0, 0.0) // does not participate in layout
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
                // TODO: remove dragging
                let cur_gate_pos = self.view.simulation_lens.with(data, |simulation| {
                    let location = Gate::location(&simulation.circuits, &simulation.gates, self.view.gate_key);
                    (location.x, location.y)
                });
                self.view.state_lens.with_mut(data, |state| state.cur_gate_drag = Some((self.view.gate_key, mouse_pos, (cur_gate_pos.0, cur_gate_pos.1))));

                match self.view.kind {
                    GateViewKind::Button(_) => {
                        // TODO: put this into a separate view
                        self.view.simulation_lens.with_mut(data, |simulation| {
                            if let Gate::Button { logic, location: _, direction: _ } = &simulation.gates[self.view.gate_key] {
                                let current_value = logic::get_node_production(&simulation.nodes, logic.nodes.outputs()[0]);
                                let inverted = match current_value {
                                    Some(logic::Value::H) => logic::Value::L,
                                    Some(logic::Value::L) => logic::Value::H,
                                    Some(logic::Value::Z) => logic::Value::L,
                                    Some(logic::Value::X) => logic::Value::L,
                                    None => logic::Value::L,
                                };
                                logic::set_node_production(&mut simulation.nodes, logic.nodes.outputs()[0], inverted);
                            }
                        });
                    }
                    GateViewKind::Normal => {}
                }
            }
            TargetedEvent::RightMouseDown(_) => {
                // TODO: find better event for this (probably make a popup with a button)
                if let Some(ck_zoom) = self.view.ck_to_zoom {
                    self.view.state_lens.with_mut(data, |state| state.view_stack.push(ck_zoom));
                }
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
    fn draw(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos = node_pos(widget_rect, self.view.pos);
        if Some(self.view.id) == hover {
            let hover_rad = Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist;
            let mut hover_shape = graphics::CircleShape::new(hover_rad, 30); // TODO: put point count in theme
            hover_shape.set_origin((hover_rad, hover_rad));
            hover_shape.set_position(pos);
            hover_shape.set_fill_color(Theme::DEFAULT.node_hover_color);
            target.draw(&hover_shape);
        }

        let mut node_shape = graphics::CircleShape::new(Theme::DEFAULT.node_rad, 30); // TODO: put point count in theme
        node_shape.set_origin((Theme::DEFAULT.node_rad, Theme::DEFAULT.node_rad));
        node_shape.set_position(pos);
        node_shape.set_fill_color(self.view.color);
        target.draw(&node_shape);
    }

    fn find_hover(&self, widget_top_left: graphics::Vector2f, mouse_pos: graphics::Vector2f) -> Option<ViewId> {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos = node_pos(widget_rect, self.view.pos);
        if vector_dist(pos, mouse_pos) < Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist {
            return Some(self.view.id);
        }
        None
    }

    fn size(&self) -> graphics::Vector2f {
        graphics::Vector2f::new(0.0, 0.0)
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
    fn draw(&self, app: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        self.draw_inner(app, target, widget_top_left, hover);
    }
    fn draw_inner(&self, _: &crate::App, target: &mut dyn graphics::RenderTarget, widget_top_left: graphics::Vector2f, hover: Option<ViewId>) {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos1 = node_pos(widget_rect, self.view.pos1);
        let pos2 = node_pos(widget_rect, self.view.pos2);
        let line_weight = if Some(self.view.id) == hover { Theme::DEFAULT.connection_width + Theme::DEFAULT.connection_hover_dist } else { Theme::DEFAULT.connection_width };

        let dist = vector_dist(pos1, pos2);
        let mut connection_shape = graphics::RectangleShape::new();

        connection_shape.set_size((dist, line_weight));
        connection_shape.set_origin((0.0, line_weight / 2.0));
        connection_shape.set_position(pos1);
        connection_shape.set_rotation(f32::atan2(pos2.y - pos1.y, pos2.x - pos1.x).to_degrees());
        connection_shape.set_fill_color(self.view.color);

        target.draw(&connection_shape);
    }

    fn find_hover(&self, widget_top_left: graphics::Vector2f, mouse_pos: graphics::Vector2f) -> Option<ViewId> {
        let widget_rect = graphics::FloatRect::from_vecs(widget_top_left, self.widget_size);
        let pos1 = node_pos(widget_rect, self.view.pos1);
        let pos2 = node_pos(widget_rect, self.view.pos2);
        if min_dist_to_line_squared((pos1, pos2), mouse_pos) < Theme::DEFAULT.connection_hover_dist.powf(2.0) {
            Some(self.view.id)
        } else {
            None
        }
    }

    fn size(&self) -> graphics::Vector2f {
        graphics::Vector2f::new(0.0, 0.0) // does not participate in layout
    }

    fn send_targeted_event(&self, app: &crate::App, data: &mut Data, target: ViewId, event: TargetedEvent) {
        if target == self.view.id {
            self.targeted_event(app, data, event);
        }
    }

    fn targeted_event(&self, _: &crate::App, _: &mut Data, _: TargetedEvent) {}
    fn general_event(&self, _: &crate::App, _: &mut Data, _: GeneralEvent) {}
}

fn gate_rect(widget_rect: graphics::FloatRect, gate_pos: (f32, f32), direction: simulation::GateDirection, num_inputs: usize, num_outputs: usize) -> graphics::FloatRect {
    let gate_size = gate_display_size(direction, num_inputs, num_outputs);
    graphics::FloatRect::from_vecs(widget_rect.center() + gate_pos.into() - gate_size / 2.0, gate_size)
}

fn gate_display_size(direction: simulation::GateDirection, num_inputs: usize, num_outputs: usize) -> graphics::Vector2f {
    const EXTRA_SPACE: f32 = 40.0;
    const FIXED_SIZE: f32 = 50.0;

    let variable_size = (std::cmp::max(num_inputs, num_outputs) - 1) as f32 * NODE_SPACING + EXTRA_SPACE;
    match direction {
        // nodes on left and right - height variable, width constant
        simulation::GateDirection::LTR | simulation::GateDirection::RTL => graphics::Vector2f::new(FIXED_SIZE, variable_size),

        // nodes on top and botton - width variable, height constant
        simulation::GateDirection::TTB | simulation::GateDirection::BTT => graphics::Vector2f::new(variable_size, FIXED_SIZE),
    }
}

fn coord_centered_around(center: f32, total: usize, index: usize) -> f32 {
    let box_height: f32 = ((total - 1) as f32) * NODE_SPACING;
    let box_start_y = center - (box_height / 2.0);
    box_start_y + (index as f32) * NODE_SPACING
}

fn circuit_input_pos(widget_rect: graphics::FloatRect, num_inputs: usize, num_outputs: usize, index: usize) -> graphics::Vector2f {
    graphics::Vector2f::new(widget_rect.left, coord_centered_around(widget_rect.center().y, num_inputs, index))
}
fn circuit_output_pos(widget_rect: graphics::FloatRect, num_inputs: usize, num_outputs: usize, index: usize) -> graphics::Vector2f {
    graphics::Vector2f::new(widget_rect.left + widget_rect.width, coord_centered_around(widget_rect.center().y, num_outputs, index))
}

fn gate_input_pos(widget_rect: graphics::FloatRect, gate_location: (f32, f32), direction: simulation::GateDirection, num_inputs: usize, num_outputs: usize, idx: usize) -> graphics::Vector2f {
    let rect = gate_rect(widget_rect, gate_location, direction, num_inputs, num_outputs);

    let graphics::Vector2 { x: center_x, y: center_y } = rect.center();
    let left_x = rect.left;
    let right_x = rect.left + rect.width;
    let top_y = rect.top;
    let bottom_y = rect.top + rect.height;

    match direction {
        simulation::GateDirection::LTR => graphics::Vector2f::new(left_x, coord_centered_around(center_y, num_inputs, idx)),
        simulation::GateDirection::RTL => graphics::Vector2f::new(right_x, coord_centered_around(center_y, num_inputs, idx)),
        simulation::GateDirection::TTB => graphics::Vector2f::new(coord_centered_around(center_x, num_inputs, idx), top_y),
        simulation::GateDirection::BTT => graphics::Vector2f::new(coord_centered_around(center_x, num_inputs, idx), bottom_y),
    }
}
fn gate_output_pos(widget_rect: graphics::FloatRect, gate_location: (f32, f32), direction: simulation::GateDirection, num_inputs: usize, num_outputs: usize, idx: usize) -> graphics::Vector2f {
    let rect = gate_rect(widget_rect, gate_location, direction, num_inputs, num_outputs);

    let graphics::Vector2 { x: center_x, y: center_y } = rect.center();
    let left_x = rect.left;
    let right_x = rect.left + rect.width;
    let top_y = rect.top;
    let bottom_y = rect.top + rect.height;

    match direction {
        simulation::GateDirection::LTR => graphics::Vector2f::new(right_x, coord_centered_around(center_y, num_outputs, idx)),
        simulation::GateDirection::RTL => graphics::Vector2f::new(left_x, coord_centered_around(center_y, num_outputs, idx)),
        simulation::GateDirection::TTB => graphics::Vector2f::new(coord_centered_around(center_x, num_outputs, idx), bottom_y),
        simulation::GateDirection::BTT => graphics::Vector2f::new(coord_centered_around(center_x, num_outputs, idx), top_y),
    }
}

fn node_pos(widget_rect: graphics::FloatRect, pos: NodeViewPos) -> graphics::Vector2f {
    match pos {
        NodeViewPos::FarLeftEdge { index, num_inputs, num_outputs } => circuit_input_pos(widget_rect, num_inputs, num_outputs, index),
        NodeViewPos::FarRightEdge { index, num_inputs, num_outputs } => circuit_output_pos(widget_rect, num_inputs, num_outputs, index),
        NodeViewPos::GateInput { gate_pos, gate_direction, index, num_inputs, num_outputs } => gate_input_pos(widget_rect, gate_pos, gate_direction, num_inputs, num_outputs, index),
        NodeViewPos::GateOutput { gate_pos, gate_direction, index, num_inputs, num_outputs } => gate_output_pos(widget_rect, gate_pos, gate_direction, num_inputs, num_outputs, index),
    }
}

fn vector_dist_squared(a: graphics::Vector2f, b: graphics::Vector2f) -> f32 {
    (b.x - a.x).powf(2.0) + (b.y - a.y).powf(2.0)
}
fn vector_dist(a: graphics::Vector2f, b: graphics::Vector2f) -> f32 {
    vector_dist_squared(a, b).sqrt()
}

fn min_dist_to_line_squared(line_segment: (graphics::Vector2f, graphics::Vector2f), point: graphics::Vector2f) -> f32 {
    fn lerp(a: graphics::Vector2f, b: graphics::Vector2f, t: f32) -> graphics::Vector2f {
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

fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> graphics::Color {
    fn value_to_color(v: logic::Value) -> graphics::Color {
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
