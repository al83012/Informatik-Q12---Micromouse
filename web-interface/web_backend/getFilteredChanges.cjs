/**
 * getFilteredChanges
 * -------------------
 * Walks a path_tree from a given top node, collects every change bubbled up
 * through the subtree, resolves each change to the from/to coordinates of the
 * node it refers to, and returns a list of changes with no coordinate overlap
 * between them.
 *
 * Overlap = two changed nodes' straight-line segments (from -> to) sharing part
 * of the same line (colinear overlap; direction doesn't matter, A->B == B->A).
 * Where two segments overlap, the lower-priority one is trimmed down to its
 * non-overlapping remainder. Priority order (highest to lowest): 0, 1, -1.
 * Trimming can split a segment into 0, 1, or 2 output pieces, depending on
 * whether the overlap sits at an end or in the middle.
 *
 * Two lines that merely cross at a single point (not colinear) are NOT treated
 * as an overlap, since there's no length to cut out of either one.
 *
 * ASSUMPTIONS (flag these if they don't match your setup):
 * 1. A removed node's entry (change === -1) is still present in `tree`
 *    (e.g. only detached from its parent's children, not deleted) so its
 *    from/to can still be read. If removals actually delete the node from
 *    path_tree, from/to needs to be stored on the change record itself instead.
 * 2. Ties between two changes of EQUAL priority are resolved in favor of
 *    whichever was encountered first during the tree traversal (parents /
 *    earlier children before later ones).
 * 3. Segments can be any straight line (axis-aligned or diagonal), not just
 *    single-step grid edges.
 */

// ---- small 2D vector helpers -------------------------------------------------

const V = {
    sub: (a, b) => [a[0] - b[0], a[1] - b[1]],
    add: (a, b) => [a[0] + b[0], a[1] + b[1]],
    scale: (a, t) => [a[0] * t, a[1] * t],
    dot: (a, b) => a[0] * b[0] + a[1] * b[1],
    cross: (a, b) => a[0] * b[1] - a[1] * b[0],
};

const EPS = 1e-9;
const approxEq = (a, b, eps = EPS) => Math.abs(a - b) < eps;
const isZeroVec = (v) => approxEq(v[0], 0) && approxEq(v[1], 0);

/** True if segment P (pFrom->pTo) and segment Q (qFrom->qTo) lie on the same infinite line. */
function isColinear(pFrom, pTo, qFrom, qTo) {
    const dP = V.sub(pTo, pFrom);
    const dQ = V.sub(qTo, qFrom);

    if (isZeroVec(dP) && isZeroVec(dQ)) {
        return approxEq(pFrom[0], qFrom[0]) && approxEq(pFrom[1], qFrom[1]);
    }
    if (isZeroVec(dP)) {
        return approxEq(V.cross(dQ, V.sub(pFrom, qFrom)), 0); // P is a point on Q's line?
    }
    if (isZeroVec(dQ)) {
        return approxEq(V.cross(dP, V.sub(qFrom, pFrom)), 0); // Q is a point on P's line?
    }
    if (!approxEq(V.cross(dP, dQ), 0)) return false; // not even parallel
    return approxEq(V.cross(dP, V.sub(qFrom, pFrom)), 0);
}

/**
 * Removes the portion of segment P that overlaps with segment O, returning
 * 0, 1, or 2 remaining pieces of P (as {from, to}). If P and O aren't
 * colinear, P is returned unchanged.
 */
function subtractOverlap(pFrom, pTo, oFrom, oTo) {
    if (!isColinear(pFrom, pTo, oFrom, oTo)) {
        return [{ from: pFrom, to: pTo }];
    }

    const d = V.sub(pTo, pFrom);
    const lenSq = V.dot(d, d);

    // P is a degenerate point: fully removed if that point lies within O's range.
    if (approxEq(lenSq, 0)) {
        const dO = V.sub(oTo, oFrom);
        const lenSqO = V.dot(dO, dO);
        if (approxEq(lenSqO, 0)) {
            const same = approxEq(pFrom[0], oFrom[0]) && approxEq(pFrom[1], oFrom[1]);
            return same ? [] : [{ from: pFrom, to: pTo }];
        }
        const t = V.dot(V.sub(pFrom, oFrom), dO) / lenSqO;
        return t >= -EPS && t <= 1 + EPS ? [] : [{ from: pFrom, to: pTo }];
    }

    // Parametrize P as t=0 (pFrom) .. t=1 (pTo); express O's endpoints in that parametrization.
    const paramOf = (point) => V.dot(V.sub(point, pFrom), d) / lenSq;
    let t0 = paramOf(oFrom);
    let t1 = paramOf(oTo);
    if (t0 > t1) [t0, t1] = [t1, t0];

    const overlapStart = Math.max(0, t0);
    const overlapEnd = Math.min(1, t1);
    if (overlapStart > overlapEnd + EPS) {
        return [{ from: pFrom, to: pTo }]; // O doesn't actually fall within P's bounds
    }

    const pieces = [];
    if (overlapStart > EPS) {
        pieces.push({ from: pFrom, to: V.add(pFrom, V.scale(d, overlapStart)) });
    }
    if (overlapEnd < 1 - EPS) {
        pieces.push({ from: V.add(pFrom, V.scale(d, overlapEnd)), to: pTo });
    }
    return pieces;
}

/** Trims `pieces` against every already-accepted segment, splitting as needed. */
function trimAgainstAccepted(pieces, accepted) {
    let current = pieces;
    for (const acc of accepted) {
        if (current.length === 0) break;
        const next = [];
        for (const piece of current) {
            next.push(...subtractOverlap(piece.from, piece.to, acc.from, acc.to));
        }
        current = next;
    }
    return current;
}

/**
 * @param {string} topId - id of the node to start traversal from
 * @param {object} tree - the path_tree object (defaults to PathTree.path_tree if present)
 * @returns {{id: string, change: number, from: [number, number], to: [number, number]}[]}
 */
function getFilteredChanges(topId, tree = (typeof PathTree !== 'undefined' ? PathTree.path_tree : undefined)) {
    // 1. Gather every change in the subtree, top to bottom
    const allChanges = [];
    (function traverse(nodeId) {
        const node = tree[nodeId];
        if (!node) return;
        if (Array.isArray(node.changes)) {
            allChanges.push(...node.changes);
            node.changes = [];
        }
        if (Array.isArray(node.children)) node.children.forEach(traverse);
    })(topId);

    // 2. Resolve each change to the from/to of the node it refers to
    const segments = allChanges
        .map((c) => {
            const node = tree[c.id];
            if (!node) return null; // node data unavailable, can't place it geometrically
            return { id: c.id, change: c.change, from: [...node.from], to: [...node.to] };
        })
        .filter(Boolean);

    // 3. Sort by priority (0 > 1 > -1), stable on original traversal order for ties
    const rank = (change) => (change === 0 ? 0 : change === 1 ? 1 : 2);
    const ordered = segments
        .map((seg, idx) => ({ seg, idx }))
        .sort((a, b) => rank(a.seg.change) - rank(b.seg.change) || a.idx - b.idx)
        .map((x) => x.seg);

    // 4. Accept highest-priority segments first, trimming each against everything already accepted
    const accepted = [];
    for (const seg of ordered) {
        const remaining = trimAgainstAccepted([{ from: seg.from, to: seg.to }], accepted);
        for (const piece of remaining) {
            accepted.push({ id: seg.id, change: seg.change, from: piece.from, to: piece.to });
        }
    }

    return accepted;
}

module.exports = { getFilteredChanges };
