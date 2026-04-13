use std::marker::PhantomData;

use tracing::debug;
use tungstenite::{Message, Utf8Bytes};

use crate::{
    map::{
        map::{PartialMap, WallDiscoveryStatus},
        measurement::{Measurement, MeasurementValue},
        world_data::{PartialWorldData, WorldData},
    },
    transform::{direction::RelativeDirection, position::MouseTransform},
};

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CommandId(pub u32);
// Percentage 0-100
type Battery = u8;

type Depth = u32;

pub type StepNum = u32;

pub enum MicromouseMessage {
    MicromouseCommand(CommandMessage),
    MicromouseResponse(MicromouseResponse),
}

#[derive(Debug)]
pub enum MicromouseResponse {
    Debug(String),
    Measurement(MeasurementMessage),
    CommandFinished(CommandFinishedMessage),
    Desync(Vec<CommandId>),
    Stop,
    Continue,
    Restart,
    Battery(Battery),
}

pub struct FormatError<T> {
    pub faulty_text: String,
    _ty: PhantomData<T>, //Storing the type which was expected
}

impl<T> FormatError<T> {
    pub fn new(faulty_text: impl Into<String>) -> Self {
        Self {
            faulty_text: faulty_text.into(),
            _ty: PhantomData,
        }
    }

    // Any conversion from Str -> A can also be seen as a conversion from Str -> B, if B can be
    // derived from A
    pub fn map<U>(self) -> FormatError<U>
    where
        U: From<T>,
    {
        FormatError::new(self.faulty_text)
    }

    pub fn caused_by<U, A>(from_result: Result<A, FormatError<U>>) -> Result<A, Self> {
        from_result.map_err(|e| Self::new(format!("Err caused by {e}")))
    }
}

impl<T> std::fmt::Display for FormatError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(FormatError: {})", self.faulty_text)
    }
}

impl<T> std::fmt::Debug for FormatError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(FormatError: {})", self.faulty_text)
    }
}

impl TryFrom<String> for MicromouseResponse {
    type Error = FormatError<MicromouseResponse>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.trim() {
            "CONTINUE" => Ok(MicromouseResponse::Continue),
            "STOP" => Ok(MicromouseResponse::Stop),
            "RESTART" => Ok(MicromouseResponse::Restart),
            v => {
                let parts = v.split_whitespace().collect::<Vec<_>>();
                let parts = parts.as_slice();

                match parts {
                    ["DBG", dbg_msg @ ..] => {
                        Ok(MicromouseResponse::Debug(dbg_msg.join(" ").to_string()))
                    }
                    ["MEASUREMENT", cmd_id, measurement_occurence, depth] => {
                        Ok(MicromouseResponse::Measurement(MeasurementMessage {
                            from_cmd: FormatError::caused_by(CommandId::try_from(
                                cmd_id.to_string(),
                            ))?,
                            interrupt: FormatError::caused_by(MeasurementOccurence::try_from(
                                measurement_occurence.to_string(),
                            ))?,
                            depth: depth
                                .parse::<Depth>()
                                .map_err(|_e| FormatError::new(value))?,
                            is_sensorlimit: false,
                        }))
                    }
                    ["MEASUREMENT", cmd_id, measurement_occurence, depth, "SENSORLIMIT"] => {
                        Ok(MicromouseResponse::Measurement(MeasurementMessage {
                            from_cmd: FormatError::caused_by(CommandId::try_from(
                                cmd_id.to_string(),
                            ))?,
                            interrupt: FormatError::caused_by(MeasurementOccurence::try_from(
                                measurement_occurence.to_string(),
                            ))?,
                            depth: depth
                                .parse::<Depth>()
                                .map_err(|_e| FormatError::new(value))?,
                            is_sensorlimit: true,
                        }))
                    }
                    ["CMD-FINISHED", cmd_id] => Ok(MicromouseResponse::CommandFinished(
                        CommandFinishedMessage {
                            cmd_id: FormatError::caused_by(CommandId::try_from(
                                cmd_id.to_string(),
                            ))?,
                            reason: None,
                        },
                    )),

                    ["CMD-FINISHED", cmd_id, interrupt_occurence] => Ok(
                        MicromouseResponse::CommandFinished(CommandFinishedMessage {
                            cmd_id: FormatError::caused_by(CommandId::try_from(
                                cmd_id.to_string(),
                            ))?,
                            reason: Some(FormatError::caused_by(InterruptOccurence::try_from(
                                interrupt_occurence.to_string(),
                            ))?),
                        }),
                    ),
                    ["BATTERY", percent] => Ok(MicromouseResponse::Battery(
                        percent
                            .parse::<Battery>()
                            .map_err(|_e| FormatError::new(percent.to_string()))?,
                    )),
                    ["DESYNC", desynced_cmd_ids @ ..] if !desynced_cmd_ids.is_empty() => {
                        let mut des = Vec::with_capacity(desynced_cmd_ids.len());
                        for d in desynced_cmd_ids {
                            des.push(FormatError::caused_by(CommandId::try_from(d.to_string()))?)
                        }
                        Ok(MicromouseResponse::Desync(des))
                    }
                    _ => Err(FormatError::new(value)),
                }
            }
        }
    }
}

impl TryFrom<String> for MeasurementOccurence {
    type Error = FormatError<Self>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.split("_").collect::<Vec<_>>().as_slice() {
            [num, dir @ ("L" | "R" | "F")] => Ok(Self {
                direction: FormatError::caused_by(RelativeDirection::try_from(dir.to_string()))?,
                at_step: num
                    .parse::<StepNum>()
                    .map_err(|_e| FormatError::new(value))?,
            }),
            _ => Err(FormatError::new(value)),
        }
    }
}

impl TryFrom<String> for InterruptOccurence {
    type Error = FormatError<Self>;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.rsplitn(2, "_").collect::<Vec<_>>().as_slice() {
            [action, occurence @ ("STOP-IF-OPEN" | "STOP-IF-BLOCKED" | "CONTINUE")] => Ok(Self {
                occurence: FormatError::caused_by(MeasurementOccurence::try_from(
                    occurence.to_string(),
                ))?,
                action: FormatError::caused_by(InterruptAction::try_from(action.to_string()))?,
            }),
            _ => Err(FormatError::new(value)),
        }
    }
}
impl From<&CommandMessage> for Message {
    fn from(val: &CommandMessage) -> Self {
        let cmd = &val.cmd;
        let cmd_id = &val.cmd_id;
        let interrupts = &cmd.interrupts;
        let (ty_str, num) = match cmd.ty {
            MovementType::Turn(left) => ("TURN", left as i32),
            MovementType::Move(fwd) => ("MOVE", fwd as i32),
        };

        let mut interrupt_str = if interrupts.is_empty() {
            " ".to_string()
        } else {
            " MEASURE".to_string()
        };
        for i in interrupts {
            interrupt_str = format!("{interrupt_str} {i}");
        }

        let msg_str = format!("{ty_str} {cmd_id} {num}{interrupt_str}");

        Message::Text(Utf8Bytes::from(msg_str))
    }
}

impl From<&CommandMessage> for String {
    fn from(value: &CommandMessage) -> Self {
        let cmd = &value.cmd;
        let cmd_id = &value.cmd_id;
        let interrupts = &cmd.interrupts;
        let (ty_str, num) = match cmd.ty {
            MovementType::Turn(left) => ("TURN", left as i32),
            MovementType::Move(fwd) => ("MOVE", fwd as i32),
        };

        let mut interrupt_str = if interrupts.is_empty() {
            "".to_string()
        } else {
            " MEASURE".to_string()
        };
        for i in interrupts {
            interrupt_str = format!("{interrupt_str} {i}");
        }

        format!("{ty_str} {cmd_id} {num}{interrupt_str}")
    }
}

impl std::fmt::Display for MeasurementInterrupt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rel_dir = &self.direction;
        let at_step = &self.at_step;
        let action = &self.action;

        write!(f, "{at_step}_{rel_dir}_{action}")
    }
}

impl std::fmt::Display for InterruptStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Each => "X".to_string(),
                Self::At(x) => x.to_string(),
            }
        )
    }
}

impl std::fmt::Display for InterruptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                InterruptAction::Continue => "CONTINUE",
                InterruptAction::StopIfBlocked => "STOP-IF-BLOCKED",
                InterruptAction::StopIfOpen => "STOP-IF-OPEN",
            }
        )
    }
}

impl std::fmt::Display for CommandId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl TryFrom<String> for CommandId {
    type Error = FormatError<Self>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.starts_with("#") {
            Ok(CommandId(
                value[1..]
                    .parse::<u32>()
                    .map_err(|_| FormatError::new(value))?,
            ))
        } else {
            Err(FormatError::new(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Command {
    pub ty: MovementType,
    pub interrupts: Vec<MeasurementInterrupt>,
}

#[derive(Clone, Copy, Debug)]
pub enum MovementType {
    Turn(i8),
    Move(u8),
}

/// The task for an interrupt (be it terminating or continuing) --> Sent to micromouse
#[derive(Clone, Copy, Debug)]
pub struct MeasurementInterrupt {
    pub direction: RelativeDirection,
    pub at_step: InterruptStep,
    pub action: InterruptAction,
}

/// Description of what and how to perform the measurement, without the information when
#[derive(Clone, Copy, Debug)]
pub struct InterruptType {
    pub direction: RelativeDirection,
    pub action: InterruptAction,
}

impl From<MeasurementInterrupt> for InterruptType {
    fn from(value: MeasurementInterrupt) -> Self {
        Self {
            direction: value.direction,
            action: value.action,
        }
    }
}

impl From<&MeasurementInterrupt> for InterruptType {
    fn from(value: &MeasurementInterrupt) -> InterruptType {
        InterruptType {
            direction: value.direction,
            action: value.action,
        }
    }
}

// Specific time when an interrupt happened
// Only contains a specific Step number, no action
/// Direction and Step number at which a measurement took place (not what to do with it, no
/// "Each"-option)
#[derive(Clone, Copy, Debug)]
pub struct MeasurementOccurence {
    pub direction: RelativeDirection,
    pub at_step: StepNum,
}

/// MeasurementOccurence, but with the action which will be performed as a result of the interrupt
/// --> Like MeasurementInterrupt, but like the other "Occurence"-type: cannot use step-num "Each"
#[derive(Clone, Copy, Debug)]
pub struct InterruptOccurence {
    pub occurence: MeasurementOccurence,
    pub action: InterruptAction,
}

#[derive(Clone, Copy, Debug)]
pub struct CommandFinishedMessage {
    pub cmd_id: CommandId,
    pub reason: Option<InterruptOccurence>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptAction {
    Continue,
    StopIfBlocked,
    StopIfOpen,
}

impl TryFrom<String> for InterruptAction {
    type Error = FormatError<Self>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "CONTINUE" => Ok(Self::Continue),
            "STOP-IF-OPEN" => Ok(Self::StopIfOpen),
            "STOP-IF-BLOCKED" => Ok(Self::StopIfBlocked),
            _ => Err(FormatError::new(value)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum InterruptStep {
    Each,
    At(StepNum),
}
#[derive(Clone, Debug)]
pub struct CommandMessage {
    pub cmd: Command,
    pub cmd_id: CommandId,
}

#[derive(Debug)]
pub struct MeasurementMessage {
    pub from_cmd: CommandId,
    pub interrupt: MeasurementOccurence,
    pub depth: Depth,
    pub is_sensorlimit: bool,
}

/// Represents a movement in the context of its starting conditions: Does not do any work to figure
/// out interrupts; believes the movement will succeed
pub struct TransformedMovement {
    start_transform: MouseTransform,
    movement: MovementType,
}

/// Represents a command in the context of its starting conditions: Once the first measurement is
/// received, we know that a new command was started, then: we need to keep track of how the
/// command execution is moving the micromouse around
pub struct TransformedCommand<const N: usize> {
    start_transform: MouseTransform,
    command: Command,
    starting_map: PartialMap<N>,
}

// Stores one of the potential results of a transformed command --> The partial world data shows
// the transform with which the command finishes as well as the parts of the world which are
// required for this case to become true
pub struct TransformedCommandResult<const N: usize>(pub PartialWorldData<N>, pub StepNum);

impl MovementType {
    pub fn max_step_count(&self) -> usize {
        match self {
            Self::Turn(x) => x.unsigned_abs() as usize,
            Self::Move(x) => *x as usize,
        }
    }
}

impl TransformedMovement {
    pub fn new(movement: MovementType, current_transform: MouseTransform) -> Self {
        Self {
            start_transform: current_transform,
            movement,
        }
    }
    pub fn at_step(&self, n: usize) -> Option<MouseTransform> {
        if n > self.max_step_count() {
            return None;
        }
        debug!(target: "tests/map", "Movement at step {n} ({:?} & {:?}) ", self.start_transform, self.movement);
        Some(match self.movement {
            MovementType::Turn(i) => self.start_transform.rotated(i.signum() * n as i8),
            MovementType::Move(_) => self.start_transform.moved(n as u8)?,
        })
    }
    pub fn max_step_count(&self) -> usize {
        self.movement.max_step_count()
    }
}

impl InterruptStep {
    pub fn matches(&self, step_number: usize) -> bool {
        match self {
            InterruptStep::Each => true,
            InterruptStep::At(x) => *x as usize == step_number,
        }
    }
}

impl<const N: usize> TransformedCommand<N> {
    pub fn new(cmd: Command, current_world: impl Into<WorldData<N>>) -> Self {
        let world: WorldData<N> = current_world.into();

        TransformedCommand {
            start_transform: world.mouse,
            command: cmd,
            starting_map: PartialMap(world.map),
        }
    }
    //TODO: Confirm
    pub fn possible_results(&self) -> Vec<TransformedCommandResult<N>> {
        // Transformed Movement = Movement we would get, if no interrupt ever activated
        let transf_movement = TransformedMovement {
            start_transform: self.start_transform,
            movement: self.command.ty,
        };
        let max_step = transf_movement.max_step_count();
        let mut results = vec![];

        let mut step_start = PartialWorldData::new(self.starting_map, self.start_transform);
        // let mut step_start_transf = self.start_transform;

        for i in 0..max_step {
            for interrupt in self.command.interrupts.iter() {
                if !interrupt.at_step.matches(i) {
                    continue;
                }
                let measurement = step_start.measure_one(interrupt.direction);

                match (measurement, interrupt.action) {
                    (_, InterruptAction::Continue) => {
                        // Does not matter, command execution independent of wall, even if it is
                        // known
                        continue;
                    }
                    (WallDiscoveryStatus::Exists(true), InterruptAction::StopIfBlocked) => {
                        // NEEDS TO STOP, no next step
                        results.push(TransformedCommandResult(step_start, i as u32));
                        return results;
                    }
                    (WallDiscoveryStatus::Exists(true), InterruptAction::StopIfOpen) => {
                        //Continues --> Interrupt explicitly not triggered
                        continue;
                    }
                    (
                        WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited,
                        InterruptAction::StopIfBlocked,
                    ) => {
                        //Continues --> Interrupt explicitly not triggered
                        continue;
                    }
                    (
                        WallDiscoveryStatus::Exists(false) | WallDiscoveryStatus::Visited,
                        InterruptAction::StopIfOpen,
                    ) => {
                        // NEEDS TO STOP, no next step
                        results.push(TransformedCommandResult(step_start, i as u32));
                        return results;
                    }
                    (WallDiscoveryStatus::Undiscovered, _action) => {
                        // let terminating_case = match action {
                        //     InterruptAction::StopIfOpen => WallDiscoveryStatus::Exists(false),
                        //     InterruptAction::StopIfBlocked => WallDiscoveryStatus::Exists(true),
                        //     _ => unreachable!(
                        //         "Sorted out at the start, Continue-cases are not wanted here"
                        //     ),
                        // };
                        let terminating_world = step_start
                            .clone()
                            .with_interrupt_termination_triggered(
                                true,
                                interrupt.direction,
                                interrupt.action,
                            )
                            .expect("Interrupting here should be possible");
                        //The terminating option is a separate result
                        results.push(TransformedCommandResult(terminating_world, i as u32));

                        // If there is another interrupt contradicting the one that was applied
                        // here --> Will automatically be weeded out in the next iterations of the
                        // interrupt-loop <-- step_start is already adjusted to consider the only
                        // way in which this movement can be continued
                        //
                        // ALSO: Do not need to add the option "What if a later interrupt stops the
                        // program and not this one?" --> Interrupts are processed in order
                        step_start = step_start
                            .with_interrupt_termination_triggered(
                                false,
                                interrupt.direction,
                                interrupt.action,
                            )
                            .expect("Not interrupting here should be possible");
                    }
                }
            }
        }

        // Even the "normal" end of the commands has to be considered
        results.push(TransformedCommandResult(step_start, max_step as u32));

        results
    }
}

impl Command {
    pub fn max_step_count(&self) -> usize {
        self.ty.max_step_count()
    }
}

impl MeasurementInterrupt {
    // continue-interrupts cannot interrupt; keeping it open / readable for future understanding
    pub fn could_interrupt(&self) -> bool {
        self.action.could_interrupt()
    }
}

impl InterruptAction {
    pub fn could_interrupt(&self) -> bool {
        *self != Self::Continue
    }
}

impl MeasurementMessage {
    pub fn transform_by(&self, from_transform: &MouseTransform) -> Measurement {
        let pos = from_transform.pos;

        let occurence = self.interrupt;
        let rel_dir = occurence.direction;

        let dir = rel_dir.transform_by(&from_transform.dir);

        let value = if self.is_sensorlimit {
            MeasurementValue::Value { cells: self.depth }
        } else {
            MeasurementValue::OutsideRange {
                at_least_cells: self.depth,
            }
        };
        Measurement {
            value,
            direction: dir,
            position: pos,
        }
    }
}
