pub(crate) mod calculator;
pub(crate) mod connection;
pub(crate) mod draw;
/* TODO: this goes into location component
    pub(crate) fn calculate_locations(&mut self) {
        let positions = crate::simulation::position::calculate_locations(self);
        for (gate_i, position) in positions {
            self.get_gate_mut(gate_i).location = position;
        }
    }
*/
