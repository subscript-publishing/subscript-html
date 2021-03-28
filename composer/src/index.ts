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
    constructor() {
        super();
        this.uid = guid();
        this.level = 1;
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