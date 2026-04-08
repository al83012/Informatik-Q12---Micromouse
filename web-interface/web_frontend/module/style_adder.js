export class StyleAdder {
    static disableForClass() {
        const elem = document.getElementsByClassName("unselectable");
        for (let i = 0; i < elem.length; i++) {
            StyleAdder.disableSelection(elem.item(i));
        }
    }

    static disableSelection(element) {
        if (typeof element.onselectstart != 'undefined') {
            element.onselectstart = function () {
                return false;
            };
        } else if (typeof element.style.MozUserSelect != 'undefined') {
            element.style.MozUserSelect = 'none';
        } else {
            element.onmousedown = function () {
                return false;
            };
        }
    }
}