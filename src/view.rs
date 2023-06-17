use crate::{simulation::Simulation, view::simulation::SimulationWidget};

use nannou::prelude::*;

mod connection;
mod gate;
mod node;
mod simulation;

// mvc pattern inspired by elm architecture
pub(crate) struct View {
    sim: simulation::SimulationWidget,
}

trait Widget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, hovered: Option<&dyn Widget>);
    // iterate through this and child widgets in z order to check which one the mouse is currently over
    fn find_hover(&self, mouse: Vec2) -> Option<&dyn Widget>;

    // TODO: reconsider whether or not to use listeners
    /* TODO: figure out how this is supposed to work (mostly figure out how mouse up and mouse dragged is supposed to work, because they in theory should be global events and not specific to a widget)
    fn mouse_down(&self, mouse_loc: Vec2) -> Option<crate::Message> {
        None
    }
    fn mouse_click(&self, mouse_loc: Vec2) -> Option<crate::Message> {
        None
    }
    */
}

pub(crate) fn render(app: &nannou::App, draw: &nannou::Draw, simulation: &Simulation) {
    let view = view(app, simulation);
    let hover = view.sim.find_hover(app.mouse.position());
    view.sim.draw(simulation, draw, hover);
}

pub(crate) fn event(app: &nannou::App, simulation: &Simulation, event: nannou::Event) -> Option<crate::Message> {
    let view = view(app, simulation);
    if let nannou::Event::WindowEvent { id: _, simple: Some(event) } = event {
        None
        /* TODO
        match event {
            WindowEvent::MouseMoved(mouse_loc) => {
                if let nannou::state::mouse::ButtonPosition::Down(_) = app.mouse.buttons.left() {
                    let hovered = view.sim.find_hover(app.mouse.position());
                    if let Some(hovered) = hovered {
                        // TODO: better way of dispatching these because the same widget should get the events throughout the entire mouse drag, not the one currently being hovered over
                        // so for example if you drag a gate over another one that happens to have a higher z the other gate will steal the drag
                        hovered.mouse_dragged(mouse_loc)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            _ => None, // TODO: handle other events
        }
        */
    } else {
        None
    }
}

fn view(app: &nannou::App, simulation: &Simulation) -> View {
    View { sim: SimulationWidget::new(app.window_rect(), simulation) }
}
