# Nachrichten vom Backend

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

### Finished Command

`<EVENT>` =

`{"FinishedCommand": {"cmd_id": <COMMAND_ID>, "require_new": <REQUIRE_NEW>}}`
  Just received the feedback from the micromouse that a command with the given command id was finished, if require_new is `true`, it indicates that the internal command queue of the micromouse is empty and it will need a new command before it can do anything