# Plan for how to do Strategy Execution

## Goal

- The easiest way would be just waiting for all the measurements to arrive, so that one can guarantee, that `Current World` = `World at end of step`, or in other words: *Since the step is finished, and thus the measurements as well, the World currently stored in memory is the only basis for deciding the next command)
- This is not fully fitting into the program due to 2 important factors:
    - For visualizing maze solvers, it could be nice, to visually not just provide the next step, but also a 'look into the future', which would include going down 'potential' paths and having a branching system
    - One major concern in this speed-based application is latency: By precomputing the next `n` commands, we can have a buffer of size `n`, which would prevent the micromouse from ever having no command at all

## CommandApplication

- The CommandApplication-type represents the execution of a single command given a `starting_world` (`Map` and `Transform`)
- We can use the `potential_result`-type functions to figure out, which termination states are possible (since we only care about termination states for the purposes of deciding the next command (we do not need to consider substeps, which add no further information on their own))
- All `potential_results` are given in the `Requirement`-Form, which means, that they contain a partial map, which is the smallest common multiple of the starting world and the interrupts being on or off: The world is split into two parts:
    - The `Transform`: Is the position of the mouse at the step at which this `potential_result` would terminate
    - The `Map`: Is the minimal map (the one with the fewest 'guesses') possible, which still matches the `starting_world`, but also ensures that all potentially terminating interrupts this command could have triggered before that are off, while the condition the current interrupt needs to terminate (or the end of the command) is on
- This also means, that if a given `starting_world` (or `filter`) forces a command to finish at a certain interrupt (because that which is known about the world triggers the interrupt), it will not continue past this point (since that has been excluded by the command having to finish before that)
- In the context of the `StrategyTree`, the `CommandApplication` is useful for taking an emitted command from a `Node` of the tree (be it the `root` (the best current knowledge, what is going on) or a `potential_result` further down the line) and branching that Node's `Set of potential_results` into another `Set of potential_results` by calling the `potential-result`-function on the command application for every `potential_result` in the `Set of potential_results`(The `starting-states` or the command), which will itself return a `Set of potential_results`,which will all follow the relationship:

```condition
Given a Parent-Child relationship Set(A) --> Set(B) between two nodes:
There exists: a starting state A (A element of Set(A));
There exists: a potential_result of applying a command to that starting state B (B element of Set(B));
Such that:
B is strictly an upgrade of A
```

(Maybe it would be worthwhile to actually keep the sets separate by adding arbitrary identifieres or hashes)

## Strategy Tree

### Building

### Merging

### Pruning
