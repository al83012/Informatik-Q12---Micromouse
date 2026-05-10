# Messages from backend

## MicromouseManagerEvent

```json
{
    manager_event: <EVENT>,
}
```

### Error

`<EVENT>` =

```json
{
    "MicromouseManagerError": <ERROR>
}
```

- `"ConnectionClosedPermanently"`
  The connection was closed internally (The thread used to run the messaging was dropped, the program exited standard execution and will restart)
- `{"UnknownResponse": {"faulty_text": <TEXT>}}`
  A message received from the micromouse does not match the formatting guidelines / could not be read
- `"CmdConfirmThenRequested"`
  Indicates an error in the Desync-Communication: The execution of a command was confirmed to have started, but then a request to resend that command arrived
- `{"CmdStartBeforeFinish": {"new_cmd": <COMMAND_ID>, "unfinished_cmd": <COMMAND_ID>}}`
  The micromouse sent the signs of starting a new command, but never confirmed finishing the last command that was started
- `{"CmdNotKnown": <COMMAND_ID>}`
  The start of a command was confirmed, but the command id is unknown to the backend
- `"MeasurementWithoutAssociatedCmd"`
  Tried to transform a Measurement to the current position of the micromouse, but the current command was empty internally
- `{"CmdTooLong": <COMMAND_ID>}`
  Tried to process a measurement, but the given measurement-step exceeds the maximum number of steps that command could have performed
- `{"ImpossiblePosition": <COMMAND_ID>}`
  The position which was calculated for the micromouse at the current moment is impossible as it is outside map bounds

### UpdatePosition

//TODO

### UpdatedMap

//TODO

### STOP

`<EVENT>` = `"Stop"`

A Stop was triggered (may be restarted (requires manual resetting to start square) or continued)

### RESTART

`<EVENT>` = `"Restart"`

A Restart was triggered; It should have been manually ensured, that the micromouse is correctly oriented in the starting square

### Finished Command

`<EVENT>` =

`{"FinishedCommand": {"cmd_id": <COMMAND_ID>, "require_new": <REQUIRE_NEW>}}`
  Just received the feedback from the micromouse that a command with the given command id was finished, if require_new is `true`, it indicates that the internal command queue of the micromouse is empty and it will need a new command before it can do anything


# Messages from frontend
## StrategyChange

```json
{
    "StrategyChange": <StrategyChange>
}
```
`<StrategyChange>` = 
```json
{
    "set_position" : <POSITION>, //OPTIONAL, overwrites the current position (in cases of reset etc.)
    "reset_map" : boolean, //If true, resets the map (Making the micromouse forget what it has seen, but not its position)
    "set_strategy" : <STRATEGY_CONFIG>, //OPTIONAL, the new strategy to start
    "set_goal" : <GOAL_POSITION> //OPTIONAL, overwrites the goal position
}
```

`<POSITION>` =
```json
{
    "x": u32,
    "y": u32
}
```

### Strategy Config
`<STRATEGY_CONFIG>` = 
```json
    { "<CONFIG_NAME" : <CONFIG> }
```

#### DepthFirst
`<CONFIG_NAME> = "DepthFirstConfig"`
`<CONFIG>` =
```json
{
    "forward_first" : boolean // Whether the path without turns should be preferred (instead of random order)
}
```

#### BreadthFirst
...
#### DbgKnownPath
Usecase: Is reliant on a path from the current pos to the goal already existing; Use for returning to the starting square for instance; Beware to not reset the map
`<CONFIG_NAME> = "DbgKnownPathConfig"`
...

#### FloodFill
...

#### FollowWall
...

#### RandomMove
...

## Pause
```json
"Pause"
```

Pause execution, but do not stop the strategy tree

## Continue
```json
"Continue"
```

Unpause execution, but also maintain the previous state

## Cancel
THERE IS NO CANCEL; IT SHOULD BE DONE VIA A STRATEGY CHANGE
