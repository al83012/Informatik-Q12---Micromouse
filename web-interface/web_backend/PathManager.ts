export class PathManager {
    static path_tree: {
        [key: string]: {
            from: [number, number],
            to: [number, number],
            playing: boolean,
            children: string[],
            parent: string,
            changes: {
                id: string,
                change: number,
            }[],
            moveNull: boolean,
        }
    } = {};

    static top_node_id: string = "-1";
    static is_playing: boolean = false;

    static setPlayingState(state: string) {
        this.is_playing = state === "started";
    }

    static addNode(from: [number, number], to: [number, number], id: string, parent: string) {
        if (this.top_node_id === "-1") {
            this.path_tree[parent] = {
                from: [0, 0],
                to: [0, 0],
                playing: false,
                children: [],
                parent: "-1",
                changes: [],
                moveNull: true,
            };
            this.top_node_id = parent;
        }

        this.path_tree[id] = {
            from: from,
            to: to,
            playing: false,
            children: [],
            parent: parent,
            changes: [],
            moveNull: from[0] === to[0] && from[1] === to[1],
        }

        this.path_tree[parent].children.push(id);
        this.path_tree[parent].changes.push({id: id, change: 1});

        this.updateParentChanges(parent);
    }

    static pruneNode(id: string) {
        let parent_id = this.path_tree[id].parent;
        const node_index = this.path_tree[parent_id].children.indexOf(id, 0);
        if (node_index > -1) {
            this.path_tree[parent_id].children.splice(node_index, 1);
        }


        //remove the "create" change if present and then cancel the removal
        if (this.path_tree[parent_id].changes.some(change => change.id === id && change.change === 1)) {
            for (let i = 0; i < this.path_tree[parent_id].changes.length; i++) {
                if (this.path_tree[parent_id].changes[i].id === id && this.path_tree[parent_id].changes[i].change === 1) {
                    this.path_tree[parent_id].changes.splice(i, 1);
                    return;
                }
            }
        }

        this.path_tree[parent_id].changes.push({id: id, change: -1});
        this.updateParentChanges(parent_id);

        //prune all children
        for (let child_id of this.path_tree[id].children) {
            this.pruneNode(child_id);
        }
    }

    private static updateParentChanges(node_id: string) {
        let parent_id = this.path_tree[node_id].parent;
        if (parent_id === "-1") return;
        let parent = this.path_tree[parent_id];
        if (!parent.changes.some(change => change.id === node_id)) {
            parent.changes.push({id: node_id, change: 0});
        }
        this.updateParentChanges(parent_id);
    }

    static removeNode(id: string) {
        //TODO: maybe mark this node?
    }

    private static getRelevants(node_id: string): { id: string; change: number }[][] {
        if (this.path_tree[node_id].changes.length === 0) return undefined;

        let relevant_changes: { id: string; change: number }[][] = [[]];
        let relevant_nodes: string[] = [];

        for (let change of this.path_tree[node_id].changes) {
            relevant_changes[0].push(change);
            relevant_nodes.push(change.id);
        }
        this.path_tree[node_id].changes = [];

        for (let rel_node_id of relevant_nodes) {
            let rel_changes = this.getRelevants(rel_node_id);
            if (rel_changes === undefined) continue;
            for (let i = 0; i < rel_changes.length; i++) {
                if (relevant_changes.length > i + 1) {
                    rel_changes[i].forEach(change => relevant_changes[i + 1].push(change));
                } else {
                    relevant_changes.push(rel_changes[i]);
                }
            }
        }

        //clear duplicate changes (idk why this happens :) -> turns out the server sent wrong data :) )
        relevant_changes = relevant_changes.map(group => {
            let new_group: {id: string; change: number}[] = [];
            group.forEach(change =>
                {if (!new_group.some(c => c.id === change.id && c.change === change.change)) {new_group.push(change);}});
            return new_group;
        });

        return (relevant_changes[0].length === 0) ? undefined : relevant_changes;
    }

    static getChanges(): [string, number, number][] {
        let changes: [string, number, number][] = [];

        let relevants = this.getRelevants(this.top_node_id);

        for (let i = 0; i < relevants.length; i++) {
            let relevant = relevants[i];

            let first = true;
            for (let change of relevant) {
/*                let direction: number;

                let node = this.path_tree[change.id];
                let parent_node = this.path_tree[node.parent];*/

                if (first) {
                    changes.push([change.id, change.change, 1]);
                } else {
                    changes.push([change.id, change.change, 0]);
                }
            }
        }

        console.log("CHANGES.....................................");
        console.log(changes);
        return changes;
    }

    static unfoldCompact(changes: [string, number, number][]): {
        from: [number, number],
        to: [number, number],
        change: number,
        start_from: [number, number, string], //x, y, direction (n,s,e,w)
    }[][] {
        let unfolded: {
            from: [number, number],
            to: [number, number],
            change: number,
            start_from: [number, number, string],
        }[][] = [];

        let insertion: {
            from: [number, number],
            to: [number, number],
            change: number,
            start_from: [number, number, string],
        }[][] = [];

        for (let change of changes) {
            let node = this.path_tree[change[0]];
            let from = node.from;
            let to = node.to;
            let start_node = this.path_tree[node.parent];
            let direction: string = "";


            //TODO: make it better, so the from pos is only moved, if the parent or parent of parent(moveNull) exists
            if (to[0] - from[0] > 0) { //dir = e
                direction = "e";
                if (unfolded.length !== 0) {
                    from[0] += 1;
                }
            } else if (to[0] - from[0] < 0) { //dir = w
                direction = "w";
                if (unfolded.length !== 0) {
                    from[0] -= 1;
                }
            } else if (to[1] - from[1] > 0) { //dir = s
                direction = "s";
                if (unfolded.length !== 0) {
                    from[1] += 1;
                }
            } else if (to[1] - from[1] < 0) { //dir = n
                direction = "n";
                if (unfolded.length !== 0) {
                    from[1] -= 1;
                }
            }

            let change_type = change[1];

            let part: {
                from: [number, number],
                to: [number, number],
                change: number,
                start_from: [number, number, string],
            } = {
                from: from,
                to: to,
                change: change_type,
                start_from: [start_node.to[0], start_node.to[1], direction],
            }

            let parts: {
                from: [number, number],
                to: [number, number],
                change: number,
                start_from: [number, number, string],
            }[] = [];

            for (let ar of unfolded) {
                label_c_for: for (let c of ar) {
                    if ((c.start_from[2] === part.start_from[2] || c.start_from[2] === this.flipDirection(part.start_from[2])) &&
                        (this.rangedOverlap(c.from[0], c.to[0], part.from[0], part.to[0])
                        || this.rangedOverlap(c.from[1], c.to[1], part.from[1], part.to[1]))
                    ) {
                        if (c.from[0] === -99 && c.from[1] === -99 && c.to[0] === -99 && c.to[1] === -99) {continue label_c_for;}
                        let direction: number = (this.rangedOverlap(c.from[0], c.to[0], part.from[0], part.to[0])) ? 0 : 1;
                        let positive: boolean = (c.to[direction] - c.from[direction] > 0);
                        let part_positive: boolean = (part.to[direction] - part.from[direction] > 0);
                        let bidirectional: boolean = (c.start_from[2] !== part.start_from[2]);

                        //RUles for deletion:
                        // 1. Delete the one with lower priority
                        // 2. If the one with higher prio is fully contained (isContained())
                        //    -> split the lower prio into two halfes (two parts)
                        // 3. remove from the lower prio part everything overlapping with the higher prio part
                        // 4. if the prio is the same -> start cutting from the "from" and if fully contained -> just delete

                        //priorities: "keep" over "add" over "remove"
                        if (c.change === part.change) { //prio is the same (Rule 4)
                            //remove from the from_part
                            if (this.isContained(part.from[direction], part.to[direction], c.from[direction], c.to[direction])) {

                            }
                        } else if (this.hasHigherPriority(c.change, part.change)) {
                            //c has higher priority
                            if (this.isContained(c.from[direction], c.to[direction], part.from[direction], part.to[direction])) {
                                //part is fully in c
                                //Rule 3 -> delete part by not adding it to "parts"
                            } else if (this.isContained(part.from[direction], part.to[direction], c.from[direction], c.to[direction])) {
                                //Rule 2 -> split part into two parts
                                parts = this.splitPart(part, c.from[direction], c.to[direction], direction, positive, bidirectional);
                            } else {
                                //Rule 3 -> delete Overlapping part
                                if (this.isBetween(c.from[direction], c.to[direction], part.from[direction])) {
                                    if (this.isBetween(part.from[direction], part.to[direction], c.from[direction])) {
                                        part.from[direction] = c.from[direction] + (part_positive ? 1 : -1);
                                    } else if (this.isBetween(part.from[direction], part.to[direction], c.to[direction])) {
                                        part.from[direction] = c.to[direction] + (part_positive ? 1 : -1);
                                    }
                                } else if (this.isBetween(c.from[direction], c.to[direction], part.to[direction])) {
                                    if (this.isBetween(part.from[direction], part.to[direction], c.from[direction])) {
                                        part.to[direction] = c.from[direction] + (part_positive ? -1 : 1);
                                    } else if (this.isBetween(part.from[direction], part.to[direction], c.to[direction])) {
                                        part.to[direction] = c.to[direction] + (part_positive ? -1 : 1);
                                    }
                                }
                                parts.push(part);
                            }
                        } else if (this.hasHigherPriority(part.change, c.change)) {
                            //part has higher priority
                            if (this.isContained(part.from[direction], part.to[direction], c.from[direction], c.to[direction])) {
                                //c is fully in part
                                //Rule 3 -> delete c by marking it
                                c.from = [-99, -99];
                                c.to = [-99, -99];
                                parts.push(part);
                            } else if (this.isContained(c.from[direction], c.to[direction], part.from[direction], part.to[direction])) {
                                //Rule 2 -> split part into two parts
                                insertion.push(this.splitPart(c, part.from[direction], part.to[direction], direction, part_positive, bidirectional));
                                c.from = [-101, -99]; //mark c for insertion later
                                c.to = [-99, -99];
                                parts.push(part);
                            } else {
                                //Rule 3 -> delete Overlapping part
                                if (this.isBetween(part.from[direction], part.to[direction], c.from[direction])) {
                                    if (this.isBetween(c.from[direction], c.to[direction], part.from[direction])) {
                                        c.from[direction] = part.from[direction] + (positive ? 1 : -1);
                                    } else if (this.isBetween(c.from[direction], c.to[direction], part.to[direction])) {
                                        c.from[direction] = part.to[direction] + (positive ? 1 : -1);
                                    }
                                } else if (this.isBetween(part.from[direction], part.to[direction], c.to[direction])) {
                                    if (this.isBetween(c.from[direction], c.to[direction], part.from[direction])) {
                                        c.to[direction] = part.from[direction] + (positive ? -1 : 1);
                                    } else if (this.isBetween(c.from[direction], c.to[direction], part.to[direction])) {
                                        c.to[direction] = part.to[direction] + (positive ? -1 : 1);
                                    }
                                }
                                parts.push(part);
                            }
                        }
                    }
                }
            }

            if (unfolded.length === 0) {
                unfolded.push([part]);
            }

            unfolded = unfolded.map(ar => {
                let group: {
                    from: [number, number],
                    to: [number, number],
                    change: number,
                    start_from: [number, number, string],
                }[] = [];

                for (let c of ar) {
                    if (c.from[0] === -101 && c.from[1] === -99 && c.to[0] === -99 && c.to[1] === -99) {
                        group.push(...insertion.shift());
                    } else if (!(c.from[0] === -99 && c.from[1] === -99 && c.to[0] === -99 && c.to[1] === -99)) {
                        group.push(c);
                    }
                }

                return group;
            }).filter(ar => ar.length !== 0);

            if (parts.length !== 0) {
                if (change[2] === 1) {
                    unfolded.push(parts);
                } else {
                    for (let x = 0; x < parts.length; x++) {
                        unfolded[unfolded.length - 1].push(parts[x]);
                    }
                }
            }
        }

        /*unfolded = unfolded.map(ar => ar.filter(
            c => !(c.from[0] === c.to[0] && c.from[1] === c.to[1])));*/

        return unfolded;
    }

    private static splitPart(part: {
        from: [number, number],
        to: [number, number],
        change: number,
        start_from: [number, number, string]},
                             from: number, to: number, direction: number, positive_split: boolean, bidirectional: boolean): {
        from: [number, number],
        to: [number, number],
        change: number,
        start_from: [number, number, string]
    }[] { //direction is 0=x or 1=y

        let i_start = part.from[direction];
        let i_end = (!bidirectional) ? ((positive_split) ? from-1 : from+1) : ((positive_split) ? to+1 : to-1);

        let i2_start = (!bidirectional) ? ((positive_split) ? to+1 : to-1) : ((positive_split) ? from-1 : from+1);
        let i2_end = part.to[direction];

        let part1: {
            from: [number, number],
            to: [number, number],
            change: number,
            start_from: [number, number, string],
        } = {
            from: [part.from[0], part.from[1]],
            to: [part.to[0], part.to[1]],
            change: part.change,
            start_from: [part.start_from[0], part.start_from[1], part.start_from[2]],
        }

        let part2: {
            from: [number, number],
            to: [number, number],
            change: number,
            start_from: [number, number, string],
        } = {
            from: [part.from[0], part.from[1]],
            to: [part.to[0], part.to[1]],
            change: part.change,
            start_from: [part.start_from[0], part.start_from[1], part.start_from[2]],
        }

        part1.from[direction] = i_start;
        part1.to[direction] = i_end;
        part2.from[direction] = i2_start;
        part2.to[direction] = i2_end;

        return [part1, part2];
    }

    private static hasHigherPriority(change1: number, change2: number): boolean {
        if (change1 === 0 && change2 === 1) return true;
        if (change1 === 1 && change2 === -1) return true;
        return false;
    }

    private static flipDirection(direction: string): string {
        if (direction === "e") return "w";
        if (direction === "w") return "e";
        if (direction === "s") return "n";
        if (direction === "n") return "s";
        return direction;
    }

    private static isBetween(start: number, end: number, value: number): boolean {
        return (start <= value && value <= end) || (start >= value && value >= end);
    }

    private static rangedOverlap(start1: number, end1: number, start2: number, end2: number): boolean {
        return this.isBetween(start1, end1, start2)
            || this.isBetween(start1, end1, end2)
            || this.isBetween(start2, end2, start1)
            || this.isBetween(start2, end2, end1);
    }

    private static isContained(start1: number, end1: number, start2: number, end2: number): boolean {
        /*return (start1 >= start2 && start1 > end2 && start2 >= end1 && end1 >= start2) ||
            (start1 <= start2 && start1 < end2 && start2 <= end1 && end1 <= start2);*/
        return this.isBetween(start1, end1, start2) && this.isBetween(start1, end1, end2);
    }


    static unfold(changes: [string, number, number][]): number[][] {
        let unfolded: number[][] = [];

        for (let change of changes) {
            let part: number[] = [];
            let node = this.path_tree[change[0]];
            let change_type = change[1];
            let change_group = change[2];
            let from = node.from;
            let to = node.to;

            if (to[0] - from[0] > 0) {
                for (let i = 0; i <= to[0] - from[0]; i++) {
                    part.push(from[0] + i);
                    part.push(from[1]);
                }
            } else if (to[0] - from[0] < 0) {
                for (let i = 0; i >= to[0] - from[0]; i--) {
                    part.push(from[0] + i);
                    part.push(from[1]);
                }
            } else if (to[1] - from[1] > 0) {
                for (let i = 0; i <= to[1] - from[1]; i++) {
                    part.push(from[0]);
                    part.push(from[1] + i);
                }
            } else if (to[1] - from[1] < 0) {
                for (let i = 0; i >= to[1] - from[1]; i--) {
                    part.push(from[0]);
                    part.push(from[1] + i);
                }
            }

            if (part.length === 0) continue;

            part.push(change_type);
            part.push(change_group);

            unfolded.push(part);
        }

        console.log("UNFILTERED++++++++++++++++++++++++++++");
        console.log(unfolded);
        unfolded = filterOverlappingPaths(unfolded);
        console.log("FILTERED------------------------------");
        console.log(unfolded);


        return unfolded;
    }

    static hasTopNode(): boolean {
        return this.top_node_id !== "-1";
    }

    static hasChanges(): boolean {
        return this.path_tree[this.top_node_id].changes.length !== 0;
    }

    static canPlay(): boolean {
        return this.hasChanges() && !this.is_playing;
    }
}


/**
 * Code written by claude ai for a small test is following this comment.
 * This code was only used in one test and is not used anymore (the unfold function got replaced by unfoldCompact).
 */


/**
 * Path format: [x0, y0, x1, y1, ..., xn, yn, changeType, groupId]
 * Each path is guaranteed to be straight (all points collinear).
 */
type PathData = number[];

interface ParsedPath {
    points: [number, number][];
    changeType: number;
    groupId: number;
}

function parsePath(raw: PathData): ParsedPath {
    const changeType = raw[raw.length - 2];
    const groupId = raw[raw.length - 1];
    const coords = raw.slice(0, raw.length - 2);

    const points: [number, number][] = [];
    for (let i = 0; i < coords.length; i += 2) {
        points.push([coords[i], coords[i + 1]]);
    }

    return { points, changeType, groupId };
}

function toRaw(p: ParsedPath): PathData {
    return [...p.points.flat(), p.changeType, p.groupId];
}

/** Integer cross-product collinearity check (exact, no float error) */
function isCollinear(
    a: [number, number],
    b: [number, number],
    c: [number, number]
): boolean {
    const cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    return cross === 0;
}

/** Scalar projection of `point` onto the (non-normalized) direction from `origin` */
function projectionScalar(
    origin: [number, number],
    dir: [number, number],
    point: [number, number]
): number {
    return (point[0] - origin[0]) * dir[0] + (point[1] - origin[1]) * dir[1];
}

const EPS = 1e-9;

/**
 * Filters/trims overlapping straight path parts.
 * - Exact duplicates: one copy is kept.
 * - Full containment (one segment lies entirely within another on the same
 *   line): the smaller (contained) segment is removed, the larger is kept.
 * - Partial overlaps (neither fully contains the other): the overlapping
 *   points are stripped from whichever path appears LATER in the input
 *   array, leaving it as just its non-overlapping remainder. If nothing
 *   usable remains (fewer than 2 points), that path is dropped entirely.
 */
function filterOverlappingPaths(paths: PathData[]): PathData[] {
    const parsed = paths.map(parsePath);
    const removed = new Set<number>();

    for (let i = 0; i < parsed.length; i++) {
        if (removed.has(i)) continue;
        const a = parsed[i];
        if (a.points.length < 2) {
            removed.add(i);
            continue;
        }

        const aStart = a.points[0];
        const aEnd = a.points[a.points.length - 1];
        const dir: [number, number] = [aEnd[0] - aStart[0], aEnd[1] - aStart[1]];
        const aMinT = 0;
        const aMaxT = projectionScalar(aStart, dir, aEnd);

        for (let j = i + 1; j < parsed.length; j++) {
            if (removed.has(j)) continue;
            const b = parsed[j];
            if (b.points.length < 2) {
                removed.add(j);
                continue;
            }

            const bStart = b.points[0];
            const bEnd = b.points[b.points.length - 1];

            if (!isCollinear(aStart, aEnd, bStart) || !isCollinear(aStart, aEnd, bEnd)) {
                continue; // different line entirely
            }

            const bT1 = projectionScalar(aStart, dir, bStart);
            const bT2 = projectionScalar(aStart, dir, bEnd);
            const bMin = Math.min(bT1, bT2);
            const bMax = Math.max(bT1, bT2);

            const noOverlap = bMax < aMinT - EPS || bMin > aMaxT + EPS;
            if (noOverlap) continue;

            const aContainsB = aMinT - EPS <= bMin && bMax <= aMaxT + EPS;
            const bContainsA = bMin - EPS <= aMinT && aMaxT <= bMax + EPS;

            if (aContainsB) {
                // covers the exact-duplicate case too (aContainsB && bContainsA)
                removed.add(j);
                continue;
            }

            if (bContainsA) {
                // a is the smaller, fully covered segment -> drop it, stop comparing a
                removed.add(i);
                break;
            }

            // Partial overlap: trim the later path (b) down to its non-overlapping part
            const overlapStart = Math.max(aMinT, bMin);
            const overlapEnd = Math.min(aMaxT, bMax);

            const trimmed = b.points.filter((pt) => {
                const t = projectionScalar(aStart, dir, pt);
                return t < overlapStart - EPS || t > overlapEnd + EPS;
            });

            if (trimmed.length < 2) {
                removed.add(j);
            } else {
                b.points = trimmed; // mutate in place so later comparisons see the trimmed path
            }
        }
    }

    return parsed.filter((_, idx) => !removed.has(idx)).map(toRaw);
}

// --- examples ---
// Full containment: second is dropped entirely
// filterOverlappingPaths([
//   [0, 0, 1, 0, 2, 0, 3, 0, 1, 1],
//   [1, 0, 2, 0, 1, 1],
// ]);
// => [[0,0,1,0,2,0,3,0,1,1]]

// Partial overlap: later path is trimmed to its non-overlapping remainder
// filterOverlappingPaths([
//   [0, 0, 1, 0, 2, 0, 1, 1],      // t: 0..2
//   [1, 0, 2, 0, 3, 0, 4, 0, 1, 1], // t: 1..4, overlaps [1,2]
// ]);
// => [[0,0,1,0,2,0,1,1], [3,0,4,0,1,1]]