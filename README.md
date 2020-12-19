# Subscript - Content Publishing using Web Technologies

> NOTE
> * <sup>[2020/12/16]</sup> **This is a fresh git tree.** There was too much binary data in the original git tree, and for other related reasons, I’ve decided to start with a new git repo for my Subscript project. The old subscript git tree can be found here [github.com/colbyn/subscript](https://github.com/colbyn/subscript).
> * <sup>[2019]</sup> originally Subscript referred to a frontend UI framework, but that has been abandoned, I’m recycling the old Subscript name for a new project. The old project can be found here [colbyn/subscript-old](https://github.com/colbyn/subscript-old).

## Preview

![Preview](assets/desmos-preview.png)

### General Features

- [x] Rust Macros
- [x] Macros VIA a *simple* embedded scripting language that supports WebAssembly ([Example](examples/school-notes/plugins/desmos.rhai))
- [ ] Unicode Prettification (E.g. convert `"Lorem"` to `“lorem”`.)
- [ ] Macros VIA *some more mainstream* embedded scripting language (ideally one that is sandboxed such as Deno)
- [ ] Paged Media Support
- [ ] PDF Rendering (dependent on `Paged Media Support` for native page handling)

### Supported Content (Using Subscript's Macro System | Not Comprehensive)

- **Include Files:**
  ```html
  <include src="../template/base.html">
      <h1>My Book</h1>
      
      <include src="../content/chapter1.html"></include>
      <include src="../content/chapter2.html"></include>
      <include src="../content/chapter3.html"></include>

      <!--
        NOTE: SEE HTML SYNTAX HIGHLIGHTING EXTENSION FOR VS-CODE
           (MAKES MIXING LATEX/HTML MORE BEARABLE)
        -->
      <h2>Graph of <tex>y = x^2</tex></h2>
      <desmos>
          <expr>y = x^2</expr>
      </desmos>
  </include>
  ```

- **Graphing:**
  ```html
  <desmos>
      <expr>y = x^2</expr>
  </desmos>
  ```

- **Mathematics:**
  ```html
  <!--
    NOTE: SEE HTML SYNTAX HIGHLIGHTING EXTENSION FOR VS-CODE
        (MAKES MIXING LATEX/HTML MORE BEARABLE)
  -->
  <equation>
      f \triangleleft x &= f(x) \\
      x \triangleright f &= f(x) \\
      |x| &= \sqrt{x^2} \neq x \\
      |x|^2 &= x^2
  </equation>
  ```

- Lots of simple conveniences (a lot is still being moved over from the [original implementation](https://github.com/colbyn/subscript)):
    * Layout (originally called `<gallery>`):
      ```html
      <layout columns="3">
        <equation>\delta \sin(x) &= \cos(x)</equation>
        <equation>\delta \cos(x) &= -\sin(x)</equation>
        <equation>\delta \tan(x) &= \sec^2(x)</equation>
        <equation>\delta \csc(x) &= -\cot(x)\csc(x)</equation>
        <equation>\delta \sec(x) &= \tan(x)\sec(x)</equation>
        <equation>\delta \cot(x) &= -\csc^2(x)</equation>
      </layout>
      ```
    * This helps reduce nesting:
      ```html
      <list>
        <p>Lemon drops sweet roll cupcake biscuit cookie. Ice cream pie apple pie fruitcake dessert sweet roll chocolate bar.</p>
        <p>Sesame snaps lollipop marshmallow marzipan</p>
      </list>
      ```
    * Generate a “Table Of Contents” at the given location:
      ```html
      <toc></toc>
      ```
      Regarding the `<toc>` macro, this also works with `<include>`'d content, thanks to how Subscript processes macros in a bottom-up manner<sup>(unlike PostHTML + Parcel, which drove me crazy)</sup>.
    * Ad-hoc styling | This targets the parent node with a unique CSS class name:
      ```html
      <style self>
          self {
              display: grid;
              /* ... */
          }
          @media (max-width: 900px) {
              self {
                  grid-template-columns: 1fr;
              }
          }
      </style>
      ```
    * Images from a file glob [TODO]:
      ```html
      <image-gallery src="../static/images/chapter1/*.jpg" columns="2"></image-gallery>
      ```
      This pattern was very common with my [old school notes](https://colbyn.github.io/subscript/calc1/chapter6.html), where I could include [screenshots](https://colbyn.github.io/subscript/calc1/chapter6.html#2681476879558479754) of all the essential definitions from a given chapter.

Versatility in Subscript is made possible VIA macros, the syntax is akin to web components, but it's expanded out at **compile time**, instead of at runtime (i.e. a macro).


## What is Subscript?

- If you are a web developer:
    - Subscript is a akin web application bundlers such as Parcel, but is -better suited- for mostly static content publishing. For those who say otherwise, see my old [GitHub repository (colbyn/school-notes)](https://github.com/colbyn/school-notes), using Parcel resulted in significant friction from a multitude of problems, notably being that Parcel and PostHTML do not interact very well, especially with nested `<include>` resources and relative file paths.
      + For example, module A and B both include module C, where module C includes asset D. PostHTML processes `<include>` files in a top-down manner, so therefore, after inlining module C in A and B -A and B now reference module asset D, using a file path relative to module C... You can imagine Parcel would then throw errors at this... Subscript by contract mostly works in a **bottom-up** manner, where module C is processed first, then modules A and B.

- If you are from academia:
    - Subscript is akin to LaTeX, both are markup languages for displaying content.

      Furthermore both are geard towards **STEM based content** by default (unlike the [Sile typesetter](https://sile-typesetter.org)<sup>†</sup> that doen't support e.g. mathematical notation).

      Yet Subscript is based on web technologies, and therefore can leverage the colossus ecosystem that makes up the web. For instance, need to display a graph of `y=x^2`? Just use a macro that embeds Desmos, and therein simply write:
      ```html
      <desmos height="200px">
          <expr>y=x^2</expr>
      </desmos>
      ```
      
      Or, do you need to embed musical notation? Create a macro that embeds [VexFlow](https://www.vexflow.com/). 

      <sup>[†]:</sup> Regarding Sile and it's innovative layout system, since Subscript is based on web technologies, it can offer responsive grid layouts for different PDF resolutions.


## High Level TODO:

- [CSS Paged Media](https://www.w3.org/TR/css-page-3/): support traditional print use cases, or just rendering to a PDF. This is what I am currently planning on using for rendering to e.g. PDFs: [PagedJS](https://www.pagedjs.org)

## Math Preview

![Preview](assets/preview.png)

Comes with a syntax highlighting extension for VS Code.

![VS-Code Preview](assets/preview-vscode-plugin.png)

It injects the LaTeX grammar from [latex-workshop](https://marketplace.visualstudio.com/items?itemName=James-Yu.latex-workshop) into the `<tex>`, `<texblock>` and the `<equation>` html tags. 

