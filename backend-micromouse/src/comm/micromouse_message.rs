use std::marker::PhantomData;

use tungstenite::{Message, Utf8Bytes};

use crate::direction::RelativeDirection;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandId(pub u32);
// Percentage 0-100
type Battery = u8;

type Depth = u32;

type StepNum = u32;

pub enum MicromouseMessage {
    MicromouseCommand(CommandMessage),
    MicromouseResponse(MicromouseResponse),
}

pub enum MicromouseResponse {
    Debug(String),
    Measurement(MeasurementMessage),
    CommandFinished(CommandFinishedMessage),
    Desync(Vec<CommandId>),
    Stop,
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

impl TryFrom<String> for MicromouseResponse {
    type Error = FormatError<MicromouseResponse>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.trim() {
            "STOP" => Ok(MicromouseResponse::Stop),
            "RESTART" => Ok(MicromouseResponse::Restart),
            v => {
                let parts = v.split_whitespace().collect::<Vec<_>>();
                let parts = parts.as_slice();

                match parts {
                    ["DBG", dbg_msg] => Ok(MicromouseResponse::Debug(dbg_msg.to_string())),
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
                    },
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

pub struct Command {
    pub ty: MovementType,
    pub interrupts: Vec<MeasurementInterrupt>,
}

pub enum MovementType {
    Turn(i8),
    Move(u8),
}

// Interrupt directive
pub struct MeasurementInterrupt {
    pub direction: RelativeDirection,
    pub at_step: InterruptStep,
    pub action: InterruptAction,
}

// Specific time when an interrupt happened
// Only contains a specific Step number, no action
pub struct MeasurementOccurence {
    pub direction: RelativeDirection,
    pub at_step: StepNum,
}

pub struct InterruptOccurence {
    pub occurence: MeasurementOccurence,
    pub action: InterruptAction,
}

pub struct CommandFinishedMessage {
    pub cmd_id: CommandId,
    pub reason: Option<InterruptOccurence>,
}

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

pub enum InterruptStep {
    Each,
    At(StepNum),
}
pub struct CommandMessage {
    pub cmd: Command,
    pub cmd_id: CommandId,
}

pub struct MeasurementMessage {
    pub from_cmd: CommandId,
    pub interrupt: MeasurementOccurence,
    pub depth: Depth,
    pub is_sensorlimit: bool,
}
