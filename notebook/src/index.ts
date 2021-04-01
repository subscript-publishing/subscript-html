// import init from 'mathjax-full';
import * as katex from 'katex';
import {curveToBezier, pointsOnBezierCurves} from './curve-to-bezier';


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

function cartesian_to_polar(pos: {x: number, y: number}): {r: number, theta: number} {
    const {x, y} = pos;
    return {
        r: Math.sqrt(x * x + y * y),
        theta: Math.atan2(y, x),
    };
}

function polar_to_cartesian({r, theta}: {r: number, theta: number}): {x: number, y: number} {
    return {
        x: r * Math.cos(theta),
        y: r * Math.sin(theta),
    };
}

///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////


class PolarPoint {
    radius: number;
    magnitude: number;
    constructor(radius: number, magnitude: number) {
        this.radius = radius;
        this.magnitude = magnitude;
    }
    draw(ctx: CanvasRenderingContext2D, origin: {x: number, y: number}) {
        const x = this.magnitude * Math.cos(this.radius);
        const y = this.magnitude * Math.sin(this.radius);
        const pos = {x, y};
        ctx.beginPath(); // begin
        ctx.lineWidth = 5;
        ctx.lineCap = 'round';
        ctx.strokeStyle = '#29bcd442';
        ctx.moveTo(origin.x, origin.y);
        ctx.lineTo(pos.x, pos.y);
        ctx.strokeStyle = '#29bcd442';
        ctx.stroke();
    }
}

interface Point {
    x: number,
    y: number,
}

class Curve {
    points: Array<[number, number]>;
    constructor(plotter: Plotter) {
        let points: Array<[number, number]> = [];
        for (const point of plotter.points) {
            points.push([point.x, point.y]);
        }
        this.points = points;
    }
    draw_circle(ctx: CanvasRenderingContext2D, pos: Point) {
        let {x, y} = pos;
        ctx.beginPath();
        ctx.arc(x, y, 10, 0, 2 * Math.PI);
        ctx.stroke();
    }
    draw_curve(ctx: CanvasRenderingContext2D, pos: Point) {
        // let {x, y} = pos;
        // ctx.beginPath(); // begin
        // ctx.lineWidth = 5;
        // ctx.lineCap = 'round';
        // ctx.strokeStyle = '#29bcd442';
        // ctx.moveTo(this.origin.x, this.origin.y);
        // ctx.lineTo(pos.x, pos.y);
        // ctx.stroke();
    }
    draw_point(ctx: CanvasRenderingContext2D, pos: Point) {
        // let {x, y} = pos;
        // ctx.beginPath();
        // ctx.arc(x, y, 1, 0, 2 * Math.PI);
        // ctx.fill();
        // // ctx.stroke();
    }
    draw(canvas: SceneTree, ctx: CanvasRenderingContext2D) {

    }
}

class Plotter {
    origin: {x: number, y: number};
    points: Array<Point> = [];
    constructor(origin: Point) {
        this.origin = origin;
    }
    draw_circle(ctx: CanvasRenderingContext2D, pos: Point) {
        let {x, y} = pos;
        ctx.beginPath();
        ctx.arc(x, y, 10, 0, 2 * Math.PI);
        ctx.stroke();
    }
    draw_curve(ctx: CanvasRenderingContext2D, pos: Point) {
        let {x, y} = pos;
        ctx.beginPath(); // begin
        ctx.lineWidth = 5;
        ctx.lineCap = 'round';
        ctx.strokeStyle = '#29bcd442';
        ctx.moveTo(this.origin.x, this.origin.y);
        ctx.lineTo(pos.x, pos.y);
        ctx.stroke();
    }
    draw_point(ctx: CanvasRenderingContext2D, pos: Point) {
        let {x, y} = pos;
        ctx.beginPath();
        ctx.arc(x, y, 1, 0, 2 * Math.PI);
        ctx.fill();
        // ctx.stroke();
    }
    draw(ctx: CanvasRenderingContext2D, pos: Point) {
        this.draw_point(ctx, pos);
        this.points.push(pos);
    }
}

class SceneTree extends HTMLCanvasElement {
    uid: string;
    dpr: number;
    pos = { x: 0, y: 0 };
    // origin: {x: number, y: number};
    curves: Array<Curve> = [];
    current_plotter: null | Plotter = null;
    constructor() {
        super();
        this.setAttribute('scene-tree', '');
        this.uid = guid();
    }
    resolution() {
        // let ctx = this.getContext('2d');
        let dpr = this.dpr;
        let rect = this.getBoundingClientRect();
        this.width = rect.width * dpr;
        this.height = rect.height * dpr;
    }
    set_position(event: MouseEvent) {
        let rect = this.getBoundingClientRect();
        this.pos.x = (event.clientX - rect.left) * this.dpr;
        this.pos.y = (event.clientY - rect.top) * this.dpr;
    }
    draw_circle(x: number, y: number) {
        let ctx = this.getContext('2d');
        ctx.beginPath();
        ctx.arc(x, y, 10, 0, 2 * Math.PI);
        ctx.stroke();
    }
    draw_point(x: number, y: number) {
        let ctx = this.getContext('2d');
        ctx.beginPath();
        ctx.arc(x, y, 10, 0, 2 * Math.PI);
        ctx.fill();
    }
    reset_plotter() {
        if (this.current_plotter && this.current_plotter.points.length >= 3) {
            let new_curve = new Curve(
                this.current_plotter
            );
            // console.log(new_curve);
            this.curves.push(new_curve);
        }
        this.current_plotter = null;
        this.resolution();
        let ctx = this.getContext('2d');
        for (let curve of this.curves) {
            curve.draw(this, ctx);
        }
    }
    on_draw(event: MouseEvent) {
        let ctx = this.getContext('2d');
        // mouse left button must be pressed:
        if (event.buttons !== 1) return;
        this.set_position(event);
        if (!this.current_plotter) {
            const origin = {
                x: this.pos.x,
                y: this.pos.y,
            };
            this.current_plotter = new Plotter(origin);
        }
        this.current_plotter.draw(ctx, this.pos);
    }
    connectedCallback() {
        this.setAttribute('uid', this.uid);
        this.dpr = window.devicePixelRatio || 1;
        this.resolution();
        window.addEventListener('resize', () => {this.resolution()}, false);
        this.addEventListener('mouseup', (event) => {
            this.reset_plotter();
        });
        this.addEventListener('mousedown', (event) => {
            // console.log("mousedown", event);
            // this.set_position(event);
        });
        this.addEventListener('mouseenter', (event) => {
            // this.set_position(event);
            // this.set_position(event);
        });
        this.addEventListener('mousemove', (event) => {
            // this.set_position(event);
            // console.log("mousemove", event);
            this.on_draw(event);
        });
    }
}

///////////////////////////////////////////////////////////////////////////////
// PAGE ENTRY
///////////////////////////////////////////////////////////////////////////////


///////////////////////////////////////////////////////////////////////////////
// PAGE LAYOUT
///////////////////////////////////////////////////////////////////////////////


class PageElement extends HTMLElement {
    uid: string;
    header: string | null;
    broken_footer: boolean;
    scene_tree: SceneTree;
    constructor() {
        super();
        this.uid = guid();
        this.header = null;
        this.broken_footer = true;
        this.scene_tree = new SceneTree();
    }
    init_dom() {
        let header = document.createElement('header');
        let footer = document.createElement('footer');
        this.appendChild(header);
        this.appendChild(this.scene_tree);
        this.appendChild(footer);
    }
    connectedCallback() {
        this.setAttribute('uid', this.uid);
        this.init_dom();
        let observer = new MutationObserver((mutations) => {
            observer.disconnect();
            mutations.forEach((mutation) => {
                if (mutation.addedNodes.length) {
                    mutation.addedNodes.forEach((child) => {});
                }
            });
            observer.observe(this, { childList: true });
        });
        observer.observe(this, { childList: true });
    }
}



///////////////////////////////////////////////////////////////////////////////
// REGISTER
///////////////////////////////////////////////////////////////////////////////

window.customElements.define('x-scene-tree', SceneTree, {extends: "canvas"});
window.customElements.define('x-page', PageElement);


///////////////////////////////////////////////////////////////////////////////
// ENTRYPOINT
///////////////////////////////////////////////////////////////////////////////

function start() {
    let page = new PageElement();
    document.body.appendChild(page);
}

window.onload = () => {
    start();
}; 

export {}