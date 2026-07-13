import {record} from './recorder.cjs';
import { getFilteredChanges } from './getFilteredChanges.cjs';

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
    static has_changes: boolean = false;

    static setPlayingState(state: string) {
        record({state: state}, "Call -> SetPlayingState");
        this.is_playing = state === "started";
        record({}, "Finished -> SetPlayingState");
    }

    static addNode(from: [number, number], to: [number, number], id: string, parent: string) {
        record({
            from: from,
            to: to,
            id: id,
            parent: parent,
        }, "Call -> AddNode");

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

        if (this.path_tree[id] !== undefined) {
            return;
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

        //this.updateParentChanges(parent);

        this.has_changes = true;

        record({
            from: from,
            to: to,
            id: id,
            parent: parent,
            path_tree: this.path_tree,
        }, "Finished -> AddNode");
    }

    static pruneNode(id: string, no_parent: boolean = false) {
        if (!no_parent) {
            let parent_id = this.path_tree[id].parent;
            const node_index = this.path_tree[parent_id].children.indexOf(id, 0);
            if (node_index > -1) {
                this.path_tree[parent_id].children.splice(node_index, 1);
            } else {
                return;
            }
        }

        let child_list = this.path_tree[id].children.slice();
        delete this.path_tree[id];
        for (let i = 0; i < child_list.length; i++) {
            this.pruneNode(child_list[i], true);
        }

        this.has_changes = true;


        //remove the "create" change if present and then cancel the removal
        /*if (this.path_tree[parent_id].changes.some(change => change.id === id && change.change === 1)) {
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
        }*/
    }

    private static updateParentChanges(node_id: string) {
        record({node_id: node_id}, "Call -> updateParentChanges");
        let parent_id = this.path_tree[node_id].parent;
        if (parent_id === "-1") return;
        let parent = this.path_tree[parent_id];
        if (!parent.changes.some(change => change.id === node_id)) {
            parent.changes.push({id: node_id, change: 0});
        }
        this.updateParentChanges(parent_id);
        record({}, "Finished -> updateParentChanges");
    }

    static removeNode(id: string) {
        //TODO: maybe mark this node?
    }

    private static getRelevants(node_id: string): { id: string; change: number }[][] {
        record({node_id: node_id, PathTree: this.path_tree}, "Call -> getRelevants");

        if (this.path_tree[node_id].changes.length === 0) {
            record({}, "Finished -> getRelevants by returning null");
            return undefined;
        }

        let relevant_changes: { id: string; change: number }[][] = [[]];
        let relevant_nodes: string[] = [];

        for (let change of this.path_tree[node_id].changes) {
            relevant_nodes.push(change.id);
            if (this.path_tree[change.id].moveNull) {continue;}

            relevant_changes[0].push(change);
        }
        this.path_tree[node_id].changes = [];

        for (let rel_node_id of relevant_nodes) {
            let rel_changes = this.getRelevants(rel_node_id);
            record({IsRelChangesUndefined: rel_changes === undefined}, "Middle -> getRelevants");
            if (rel_changes === undefined) {continue;}
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

        record({Return: (relevant_changes[0].length === 0 && relevant_changes.length === 1) ? undefined : relevant_changes}, "Finished -> getRelevants");
        return (relevant_changes[0].length === 0 && relevant_changes.length === 1) ? undefined : relevant_changes;
    }

    private static completeChanges(node_id: string = this.top_node_id) {
        record({node_id: node_id}, "Call -> completeChanges");
        if (node_id === "-1") return;
        let node = this.path_tree[node_id];

        for (let child_id of node.children) {
            if (!(node.changes.some(change => change.id === child_id))) {
                node.changes.push({id: child_id, change: 0});
            }
            this.completeChanges(child_id);
        }

        record({}, "Finished -> completeChanges");
    }

    static getChanges(): [string, number, number][] {
        record({PathTree: this.path_tree}, "Call -> getChanges");

        let changes: [string, number, number][] = [];

        this.completeChanges(this.top_node_id);

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

        //console.log("CHANGES.....................................");
        //console.log(changes);
        record({changes: changes}, "Finished -> getChanges");
        return changes;
    }

    private static calculateOverlap(
    c: {
        from: [number, number]
        to: [number, number]
        change: number
        start_from: [number, number, string]
    }, part: {
        from: [number, number]
        to: [number, number]
        change: number
        start_from: [number, number, string]
    }, _parts: {
        from: [number, number]
        to: [number, number]
        change: number
        start_from: [number, number, string]
    }[], _insertion: {
        from: [number, number]
        to: [number, number]
        change: number
        start_from: [number, number, string]
    }[][]) {
        let direction: number = (this.rangedOverlap(c.from[0], c.to[0], part.from[0], part.to[0])) ? 0 : 1;
        let positive: boolean = (c.to[direction] - c.from[direction] > 0);
        let part_positive: boolean = (part.to[direction] - part.from[direction] > 0);
        let bidirectional: boolean = (c.start_from[2] !== part.start_from[2]);

        let parts: {
            from: [number, number]
            to: [number, number]
            change: number
            start_from: [number, number, string]
        }[] = [];

        let insertion: {
            from: [number, number]
            to: [number, number]
            change: number
            start_from: [number, number, string]
        }[][] = [];

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
                c.from = [-99, -99];
                c.to = [-99, -99];
                parts.push(part);
            } else if (this.isContained(c.from[direction], c.to[direction], part.from[direction], part.to[direction])) {
                //ignore to skip adding "part"
            } else {
                if (this.isBetween(part.from[direction], part.to[direction], c.from[direction])) {
                    c.from[direction] = (bidirectional) ? (part.from[direction] + (positive ? 1 : -1)) : (part.to[direction] + (positive ? 1 : -1));
                } else if (this.isBetween(part.from[direction], part.to[direction], c.to[direction])) {
                    c.to[direction] = (bidirectional) ? (part.to[direction] + (positive ? -1 : 1)) : (part.from[direction] + (positive ? -1 : 1));
                }
                parts.push(part);
            }
        } else if (this.hasHigherPriority(c.change, part.change)) {
            //c has higher priority
            if (this.isContained(c.from[direction], c.to[direction], part.from[direction], part.to[direction])) {
                //part is fully in c
                //Rule 3 -> delete part by not adding it to "parts"
            } else if (this.isContained(part.from[direction], part.to[direction], c.from[direction], c.to[direction])) {
                //Rule 2 -> split part into two parts
                parts = this.splitPart(part, c.from[direction], c.to[direction], direction, positive, bidirectional);
                if (part.from[direction] === c.from[direction]) {
                    parts.shift();
                } else if (part.to[direction] === c.to[direction]) {
                    parts.pop();
                }
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
                let insert = this.splitPart(c, part.from[direction], part.to[direction], direction, part_positive, bidirectional);
                if (part.from[direction] === c.from[direction]) {
                    insert.shift();
                } else if (part.to[direction] === c.to[direction]) {
                    insert.pop();
                }
                insertion.push(insert);
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

        return {parts: parts, insertion: insertion};
    }




    static convert_directionStr_to_FieldIndex(direction: string): number {
        if (direction === "PosX") return 0;
        if (direction === "PosY") return 1;
        if (direction === "NegX") return 2;
        return 3;
    }

    static convertCompact(): {
        from: [number, number],
        to: [number, number],
        direction: string
    }[] {

        this.has_changes = false;

        //           PosX     PosY     NegX     NegY
        let field: [boolean, boolean, boolean, boolean][][] = [];
        for (let x = 0; x < 16; x++) {
            field.push([]);
            for (let y = 0; y < 16; y++) {
                field[x].push([false, false, false, false]);
            }
        }

        const allNodes = [this.top_node_id];
        (function traverse(nodeId, tree) {
            const node = tree[nodeId];
            if (!node) return;
            if (Array.isArray(node.children)) node.children.forEach(c => allNodes.push(c));
            if (Array.isArray(node.children)) node.children.forEach(c => traverse(c, tree));
        })(this.top_node_id, this.path_tree);

        for (let id of allNodes) {
            let node = this.path_tree[id];

            if (node.moveNull) continue;

            if (node.to[1] - node.from[1] < 0) {
                for (let j = 0; j >= node.to[1] - node.from[1]; j--) {
                    field[node.from[0]][node.from[1] + j][3] = true;
                }
            } else if (node.to[1] - node.from[1] > 0) {
                for (let j = 0; j <= node.to[1] - node.from[1]; j++) {
                    field[node.from[0]][node.from[1] + j][1] = true;
                }
            } else if (node.to[0] - node.from[0] > 0) {
                for (let j = 0; j <= node.to[0] - node.from[0]; j++) {
                    field[node.from[0] + j][node.from[1]][0] = true;
                }
            } else {
                for (let j = 0; j >= node.to[0] - node.from[0]; j--) {
                    field[node.from[0] + j][node.from[1]][2] = true;
                }
            }
        }

        //console.log(field);

        let x = 0;
        let y = 0;
        let direction: string;

        let splits: [number, number, string][] = [[0,0, (field[0][0][0] ? "PosX" : (field[0][0][1] ? "PosY" : "-1"))]];

        let paths: {
            from: [number, number],
            to: [number, number],
            direction: string,
        }[] = [];

        while(splits.length > 0) {              //field.some(x => x.some(y => y.some(v => v)))) { //are there any true left
            let split = splits.shift();
            x = split[0];
            y = split[1];
            direction = split[2];

            if (!field[x][y][this.convert_directionStr_to_FieldIndex(direction)]) continue;

            let path: {
                from: [number, number],
                to: [number, number],
                direction: string,
            } = {
                from: [x, y],
                to: [x, y],
                direction: direction
            };

            if (direction === "-1") {
                break;
            } else if (direction === "PosX") {
                for (let j = 0; j <= 15-x; j++) {
                    if (field[x+j][y][0]) {
                        path.to[0] = x+j;
                        field[x+j][y][0] = false;
                    } else {
                        break;
                    }

                    if (y < 15) {
                        if (field[x + j][y][1]) {
                            splits.push([x + j, y + 1, "PosY"]);
                        }
                    }
                    if (y > 0) {
                        if (field[x + j][y][3]) {
                            splits.push([x + j, y - 1, "NegY"]);
                        }
                    }
                }
            } else if (direction === "NegX") {
                for (let j = 0; j <= x; j++) {
                    if (field[x-j][y][2]) {
                        path.to[0] = x-j;
                        field[x-j][y][2] = false;
                    } else {
                        break;
                    }

                    if (y < 15) {
                        if (field[x - j][y][1]) {
                            splits.push([x - j, y + 1, "PosY"]);
                        }
                    }
                    if (y > 0) {
                        if (field[x + j][y][3]) {
                            splits.push([x - j, y - 1, "NegY"]);
                        }
                    }
                }
            } else if (direction === "PosY") {
                for (let j = 0; j <= 15-y; j++) {
                    if (field[x][y+j][1]) {
                        path.to[1] = y+j;
                        field[x][y+j][1] = false;
                    } else {
                        break;
                    }

                    if (x < 15) {
                        if (field[x][y + j][0]) {
                            splits.push([x + 1, y + j, "PosX"]);
                        }
                    }
                    if (x > 0) {
                        if (field[x][y + j][2]) {
                            splits.push([x - 1, y + j, "NegX"]);
                        }
                    }
                }
            } else if (direction === "NegY") {
                for (let j = 0; j <= y; j++) {
                    if (field[x][y-j][3]) {
                        path.to[0] = y-j;
                        field[x][y-j][3] = false;
                    } else {
                        break;
                    }

                    if (x < 15) {
                        if (field[x][y - j][0]) {
                            splits.push([x + 1, y - j, "PosX"]);
                        }
                    }
                    if (x > 0) {
                        if (field[x][y - j][2]) {
                            splits.push([x - 1, y - j, "NegX"]);
                        }
                    }
                }
            }

            paths.push(path);
        }

        return paths;
    }

    static unfoldCompactTest(): {
        from: [number, number],
        to: [number, number],
        change: number,
        start_from: [number, number, string], //x, y, direction (n,s,e,w)
    }[][] {
        let unfold = [];

        let changes = getFilteredChanges(this.top_node_id, this.path_tree);

        for (let change of changes) {
            let parent = this.path_tree[change.id].parent;
            let p_node = this.path_tree[parent];
            let direction;
            if (change.to[0] - change.from[0] > 0) {
                direction = "e";
            } else if (change.to[0] - change.from[0] < 0) {
                direction = "w";
            } else if (change.to[1] - change.from[1] > 0) {
                direction = "s";
            } else {
                direction = "n";
            }

            unfold.push([{
                from: change.from,
                to: change.to,
                change: change.change,
                start_from: [p_node.to[0], p_node.to[1], direction]
            }]);
        }

        return unfold
    }

    static unfoldCompact(changes: [string, number, number][]): {
        from: [number, number],
        to: [number, number],
        change: number,
        start_from: [number, number, string], //x, y, direction (n,s,e,w)
    }[][] {
        record({changes: changes}, "Call -> unfoldCompact");
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

        let edit_changes: string[] = [];

        for (let i = 0; i < changes.length; i++) {
            record({Iteration: i, Change: changes[i], Changes: changes, Unfolded: unfolded}, "Iteration -> unfoldCompact");

            let change = changes[i];
            let node = this.path_tree[change[0]];
            let from = node.from;
            let to = node.to;
            let start_node = this.path_tree[node.parent];
            let direction: string = "";
            let change_type = change[1];
            let move_from_position = 0;


            //TODO: make it better, so the from pos is only moved, if the parent or parent of parent(moveNull) exists
            if (node.parent !== "-1") {
                let parent = node.parent;
                if (this.path_tree[parent].moveNull && this.path_tree[parent].parent !== "-1") {
                    parent = this.path_tree[parent].parent;
                }

                for (let c of changes) {
                    if (c[0] === parent) {
                        let type = c[1];
                        if (this.hasHigherPriority(change_type, type)) {
                            edit_changes.push(c[0]);
                        } else {
                            move_from_position = 1;
                        }

                        break;
                    }
                }

            }
            if (to[0] - from[0] > 0) { //dir = e
                direction = "e";
                from[0] += move_from_position;
                if (edit_changes.some(c => c === change[0])) {
                    to[0] -= 1;
                }
            } else if (to[0] - from[0] < 0) { //dir = w
                direction = "w";
                from[0] -= move_from_position;
                if (edit_changes.some(c => c === change[0])) {
                    to[0] += 1;
                }
            } else if (to[1] - from[1] > 0) { //dir = s
                direction = "s";
                from[1] += move_from_position;
                if (edit_changes.some(c => c === change[0])) {
                    to[1] -= 1;
                }
            } else if (to[1] - from[1] < 0) { //dir = n
                direction = "n";
                from[1] -= move_from_position;
                if (edit_changes.some(c => c === change[0])) {
                    to[1] += 1;
                }
            }

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

            let has_overlap = false;
            for (let ar of unfolded) {
                label_c_for: for (let c of ar) {
                    if ((c.start_from[2] === part.start_from[2] || c.start_from[2] === this.flipDirection(part.start_from[2])) &&
                        ((this.rangedOverlap(c.from[0], c.to[0], part.from[0], part.to[0])  && !(c.from[0] === c.to[0] || part.from[0] === part.to[0]))
                        || (this.rangedOverlap(c.from[1], c.to[1], part.from[1], part.to[1])  && !(c.from[1] === c.to[1] || part.from[1] === part.to[1])))
                    ) {
                        has_overlap = true;
                        if (c.from[0] === -99 && c.from[1] === -99 && c.to[0] === -99 && c.to[1] === -99) {continue label_c_for;}
                        let calculated_overlap = this.calculateOverlap(c, part, parts, insertion);
                        parts.push(...calculated_overlap.parts);
                        insertion.push(...calculated_overlap.insertion);
                    } else if (part.from[0] === part.to[0] && part.from[1] === part.to[1]) {
                        has_overlap = true;
                    }
                }
            }

            if (unfolded.length === 0) {
                unfolded.push([part]);
            } else if (!has_overlap) {
                parts.push(part);
            } else {
                //test parts for every possible overlap it could
                // have with unfolded or others in parts
                let changes = true;
                label_while_changes: while (changes) {
                    changes = false;
                    for (let ip = 0; ip < parts.length; ip++) {
                        let p = parts[ip];


                        for (let ic = ip + 1; ic < parts.length; ic++) {
                            let c = parts[ic];

                            if (c === p) {
                                changes = true;
                                parts.splice(ic, 1);
                                continue label_while_changes;
                            }

                            if ((c.start_from[2] === p.start_from[2] || c.start_from[2] === this.flipDirection(p.start_from[2])) &&
                                (this.rangedOverlap(c.from[0], c.to[0], p.from[0], p.to[0]) && !(c.from[0] === c.to[0] || p.from[0] === p.to[0]))
                                || this.rangedOverlap(c.from[1], c.to[1], p.from[1], p.to[1]) && !(c.from[1] === c.to[1] || p.from[1] === p.to[1])) {
                                changes = true;

                                let calculated_overlap = this.calculateOverlap(c, p, [], []);
                                let _parts = calculated_overlap.parts;
                                let _insertion = calculated_overlap.insertion;

                                //ic is always greater than ip, so we can safely remove the part from the array.

                                if (c.from[0] === -99) {
                                    parts.splice(ic, 1);
                                } else if (c.from[0] === -101) {
                                    parts.splice(ic, 1);
                                    parts.push(...(_insertion[0]));
                                }

                                parts.splice(ip, 1);
                                if (_parts.length > 0) {
                                    parts.push(..._parts);
                                }

                                continue label_while_changes;
                            }
                        }

                        for (let ar of unfolded) {
                            for (let ic = 0; ic < ar.length; ic++) {
                                let c = ar[ic];

                                if ((c.start_from[2] === p.start_from[2] || c.start_from[2] === this.flipDirection(p.start_from[2])) &&
                                    (this.rangedOverlap(c.from[0], c.to[0], p.from[0], p.to[0]) && !(c.from[0] === c.to[0] || p.from[0] === p.to[0]))
                                    || this.rangedOverlap(c.from[1], c.to[1], p.from[1], p.to[1]) && !(c.from[1] === c.to[1] || p.from[1] === p.to[1])) {
                                    changes = true;
                                    let _parts: {
                                        from: [number, number],
                                        to: [number, number],
                                        change: number,
                                        start_from: [number, number, string]
                                    }[] = [];

                                    let _insertion: {
                                        from: [number, number],
                                        to: [number, number],
                                        change: number,
                                        start_from: [number, number, string]
                                    }[][] = [];

                                    let calculated_overlap = this.calculateOverlap(c, p, _parts, _insertion);
                                    _parts = calculated_overlap.parts;
                                    _insertion = calculated_overlap.insertion;

                                    if (c.from[0] === -99 /*&& c.from[1] === -99 && c.to[0] === -99 && c.to[1] === -99*/) {
                                        ar.splice(ic, 1);
                                    } else if (c.from[0] === -101) {
                                        ar.splice(ic, 1);
                                        ar.push(...(_insertion[0]));
                                    }

                                    parts.splice(ip, 1);
                                    if (_parts.length > 0) {
                                        parts.push(..._parts);
                                    }

                                    continue label_while_changes;
                                }
                            }
                        }
                    }
                }
            }

            record({Iteration: i, Change: changes[i], Changes: changes, Unfolded: unfolded}, "Iteration Middle -> unfoldCompact");

            //add insertions or delete c
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

            record({Iteration: i, Change: changes[i], Changes: changes, Unfolded: unfolded}, "Iteration Middle after Filter -> unfoldCompact");

            //add parts
            if (parts.length !== 0) {
                if (change[2] === 1) {
                    unfolded.push(parts);
                } else {
                    unfolded[unfolded.length - 1].push(...parts);
                }
            }
        }

        /*unfolded = unfolded.map(ar => ar.filter(
            c => !(c.from[0] === c.to[0] && c.from[1] === c.to[1])));*/

        record({Unfolded: unfolded}, "Finished -> unfoldCompact");
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
        return /*this.hasChanges()*/ this.has_changes && !this.is_playing;
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