export class Animation {
    duration;
    finished = false;

    constructor(duration, object) {
        this.duration = duration;
        this.remaining_duration = duration;
        this.object = object;
    }

    execute() {};
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

export class AnimationHandler {
    animations = [];
    rep_animations = [];
    rep_durations = [];
    imm_animations = [];
    constructor() {}

    add(animation) {
        this.animations.push(animation);
    }

    addRepeating(animation) {
        this.rep_animations.push(animation);
        this.rep_durations.push(animation.duration);
    }

    addImmediate(animation) {
        this.imm_animations.push(animation);
    }

    nextFrame() {
        for (let i = 0; i < this.rep_animations.length; i++) {
            this.rep_animations[i].execute();
            if (this.rep_animations[i].finished) {
                this.rep_animations[i].finished = false;
                this.rep_animations[i].remaining_duration = this.rep_durations[i];
            }
        }

        let shift = 0;
        for (let i = 0; i < this.imm_animations.length; i++) {
            this.imm_animations[i-shift].execute();
            if (this.imm_animations[i-shift].finished) {
                shift++;
                this.imm_animations.splice(i-shift, 1);
            }
        }

        if (this.animations.length === 0) return;
        this.animations[0].execute();
        if (this.animations[0].finished) {
            this.animations.shift();
        }
    }
}

export class AnimGroup extends Animation {
    animations = [];

    constructor(delay) {
        super(-1); //ignore
        this.delay = delay;
        this.current_delay = delay;
    }

    add(animation) {
        this.animations.push(animation);
    }

    execute() {
        if (this.delay === -1) {
            if (this.animations.length === 0) {
                this.finished = true;
                return;
            }

            this.animations[0].execute();
            if (this.animations[0].finished) {
                this.animations.shift();
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
        let all_finished = (amount >= this.animations.length - 1);

        for (let i = 0; i < amount; i++) {
            if (!(this.animations[i].finished)) {
                this.animations[i].execute();
                all_finished = false;
            }
        }

        if (all_finished) {
            this.finished = true;
        }

        this.current_delay++;
    }
}