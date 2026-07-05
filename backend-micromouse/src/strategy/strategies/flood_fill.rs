use std::usize;

use serde::{Deserialize, Serialize};

use crate::{
    comm::micromouse_message::{
        Command, InterruptAction, InterruptStep, MeasurementInterrupt, MovementType,
        TransformedMovement,
    },
    map::world_data::WorldData,
    strategy::{
        strategies::utils::{self, value_map::ValueMap},
        strategy::{
            ComputedAction, ComputedActions, FromConfig, GoalPosition, Strategy,
            StrategyComputationResult, StrategyEndState, WithGraftingFilter,
        },
        strategy_tree::GraftingFilter,
    },
    transform::{
        direction::{Direction, RelativeDirection},
        position::MouseTransform,
    },
    utils::{nonempty::NonEmpty, path::Path},
};

#[derive(Clone, Debug)]
pub struct FloodFill<const N: usize> {
    config: FloodFillConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FloodFillConfig {
    rotation_cost: FFCost,
    move_cost: FFCost,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FFCost {
    base_value: usize,
    /// Between 0.0 and 1.0 --> 0.0 being no reduction and 1.0 being scaling by sqrt(streak)
    streak_reduction_factor: f32,
}

impl<const N: usize> FromConfig<N> for FloodFill<N> {
    type Config = FloodFillConfig;

    fn from_config(
        config: &Self::Config,
        starting_state: &crate::map::world_data::WorldData<N>,
    ) -> Self {
        todo!()
    }
}

impl WithGraftingFilter for FloodFillConfig {
    fn require_grafting_filter(&self) -> crate::strategy::strategy_tree::GraftingFilter {
        GraftingFilter::None
    }
}

#[derive(Clone, PartialEq)]
pub struct FlowFieldCell {
    enter_in_direction: Direction,
    total_cost: usize,
    fwd_streak: usize,
}

impl FlowFieldCell {
    pub const INF: Self = Self {
        enter_in_direction: Direction::PosX,
        total_cost: usize::MAX,
        fwd_streak: 0,
    };
}

pub type FlowField<const N: usize> = ValueMap<N, FlowFieldCell>;

impl<const N: usize> Strategy<N> for FloodFill<N> {
    fn next_cmd(
        &self,
        world: &crate::map::world_data::PartialWorldData<N>,
        goal: &crate::strategy::strategy::GoalPosition,
    ) -> crate::strategy::strategy::StrategyComputationResult<N, Self> {
        let mut flow_field = self.flow_field(world);

        let mut path = match self.path_on_flow_field(flow_field, *goal) {
            Ok(o) => o,
            Err(e) => return StrategyComputationResult::Computed(Err(e)),
        };
        let required_openings = path.required_openings();

        let start_next_move_from = path.start().clone();
        let Some(next_move) = path.one_towards_destination() else {
            return StrategyComputationResult::Computed(Err(StrategyEndState::ReachedGoal));
        };

        if let MovementType::Turn(_) = &next_move {
            return StrategyComputationResult::Computed(Ok(ComputedActions(NonEmpty::one(
                ComputedAction {
                    next_strategy_state: Some(self.clone()),
                    after_command: Command {
                        ty: next_move,
                        interrupts: vec![],
                    },
                },
            ))));
        }

        let move_dir = start_next_move_from.dir;
        let mut transformed_move = TransformedMovement::new(next_move, start_next_move_from);
        let end_next_move_at = transformed_move
            .at_step(transformed_move.max_step_count())
            .expect("In range");

        let max_x = start_next_move_from.pos.x.max(end_next_move_at.pos.x);
        let min_x = start_next_move_from.pos.x.min(end_next_move_at.pos.x);
        let max_y = start_next_move_from.pos.y.max(end_next_move_at.pos.y);
        let min_y = start_next_move_from.pos.y.min(end_next_move_at.pos.y);

        let pos_x_range = min_x..=max_x;
        let pos_y_range = min_y..=max_y;

        let required_openings_checkable_from_path =
            required_openings.into_iter().filter_map(|opening| {
                let rotation_from_move_dir = move_dir.shortest_rotate_to(&opening.dir);
                if pos_x_range.contains(&opening.pos.x)
                    && pos_y_range.contains(&opening.pos.y)
                    && rotation_from_move_dir.abs() <= 1
                {
                    let dir = match rotation_from_move_dir {
                        -1 => RelativeDirection::Right,
                        0 => RelativeDirection::Forward,
                        1 => RelativeDirection::Left,
                        _ => unreachable!("Checked"),
                    };
                    let interrupt = MeasurementInterrupt {
                        direction: dir,
                        at_step: InterruptStep::At(
                            start_next_move_from
                                .pos
                                .distance_straight_line(opening.pos)
                                .expect("On straight line"),
                        ),
                        action: InterruptAction::StopIfBlocked,
                    };
                    Some(interrupt)
                } else {
                    None
                }
            });

        StrategyComputationResult::Computed(Ok(ComputedActions(NonEmpty::one(ComputedAction {
            next_strategy_state: Some(self.clone()),
            after_command: Command {
                ty: next_move,
                interrupts: required_openings_checkable_from_path.collect(),
            },
        }))))
    }
}

impl<const N: usize> FloodFill<N> {
    pub fn flow_field(&self, world: impl AsRef<WorldData<N>>) -> FlowField<N> {
        let world = world.as_ref();

        let mut flow_field = ValueMap::new(FlowFieldCell::INF);

        let flow_start = world.mouse;
        *flow_field.value_mut(flow_start.pos).expect("Has to exist") = FlowFieldCell {
            enter_in_direction: flow_start.dir,
            total_cost: 0,
            fwd_streak: 0,
        };

        let mut propagate = vec![flow_start.pos];

        while let Some(next_propagate) = propagate.pop() {
            let cell = flow_field
                .value(next_propagate)
                .expect("Has to exist")
                .clone();

            let neighbor_dirs = vec![
                Direction::PosX,
                Direction::PosY,
                Direction::NegX,
                Direction::NegY,
            ];
            for dir in neighbor_dirs {
                let pos_offset = dir.steps_in_dir(1);
                let Some(neighbor_pos) = next_propagate + pos_offset else {
                    continue;
                };

                let neighbor = flow_field.value_mut(neighbor_pos).expect("Has to exist");

                let cost = self.cost_to_neighbor(&cell, dir);
                if cost < neighbor.total_cost {
                    propagate.push(neighbor_pos);
                    neighbor.total_cost = cost;
                    neighbor.fwd_streak = if dir == cell.enter_in_direction {
                        cell.fwd_streak + 1
                    } else {
                        1
                    };
                    neighbor.enter_in_direction = dir;
                }
            }
        }
        flow_field
    }

    pub fn cost_to_neighbor(&self, cell: &FlowFieldCell, dir_to_neighbor: Direction) -> usize {
        let rotation_amount = cell
            .enter_in_direction
            .shortest_rotate_to(&dir_to_neighbor)
            .unsigned_abs();
        let rotation_cost = self.config.rotation_cost.value(rotation_amount as usize);

        let len = if rotation_amount == 0 {
            cell.fwd_streak + 1
        } else {
            1
        };
        let previous_streak_cost = self.config.move_cost.value(len - 1);
        let current_streak_cost = self.config.move_cost.value(len);
        let additional_move_cost = current_streak_cost - previous_streak_cost;

        rotation_cost + additional_move_cost
    }

    pub fn path_on_flow_field(
        &self,
        flow_field: FlowField<N>,
        goal: GoalPosition,
    ) -> Result<Path, StrategyEndState> {
        let goal = goal.0;
        let goal_cell = flow_field.value(goal).expect("Has to exist");

        let mut path = Path::new(MouseTransform {
            pos: goal,
            dir: goal_cell.enter_in_direction.rotated(2),
        });

        let mut current_cell = goal;

        loop {
            let current_cell_flow =
                flow_field
                    .value(current_cell)
                    .ok_or(StrategyEndState::NoPossibleAction(
                        "Goal is walled off from current position".to_string(),
                    ))?;
            if current_cell_flow.total_cost == 0 {
                break;
            }

            let entered_from_dir = current_cell_flow.enter_in_direction.rotated(2);

            let entered_from_cell = (current_cell + entered_from_dir.steps_in_dir(1)).ok_or(
                StrategyEndState::NoPossibleAction(
                    "Goal is walled off from current position; Path leads to wall".to_string(),
                ),
            )?;

            let last = path.last();
            if last.dir != entered_from_dir {
                path.connect_to(MouseTransform {
                    pos: current_cell,
                    dir: entered_from_dir,
                });
            }
            path.connect_to(MouseTransform {
                pos: entered_from_cell,
                dir: entered_from_dir,
            });
            current_cell = entered_from_cell;
        }

        let path = path.reversed();

        let path = path.reduced();

        Ok(path)
    }
}

impl FFCost {
    pub fn value(&self, action_len: usize) -> usize {
        let reduction_factor = self.streak_reduction_factor.clamp(0.0, 1.0);
        let val = action_len as f32;
        val.powf(1.0 - 0.5 * reduction_factor).ceil() as usize
    }
}
