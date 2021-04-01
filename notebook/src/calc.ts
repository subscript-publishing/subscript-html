// // import * as _ from 'lodash';
import {MathfieldElement} from 'mathlive';

interface RelativePosition {
    position: "before" | "after",
    element: EntryElement,
}

function guid(): string {
    let s4 = () => {
        return Math.floor((1 + Math.random()) * 0x10000)
            .toString(16)
            .substring(1);
    }
    //return id of format 'aaaaaaaa'_'aaaa'_'aaaa'_'aaaa'_'aaaaaaaaaaaa'
    return 'I' + s4() + s4() + '_' + s4() + '_' + s4() + '_' + s4() + '_' + s4() + s4() + s4();
}


export class EntryElement extends HTMLElement {
    math_field: MathfieldElement;
    uid: string;
    mount: HTMLElement;
    constructor(mount: HTMLElement, config?: RelativePosition) {
        super();
        let header = document.createElement('header');
        let footer = document.createElement('footer');
        this.mount = mount;
        this.uid = guid();
        this.math_field = new MathfieldElement();
        this.setAttribute('field-wrapper', '');
        this.appendChild(header);
        this.appendChild(this.math_field);
        this.appendChild(footer);
        this.setAttribute('uid', this.uid);
        ///////////////////////////////////////////////////////////////////////
        // ATTATCH TO DOM
        ///////////////////////////////////////////////////////////////////////
        if (config) {
            console.assert(
                config.position == 'before' ||
                config.position == 'after'
            );
            if (config.position == 'before') {
                config.element.before(this);
            }
            if (config.position == 'after') {
                config.element.after(this);
            }
        }
        console.log(this);
        ///////////////////////////////////////////////////////////////////////
        // EVENTS
        ///////////////////////////////////////////////////////////////////////
        let focus_out = (event: any) => {
            let delete_me = false;
            let is_direction = (ty: 'forward' | 'backward'): boolean => {
                if (event instanceof String) {
                    return event == ty;
                }
                if (event.detail && event.detail.direction) {
                    return event.detail.direction == ty;
                }
                return false;
            };
            if (is_direction('backward')) {
                console.log("backward this", this);
                let sibling = this.previousSibling as EntryElement;
                if (this.math_field.value == '') {
                    delete_me = true;
                }
                if (sibling && sibling != undefined) {
                    (sibling as EntryElement).math_field.focus();
                } else {
                    let _ = new EntryElement(this.mount, {
                        position: 'before',
                        element: this,
                    });
                }
            }
            if (is_direction('forward')) {
                let sibling = this.nextSibling as EntryElement;
                if (sibling && sibling != undefined) {
                    (sibling).math_field.focus();
                } else if (this.math_field.value != '') {
                    let _ = new EntryElement(this.mount, {
                        position: 'after',
                        element: this,
                    });
                }
            }
            if (delete_me) {
                this.math_field.remove();
            }
        };
        this.math_field.focus();
        this.setAttribute("tabindex", '0');
        this.math_field.addEventListener('keypress', (event: any) => {
            if (event.key === 'Enter') {
                focus_out('forward');
            }
        });
        this.math_field.addEventListener('focus-out', (ev: any) => {
            focus_out(ev);
        });
        this.math_field.addEventListener('unmount', (event: any) => {
            this.remove();
        });
    }
    remove() {
        this.math_field.remove();
        return super.remove();
    }
}


window.customElements.define('x-entry', EntryElement);
function start() {
    document.body.setAttribute('calc', '');
    let main = document.createElement('main');
    main.setAttribute('tabindex', '-1');
    main.setAttribute('autofocus', '');
    main.setAttribute('calc', '');
    // let main = document.querySelector('main');
    function append() {
        let x = new EntryElement(main);
        main.appendChild(x);
    }
    append();
    main.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
            append();
        }
    });
}

// window.onload = () => {
//     start();
// }; 