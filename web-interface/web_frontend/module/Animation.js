function hexToRgb(hex) {
    var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : null;
}

function componentToHex(c) {
    var hex = c.toString(16);
    return hex.length == 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
    return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

export class Animation {
    duration;
    finished = false;

    constructor(duration, object, ignore=false) {
        if (ignore) return;

        if (object !== null) {
            this.duration = duration;
            this.remaining_duration = duration;
            this.object = object;
            this.has_animation = object?.has_animation ?? false;
        } else {
            console.log("[ERROR] -> Animation: object is null");
            throw new Error("Animation: object is null");
        }
    }

    execute() {};
    reset() {
        this.finished = false;
        this.remaining_duration = this.duration;
    }
}

export class AnimFadeOut extends Animation {
    constructor(duration, object, on_finished) {
        super(duration, object);
        this.on_finished = on_finished;
    }

    execute() {
        if(this.finished) {
            return;
        }

        this.object.style.opacity = (this.remaining_duration / this.duration) + "";

        this.remaining_duration--;
        if (this.remaining_duration === 0) {
            this.object.style.opacity = 0;
            this.finished = true;
            this.on_finished(this.object);
        }
    };
}

export class AnimFadeIn extends Animation {
    constructor(duration, object, on_start) {
        super(duration, object);
        this.on_start = on_start;
    }

    execute() {
        if (this.remaining_duration === this.duration) {
            this.on_start(this.object);
        }

        if(this.finished) {
            return;
        }

        this.object.style.opacity = ((this.remaining_duration / this.duration)-1)*(-1) + "";

        this.remaining_duration--;
        if (this.remaining_duration === 0) {
            this.object.style.opacity = 1;
            this.finished = true;
        }
    };
}

export class AnimBorderOuter extends Animation {
    constructor(duration, object, factor) {
        super(duration, object);
        this.start_size = getComputedStyle(this.object, null).getPropertyValue("border-width");
        this.size = Number.parseFloat(getComputedStyle(this.object, null).getPropertyValue("border-width").replaceAll("px", ""));
        this.factor = factor;
    }

    execute() {
        if (this.remaining_duration >= this.duration/2) {
            this.object.style.borderWidth = Math.round(this.size - this.factor / this.duration) + "px";
            this.size -= this.factor / this.duration;
        } else if (this.remaining_duration > 0) {
            this.object.style.borderWidth = Math.round(this.size + this.factor / this.duration) + "px";
            this.size += this.factor / this.duration;
        } else {
            this.object.style.borderWidth = this.start_size;
            this.finished = true;
        }
        this.remaining_duration--;
    }
}

export class AnimBorderInner extends Animation {
    constructor(duration, object, factor_start, factor_end, value_complete, value_start) {
        super(duration, object);
        if (typeof value_start === "undefined" || value_start === undefined) {
            this.start_border_size = Number.parseFloat(getComputedStyle(this.object, null).
                getPropertyValue("border-width").
                replaceAll("px", ""));
            this.start_width = Number.parseFloat(getComputedStyle(this.object, null).
                getPropertyValue("width").
                replaceAll("px", ""));
            this.start_height = Number.parseFloat(getComputedStyle(this.object, null).
                getPropertyValue("height").
                replaceAll("px", ""));
        } else {
            this.start_border_size = value_start[0];
            this.start_width = value_start[1];
            this.start_height = value_start[2];
        }
        this.border_size = this.start_border_size
        this.width = this.start_width;
        this.height = this.start_height;

        this.factor_start = factor_start;
        this.factor_end = factor_end;
        this.value_complete = value_complete;
    }

    execute() {
        if (this.finished) {
            return;
        }

        this.remaining_duration--;

        if (this.remaining_duration >= this.duration/2) {
            this.border_size = this.border_size + this.factor_start / (this.duration/2);
        } else if (this.remaining_duration > 0) {
            this.border_size = this.border_size - this.factor_end / (this.duration/2);
        } else if (this.remaining_duration === 0) {
            if (this.value_complete !== -1) {
                this.border_size = this.value_complete;
            }
        }

        this.width = (this.start_width + this.start_border_size * 2) - Math.round(this.border_size) * 2;
        this.height = (this.start_height + this.start_border_size * 2) - Math.round(this.border_size) * 2;

        this.object.style.borderWidth = Math.round(this.border_size) + "px";
        this.object.style.width = Math.round(this.width) + "px";
        this.object.style.height = Math.round(this.height) + "px";

        if (this.remaining_duration <= 0) {
            this.finished = true;
        }
    }
}

export class AnimSlideColor extends Animation {
    constructor(duration, object, color_start, color_end) {
        super(duration, object);
        this.color_start = color_start;
        this.color_end = color_end;
    }

    execute() {
        if (this.finished) {
            return;
        }

        let midpoint = ((this.remaining_duration/this.duration)-1)*(-1)*200;

        this.object.style.backgroundImage = `linear-gradient(to right, ${this.color_end} 0% ${midpoint}%, ${this.color_start} ${midpoint}% 500%)`;

        this.remaining_duration--;

        if (this.remaining_duration <= 0) {
            this.finished = true;
            this.object.style.backgroundImage = `linear-gradient(to right, ${this.color_end} 0% 500%, ${this.color_start} 500% 500%)`
        }
    }
}

export class AnimBackgroundColor extends Animation {
    constructor(duration, object, color_start, color_end, on_finished = (obj) => {}, on_start = (obj) => {}) {
        super(duration, object);

        this.color_start = hexToRgb(color_start);
        this.color_end = hexToRgb(color_end);

        this.dr = this.color_end.r - this.color_start.r;
        this.dg = this.color_end.g - this.color_start.g;
        this.db = this.color_end.b - this.color_start.b;

        this.exec = on_finished;
        this.on_start = on_start;
    }

    execute() {
        if (this.finished) {
            return;
        }
        if (this.remaining_duration === this.duration) {
            this.on_start(this.object);
        }

        this.color_start.r += Math.round(this.dr/this.duration);
        this.color_start.g += Math.round(this.dg/this.duration);
        this.color_start.b += Math.round(this.db/this.duration);

        this.object.style.backgroundColor = rgbToHex(this.color_start.r, this.color_start.g, this.color_start.b);

        this.remaining_duration--;

        if (this.remaining_duration <= 0) {
            this.finished = true;
            this.exec(this.object);
            this.object.style.backgroundColor = rgbToHex(this.color_end.r, this.color_end.g, this.color_end.b);
        }
    }
}

export class AnimBorderColor extends Animation {
    constructor(duration, object, color_start, color_end) {
        super(duration, object);

        if (typeof color_end === "string") {
            this.type = "shift";
            this.color_end = parseInt(this.colourNameToHex(color_end).replaceAll("#", ""), 16);
            this.color_start = parseInt(this.colourNameToHex(color_start).replaceAll("#", ""), 16);
            this.difference = this.color_start - this.color_end;
            if (this.difference <= 0) {this.difference = this.difference * (-1);}
        } else {
            this.type = "return";
            this.amount = color_end;
            this.color_start = parseInt(this.colourNameToHex(color_start).replaceAll("#", ""), 16);
        }


    }

    execute() {
        if (this.finished) {return;}

        if (this.type === "shift") {
            if (this.color_start < this.color_end) {
                this.color_start += Math.max(Math.round(this.difference/this.duration), 1);
                console.log(this.color_start)
                this.object.style.borderColor = "#" + this.color_start.toString(16);
                console.log(this.object.style.borderColor);
            }

            if (this.remaining_duration <= 0) {
                this.finished = true;
                this.object.style.borderColor = this.color_end.toString(16);
            }
        }

        this.remaining_duration--;
    }

    colourNameToHex(colour) {
        var colours = {"aliceblue":"#f0f8ff","antiquewhite":"#faebd7","aqua":"#00ffff","aquamarine":"#7fffd4","azure":"#f0ffff",
            "beige":"#f5f5dc","bisque":"#ffe4c4","black":"#000000","blanchedalmond":"#ffebcd","blue":"#0000ff","blueviolet":"#8a2be2","brown":"#a52a2a","burlywood":"#deb887",
            "cadetblue":"#5f9ea0","chartreuse":"#7fff00","chocolate":"#d2691e","coral":"#ff7f50","cornflowerblue":"#6495ed","cornsilk":"#fff8dc","crimson":"#dc143c","cyan":"#00ffff",
            "darkblue":"#00008b","darkcyan":"#008b8b","darkgoldenrod":"#b8860b","darkgray":"#a9a9a9","darkgreen":"#006400","darkkhaki":"#bdb76b","darkmagenta":"#8b008b","darkolivegreen":"#556b2f",
            "darkorange":"#ff8c00","darkorchid":"#9932cc","darkred":"#8b0000","darksalmon":"#e9967a","darkseagreen":"#8fbc8f","darkslateblue":"#483d8b","darkslategray":"#2f4f4f","darkturquoise":"#00ced1",
            "darkviolet":"#9400d3","deeppink":"#ff1493","deepskyblue":"#00bfff","dimgray":"#696969","dodgerblue":"#1e90ff",
            "firebrick":"#b22222","floralwhite":"#fffaf0","forestgreen":"#228b22","fuchsia":"#ff00ff",
            "gainsboro":"#dcdcdc","ghostwhite":"#f8f8ff","gold":"#ffd700","goldenrod":"#daa520","gray":"#808080","green":"#008000","greenyellow":"#adff2f",
            "honeydew":"#f0fff0","hotpink":"#ff69b4",
            "indianred ":"#cd5c5c","indigo":"#4b0082","ivory":"#fffff0","khaki":"#f0e68c",
            "lavender":"#e6e6fa","lavenderblush":"#fff0f5","lawngreen":"#7cfc00","lemonchiffon":"#fffacd","lightblue":"#add8e6","lightcoral":"#f08080","lightcyan":"#e0ffff","lightgoldenrodyellow":"#fafad2",
            "lightgrey":"#d3d3d3","lightgreen":"#90ee90","lightpink":"#ffb6c1","lightsalmon":"#ffa07a","lightseagreen":"#20b2aa","lightskyblue":"#87cefa","lightslategray":"#778899","lightsteelblue":"#b0c4de",
            "lightyellow":"#ffffe0","lime":"#00ff00","limegreen":"#32cd32","linen":"#faf0e6",
            "magenta":"#ff00ff","maroon":"#800000","mediumaquamarine":"#66cdaa","mediumblue":"#0000cd","mediumorchid":"#ba55d3","mediumpurple":"#9370d8","mediumseagreen":"#3cb371","mediumslateblue":"#7b68ee",
            "mediumspringgreen":"#00fa9a","mediumturquoise":"#48d1cc","mediumvioletred":"#c71585","midnightblue":"#191970","mintcream":"#f5fffa","mistyrose":"#ffe4e1","moccasin":"#ffe4b5",
            "navajowhite":"#ffdead","navy":"#000080",
            "oldlace":"#fdf5e6","olive":"#808000","olivedrab":"#6b8e23","orange":"#ffa500","orangered":"#ff4500","orchid":"#da70d6",
            "palegoldenrod":"#eee8aa","palegreen":"#98fb98","paleturquoise":"#afeeee","palevioletred":"#d87093","papayawhip":"#ffefd5","peachpuff":"#ffdab9","peru":"#cd853f","pink":"#ffc0cb","plum":"#dda0dd","powderblue":"#b0e0e6","purple":"#800080",
            "rebeccapurple":"#663399","red":"#ff0000","rosybrown":"#bc8f8f","royalblue":"#4169e1",
            "saddlebrown":"#8b4513","salmon":"#fa8072","sandybrown":"#f4a460","seagreen":"#2e8b57","seashell":"#fff5ee","sienna":"#a0522d","silver":"#c0c0c0","skyblue":"#87ceeb","slateblue":"#6a5acd","slategray":"#708090","snow":"#fffafa","springgreen":"#00ff7f","steelblue":"#4682b4",
            "tan":"#d2b48c","teal":"#008080","thistle":"#d8bfd8","tomato":"#ff6347","turquoise":"#40e0d0",
            "violet":"#ee82ee",
            "wheat":"#f5deb3","white":"#ffffff","whitesmoke":"#f5f5f5",
            "yellow":"#ffff00","yellowgreen":"#9acd32"};

        if (typeof colours[colour.toLowerCase()] != 'undefined')
            return colours[colour.toLowerCase()];

        return "#ffffff";
    }
}

export class AnimCssChange extends Animation {
    constructor(duration, object, items, replacement) {
        if (object !== null) {
            super(duration, object);
            if (this.has_animation) {
                console.log("Possible collision with existing AnimCssChange: " + (object?.css_change_anim_repl ?? "no-anim"));
            } else {
                object.has_animation = true;
                object.css_change_anim_repl = replacement;
            }

            this.log_name = object.className;
            //console.log(this.log_name);
            this.items = items;
            this.replacement = replacement;
        } else {
            super(duration, null);
            this.has_animation = true;
        }
    }

    execute() {
        if (this.finished) {
            return;
        }

        try {
            if (this.object !== null) {
                if (this.duration === this.remaining_duration) {
                    for (let i = 0; i < this.items.length; i++) {
                        if (this.object.className.includes(" " + this.items[i])) {
                            this.object.className = this.object.className.replace(" " + this.items[i], " " + this.replacement);
                            break;
                        }
                    }
                }
            }
        } catch (e) {
            console.log("Error in AnimCssChange of tile: " + this.log_name);
        }

        this.remaining_duration--;

        if (this.remaining_duration < 0) {
            this.finished = true;
        }
    }
}

export class AnimMoveMultiples extends Animation {
    constructor(duration, object, multiples, x_factor_start, x_factor_end, y_factor_start, y_factor_end) {
        super(duration, object);
        this.multiples = multiples
        this.x_factor_start = x_factor_start;
        this.x_factor_end = x_factor_end;
        this.y_factor_start = y_factor_start;
        this.y_factor_end = y_factor_end;
        this.cur_factor_x = x_factor_start;
        this.cur_factor_y = y_factor_start;
    }

    execute() {
        if (this.finished) {
            return;
        }
        if (this.remaining_duration === this.duration) {
            /*this.object.style = this.object.style.replace(new RegExp("\\/\\*AnimMoveMultiples_start\\*\\/.*\\/\\*AnimMoveMultiples_end\\*\\/"), "")
                + "/*AnimMoveMultiples_start*//*AnimMoveMultiples_end*/           /*";*/
        }

        this.cur_factor_x = this.x_factor_start + (this.x_factor_end - this.x_factor_start) * (1-(this.remaining_duration / this.duration));
        this.cur_factor_y = this.y_factor_start + (this.y_factor_end - this.y_factor_start) * (1-(this.remaining_duration / this.duration));

        this.object.style.left = "calc(" + this.multiples + "px*" + this.cur_factor_x + ")";
        this.object.style.top = "calc(" + this.multiples + "px*" + this.cur_factor_y + ")";

       // this.object.style = this.object.style.replace(new RegExp("\\/\\*AnimMoveMultiples_start\\*\\/.*\\/\\*AnimMoveMultiples_end\\*\\/"),
       //     "/*AnimMoveMultiples_start*/left: calc(" + this.multiples + "px*" + this.cur_factor_x + "); " +
       //     "right: calc(" + this.multiples + "px*" + this.cur_factor_y + ");/*AnimMoveMultiples_end*/")

        this.remaining_duration--;

        if (this.remaining_duration < 0) {
            this.finished = true;
            this.object.style.left = "calc(" + this.multiples + "px*" + this.x_factor_end + ")";
            this.object.style.top = "calc(" + this.multiples + "px*" + this.y_factor_end + ")";
            //this.object.style = this.object.style.replace(new RegExp("\\/\\*AnimMoveMultiples_start\\*\\/.*\\/\\*AnimMoveMultiples_end\\*\\/"),
            //    "/*AnimMoveMultiples_start*/left: calc(" + this.multiples + "px*" + this.x_factor_end + "); " +
            //    "right: calc(" + this.multiples + "px*" + this.y_factor_end + ");/*AnimMoveMultiples_end*/");
        }
    }
}

export class AnimRotate extends Animation {
    constructor(duration, object, angle_start, angle_end) {
        super(duration, object);

        this.angle_start = angle_start;
        this.angle_end = angle_end;

        if (typeof angle_start === "string") {
            switch (angle_start) {
                case "n":
                    this.angle_start = 0;
                    break;
                case "s":
                    this.angle_start = 180;
                    break;
                case "w":
                    this.angle_start = 270;
                    break;
                case "e":
                    this.angle_start = 90;
                    break;
            }
        }

        if (typeof angle_end === "string") {
            switch (angle_end) {
                case "n":
                    this.angle_end = 0;
                    break;
                case "s":
                    this.angle_end = 180;
                    break;
                case "w":
                    this.angle_end = 270;
                    break;
                case "e":
                    this.angle_end = 90;
                    break;
            }
        }

    }

    execute() {
        if (this.finished) {
            return;
        }

        this.object.style.transform = "rotate(" + ((this.angle_end-this.angle_start)*(1-(this.remaining_duration / this.duration))) + "deg)";

        this.remaining_duration--;

        if (this.remaining_duration < 0) {
            this.object.style.transform = "rotate(" + this.angle_end + "deg)";
        }
    }
}

export class AnimationHandler {
    animations = [];
    rep_animations = [];
    //rep_durations = [];
    imm_animations = [];
    constructor() {}

    add(animation) {
        this.animations.push(animation);
    }

    addRepeating(animation, id) {
        if (!this.rep_animations.includes(id)) {
            animation.rep_remove = false;
            this.rep_animations[id] = animation;
            this.rep_animations.push(id);
        }
        //this.rep_durations.push(animation.duration);
    }

    removeRepeating(id) {
        if (this.rep_animations.includes(id)) {
            this.rep_animations[id].rep_remove = true;
        }
    }

    addImmediate(animation) {
        this.imm_animations.push(animation);
    }

    nextFrame() {
        /*for (let i = 0; i < this.rep_animations.length; i++) {
            this.rep_animations[i].execute();
            if (this.rep_animations[i].finished) {
                this.rep_animations[i].reset();
                /*this.rep_animations[i].finished = false;
                this.rep_animations[i].remaining_duration = this.rep_durations[i];*/
        //    }
        //}
        for (let id of this.rep_animations) {
            this.rep_animations[id].execute();
            if (this.rep_animations[id].finished) {
                this.rep_animations[id].reset();
                if (this.rep_animations[id].rep_remove) {
                    delete this.rep_animations[id];
                    this.rep_animations.splice(this.rep_animations.indexOf(id), 1);
                }
            }
        }

        //let shift = 0;
        let remove = [];
        for (let i = 0; i < this.imm_animations.length; i++) {
            //this.imm_animations[i-shift].execute();
            this.imm_animations[i].execute();
            //if (this.imm_animations[i-shift].finished) {
            if (this.imm_animations[i].finished) {
                //shift++;
                //this.imm_animations.splice(i-shift, 1);
                remove.push(i);
            }
        }
        let new_imm_animations = [];
        for (let i = 0; i < this.imm_animations.length; i++) {
            if (!(remove.includes(i))) {
                new_imm_animations.push(this.imm_animations[i]);
            }
        }
        this.imm_animations = new_imm_animations;


        if (this.animations.length === 0) return;
        this.animations[0].execute();
        if (this.animations[0].finished) {
            this.animations.shift();
        }
    }
}

export class AnimGroup extends Animation {
    animations = [];
    rem_animations = [];

    constructor(delay, action) {
        super(-1, null, true); //ignore
        this.delay = delay;
        this.current_delay = delay;
        if (action !== undefined) {
            this.action = action;
        }
    }

    add(animation) {
        this.animations.push(animation);
    }

    reset() {
        for (let anim of this.rem_animations) {
            this.animations.push(anim);
        }
        this.rem_animations = [];

        for (let i = 0; i < this.animations.length; i++) {
            this.animations[i].reset();
        }

        this.current_delay = 0;

        super.reset();
    }

    execute() {
        if (this.finished) {
            return;
        }

        if (this.delay === -1) {
            if (this.animations.length === 0) {
                this.finished = true;
                return;
            }

            this.animations[0].execute();
            if (this.animations[0].finished) {
                this.rem_animations.push(this.animations.shift());
            }

            return;
        } else if (this.delay === 0) {
            let all_finished = true;
            for (let i = 0; i < this.animations.length; i++) {
                if (!this.animations[i].finished) {
                    this.animations[i].execute();
                    all_finished = false;
                }
            }
            if (all_finished) {this.finished = true;}
            return;
        }

        let amount = Math.min(Math.floor(this.current_delay/this.delay), this.animations.length);
        let all_finished = (amount >= this.animations.length);

        for (let i = 0; i < amount; i++) {
            if (!(this.animations[i].finished)) {
                this.animations[i].execute();
                all_finished = false;
            }
        }

        if (all_finished) {
            this.finished = true;
            if (this.action !== undefined) {
                this.action();
            }
        }

        this.current_delay++;
    }
}

export function generatePathAnimGroup(path_in/*: int[][]*/, tiles, ignore_group, fnc) { //tiles are all arms tied to respective coords
    const anim_time = 2;
    let complete_group = -1, complete_time = 0;
    for (let doub = 0; doub < 2; doub++) {
        let path = path_in.map(inner => [...inner]);
        let c_group = new AnimGroup(0);
        let was_change = false;
        let changed_group;
        let changed_group_time = 0;

        let adder_group = new AnimGroup(0);
        let adder_group_time_max = 0;

        for (let i = path.length - 1; i >= 0; i--) {
            let part = structuredClone(path[i]);
            let is_group = (part.pop() === 0); //0: no ; 1: yes
            let type = part.pop() //0: same ; -1: remove ; +1: add

            let duration = 0; //add 5 to the end to make it seamless
            for (let j = 0; j < part.length; j += 2) {
                console.log("tiles: ");
                console.log(tiles);
                console.log("J: " + j);
                console.log("length: " + tiles[[part[j], part[j + 1]]].length);
                console.log("coords: " + part[j] + ", " + part[j + 1]);
                duration += tiles[[part[j], part[j + 1]]].length * anim_time;
            }
            duration /= 3;
            duration -= anim_time; //remove the last few child_times
            complete_time += duration;

            /*console.log("type:" + type)
            console.log("duration:" + duration)
            console.log(part)*/

            let group = new AnimGroup(anim_time / 3);
            let n_group;

            switch (type) {
                case 0:
                    for (let j = 0; j < part.length; j += 2) { //loop trough all coords in +2 jumps
                        for (let k = 0; k < tiles[[part[j], part[j + 1]]].length; k++) {//for all arms in tiles for the coords
                            group.add(new AnimCssChange(anim_time, tiles[[part[j], part[j + 1]]][k],
                                (doub === 0 ? ["on", "repl"] : ["highlight", "repl"]),
                                (doub === 0 ? "highlight" : "on")));
                            //tiles[[part[j], part[j+1]]][k].style.opacity = 0;
                            //FIXED: Bug where the last for animations are created but not executed
                            //fix: cant be displayed on page load, only with a timeout of 1000
                        }
                    }
                    /*n_group = new AnimGroup(duration);
                    n_group.add(group);
                    n_group.add(c_group);
                    c_group = n_group;*/
                    break;
                case 1:
                    for (let j = 0; j < part.length; j += 2) { //loop trough all coords in +2 jumps
                        for (let k = 0; k < tiles[[part[j], part[j + 1]]].length; k++) {//for all arms in tiles for the coords
                            group.add(new AnimCssChange(anim_time, tiles[[part[j], part[j + 1]]][k],
                                (doub === 0 ? ["on", "repl"] : ["add", "repl"]),
                                (doub === 0 ? "add" : "on")));
                        }
                    }
                    /*if (!was_change) {
                        changed_group = group;
                        changed_group_time = duration;
                        was_change = true;
                    } else {
                        let adder_group = new AnimGroup(0);
                        adder_group.add(group);
                        adder_group.add(changed_group);
                        n_group = new AnimGroup(Math.max(changed_group_time, duration));
                        n_group.add(adder_group);
                        n_group.add(c_group);
                        c_group = n_group;
                        was_change = false;
                    }*/
                    break;
                case -1:
                    for (let j = 0; j < part.length; j += 2) { //loop trough all coords in +2 jumps
                        for (let k = 0; k < tiles[[part[j], part[j + 1]]].length; k++) {//for all arms in tiles for the coords
                            group.add(new AnimCssChange(anim_time, tiles[[part[j], part[j + 1]]][k],
                                (doub === 0 ? ["on", "repl"] : ["remove", "repl"]),
                                (doub === 0 ? "remove" : "repl")));
                        }
                    }
                    /*if (!was_change) {
                        changed_group = group;
                        changed_group_time = duration;
                        was_change = true;
                    } else {
                        let adder_group = new AnimGroup(0);
                        adder_group.add(group);
                        adder_group.add(changed_group);
                        n_group = new AnimGroup(Math.max(changed_group_time, duration));
                        n_group.add(adder_group);
                        n_group.add(c_group);
                        c_group = n_group;
                        was_change = false;
                    }*/
                    break;
            }

            adder_group_time_max = Math.max(adder_group_time_max, duration);
            adder_group.add(group);

            if (!is_group) {
                n_group = new AnimGroup(adder_group_time_max);
                n_group.add(adder_group);
                n_group.add(c_group);
                c_group = n_group;
                adder_group = new AnimGroup(0);
            }


        }

        if (was_change) {
            let n_group = new AnimGroup(changed_group_time);
            n_group.add(changed_group);
            n_group.add(c_group);
            c_group = n_group;
        }

        if (complete_group === -1) {
            complete_group = new AnimGroup((complete_time-30 < 10 ? complete_time : complete_time-20), fnc);
        }
        complete_group.add(c_group);
    }

    return complete_group;
}