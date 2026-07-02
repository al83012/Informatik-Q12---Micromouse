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
            }[]
        }
    } = {};

    static top_node: string = "-1";

    static addNode(from: [number, number], to: [number, number], id: string, parent: string) {
        if (this.top_node === "-1") {
            this.path_tree[parent] = {
                from: [0, 0],
                to: [0, 0],
                playing: false,
                children: [],
                parent: parent,
                changes: []
            };
            this.top_node = id;
        }

        this.path_tree[id] = {
            from: from,
            to: to,
            playing: false,
            children: [],
            parent: parent,
            changes: []
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
        const change_index = this.path_tree[parent_id].changes.indexOf({id: id, change: 1}, 0);
        if (change_index > -1) {
            this.path_tree[parent_id].changes.splice(change_index, 1);
            return;
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

    private static getRelevants(node_id: string): { id: string; change: number }[] {return undefined;}

    static getChanges(): [number, number, number][] {
        let changes: [number, number, number][] = [];



        return undefined;
    }
}