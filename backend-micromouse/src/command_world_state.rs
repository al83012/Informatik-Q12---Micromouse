/// World state, which also stores the currently active command, so that one can simply call
/// "step_to(n)" to step to the state of the command (automatically doing all the sub-steps)
///
/// One should also be able to use all the partial and map functions (for instance measuring, etc.)
///
///
/// Usecase:
/// - If there is a new command / a command was started --> New
/// - Every time a measurement or a cmd-finished is returned --> Call step_to(n) to bring the
/// internal transform up to that step, but first: apply measurements
/// - if cmd-finished: call ".finish" --> Returns the WorldData again --> can be used for the next
/// command
