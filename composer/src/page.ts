// import init from 'mathjax-full';
import * as katex from 'katex';


///////////////////////////////////////////////////////////////////////////////
// UTILS
///////////////////////////////////////////////////////////////////////////////


function guid(): string {
    let s4 = () => {
        return Math.floor((1 + Math.random()) * 0x10000)
        .toString(16)
        .substring(1);
    }
    //return id of format 'aaaaaaaa'_'aaaa'_'aaaa'_'aaaa'_'aaaaaaaaaaaa'
    return '_' + s4() + s4() + '_' + s4() + '_' + s4() + '_' + s4() + '_' + s4() + s4() + s4();
}

///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////


class PageToolkitElement extends HTMLCanvasElement {
    uid: string;
    constructor() {
        super();
        // this.setAttribute('is', 'clock-canvas');
        // this.uid = guid();
        // this.setAttribute('uid', this.uid);
        // let ctx = this.getContext("2d");
    }
}

///////////////////////////////////////////////////////////////////////////////
// CONTENT
///////////////////////////////////////////////////////////////////////////////

class InlineLatexElement extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.setAttribute('latex', 'inline');
        let observer = new MutationObserver((mutations) => {
            let text_entries: Array<string> = [];
            let finalize = false;
            mutations.forEach(function(mutation) {
                console.assert(mutation.type === 'childList');
                finalize = true;
                mutation.addedNodes.forEach((child) => {
                    let txt = child.textContent.trim();
                    text_entries.push(txt);
                });
            });
            if (finalize) {
                let txt = text_entries.join('');
                let result = katex.renderToString(txt, {
                    displayMode: false,
                });
                observer.disconnect();
                this.innerHTML = result;
            }
        });
        observer.observe(this, {childList: true});
    }
}
class BlockLatexElement extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.setAttribute('latex', 'block');
        let observer = new MutationObserver((mutations) => {
            let text_entries: Array<string> = [];
            let finalize = false;
            mutations.forEach(function(mutation) {
                console.assert(mutation.type === 'childList');
                finalize = true;
                mutation.addedNodes.forEach((child) => {
                    let txt = child.textContent.trim();
                    text_entries.push(txt);
                });
            });
            if (finalize) {
                let txt = text_entries.join('');
                let result = katex.renderToString(txt, {
                    displayMode: true,
                });
                observer.disconnect();
                this.innerHTML = result;
            }
        });
        observer.observe(this, {childList: true});
    }
}
class EquationElement extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.setAttribute('latex', 'block');
        let observer = new MutationObserver((mutations) => {
            let text_entries: Array<string> = [];
            let finalize = false;
            mutations.forEach(function(mutation) {
                console.assert(mutation.type === 'childList');
                finalize = true;
                mutation.addedNodes.forEach((child) => {
                    let txt = child.textContent.trim();
                    text_entries.push(txt);
                });
            });
            if (finalize) {
                observer.disconnect();
                let txt = `\\begin{equation}\\begin{split}${text_entries.join('')}\\end{split}\\end{equation}`;
                try {
                    let result = katex.renderToString(txt, {
                        displayMode: true,
                        trust: true,
                        strict: false,
                    });
                    this.innerHTML = result;
                } catch(err) {
                    this.innerText = err;
                }
            }
        });
        observer.observe(this, {childList: true});
    }
}

///////////////////////////////////////////////////////////////////////////////
// PAGE ENTRY
///////////////////////////////////////////////////////////////////////////////


class NoteElement extends HTMLElement {
    uid: string;
    constructor() {
        super();
        this.uid = guid();
        // let content = document.createElement('p');
        // content.innerText = 'Macaroon danish biscuit apple pie. Gingerbread cake halvah wafer caramels. Soufflé gingerbread wafer powder. Cake bonbon jelly beans pie chupa chups candy canes biscuit tiramisu biscuit. Gummi bears donut cotton candy chocolate bar sugar plum jelly beans cake sugar plum. Lemon drops croissant croissant jelly soufflé biscuit halvah. Candy canes gummies lemon drops chupa chups cake candy soufflé chocolate cake oat cake. Sweet ice cream jelly beans. Icing cake chupa chups brownie. Oat cake bonbon sweet powder dragée. Gummies lollipop carrot cake caramels soufflé ice cream dessert gingerbread. Jelly ice cream donut jujubes dragée liquorice candy powder. Donut jelly ice cream biscuit sugar plum. Jelly beans chocolate cake cake donut halvah toffee marzipan lemon drops carrot cake.';
        // this.appendChild(content);
    }
    connectedCallback() {
        this.setAttribute('uid', this.uid);
        let observer = new MutationObserver(function(mutations) {
            //Detect child insertion
            mutations.forEach(function(mutation) {
                if (mutation.addedNodes.length) {
                    // console.info('Node added: ', mutation.addedNodes[0])
                }
            })
        });
        observer.observe(this, { childList: true });
    }
}

///////////////////////////////////////////////////////////////////////////////
// PAGE LAYOUT
///////////////////////////////////////////////////////////////////////////////


class LayoutElement extends HTMLElement {
    uid: string;
    level: number;
    extend: HTMLElement;
    constructor() {
        super();
        this.uid = guid();
        this.level = 1;
        this.extend = null;
    }
    init_extend() {
        if (!this.extend) {
            this.extend = document.createElement('x-note');
            this.extend.setAttribute('boxed', '');
            this.extend.setAttribute('extend', '');
            this.extend.innerHTML = `<p>+</p>`;
            this.appendChild(this.extend);
        }
    }
    init_heading(original: Element) {
        let tag_name = original.tagName.toLocaleLowerCase();
        let title_text = original.cloneNode(true);
        let header = document.createElement('header');
        header.setAttribute('tag', tag_name);
        let append_btn = (parent: Element, ty: 'top' | 'mid' | 'bot') => {
            let btn_node = document.createElement('button');
            btn_node.innerText = "+";
            parent.appendChild(btn_node);
        };
        // ASIDE
        let aside = document.createElement('aside');
        append_btn(aside, 'top');
        append_btn(aside, 'mid');
        append_btn(aside, 'bot');
        // TITLE
        let title_wrapper = document.createElement('div');
        title_wrapper.appendChild(title_text);
        title_wrapper.setAttribute('title', '');
        // INSERT
        header.appendChild(aside);
        header.appendChild(title_wrapper);
        
        // DONE
        this.replaceChild(
            header,
            original
        );
    }
    connectedCallback() {
        this.setAttribute('uid', this.uid);
        this.init_extend();
        let observer = new MutationObserver((mutations) => {
            //Detect child insertion
            if (this.extend) {
                observer.disconnect();
                mutations.forEach((mutation) => {
                    if (mutation.addedNodes.length) {
                        // this.extend = this.removeChild(this.extend);
                        mutation.addedNodes.forEach((_child) => {
                            let child = _child as Element;
                            if (child != this.extend && child.tagName) {
                                let tag_name = child.tagName.toLocaleLowerCase();
                                console.log(tag_name);
                                if (
                                    tag_name == 'h1' ||
                                    tag_name == 'h2' ||
                                    tag_name == 'h3' ||
                                    tag_name == 'h4' ||
                                    tag_name == 'h5' ||
                                    tag_name == 'h6'
                                ) {
                                    this.init_heading(child);
                                }
                            }
                        });
                    }
                });
                this.appendChild(this.extend);
                observer.observe(this, { childList: true });
            }
        });
        observer.observe(this, { childList: true });
    }
    add_header() {
        let tag;
        if (this.level == 1) {
            tag = 'h1';
        } else if (this.level == 1) {
            tag = 'h2';
        } else if (this.level == 1) {
            tag = 'h3';
        } else if (this.level == 1) {
            tag = 'h4';
        } else if (this.level == 1) {
            tag = 'h5';
        } else {
            tag = 'h6';
        }
        let element = document.createElement(tag) as HTMLHeadingElement;
    }
    add_section() {
        
    }
    add_note() {
        let entry = new NoteElement();
        this.appendChild(entry);
    }
}



///////////////////////////////////////////////////////////////////////////////
// REGISTER
///////////////////////////////////////////////////////////////////////////////

window.customElements.define('x-tex', InlineLatexElement);
window.customElements.define('x-texblock', BlockLatexElement);
window.customElements.define('x-equation', EquationElement);

window.customElements.define('page-toolkit', PageToolkitElement, {extends: "canvas"});
window.customElements.define('x-note', NoteElement);
window.customElements.define('x-layout', LayoutElement);

///////////////////////////////////////////////////////////////////////////////
// ENTRYPOINT
///////////////////////////////////////////////////////////////////////////////

function start() {
    // let page_toolkit = new PageToolkit();
    // document.body.appendChild(page_toolkit);
    // let controller = new PageLayers(document.body);
    // console.log(controller);
    // document.body.appendChild(controller);
    
    // let page_layout = new LayoutElement();
    // for (const i of [0, 0, 2, 3, 4, 5, 6, 7, 8]) {
    //     page_layout.add_entry();
    // }
    // document.body.appendChild(page_layout);
}

window.onload = () => {
    start();
}; 

export {}
