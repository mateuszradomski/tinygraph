const svg = document.getElementById("main");

const scale = 8;
let values = [
  347, 350, 289, 252, 329, 253, 277, 314, 279, 255, 278, 289, 261, 289, 336,
  261, 315, 251, 283, 337, 260, 258, 296, 271, 294, 269, 261, 326, 323, 257,
  257, 259, 296, 256, 324, 268, 321, 281, 342, 301, 253, 277, 284, 332, 333,
  312, 252, 329, 315, 313, 340, 280, 275, 323, 286, 286, 325, 290, 313, 297,
  340, 305, 342, 256, 310, 287, 300, 346, 314, 261, 251, 281, 279, 278, 261,
  319, 313, 311, 331, 300, 250, 291, 266, 280, 307, 287, 273, 279, 345, 328,
  302, 311, 338, 263, 288, 276, 265, 258, 338, 323,
];

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

class LineGraph {
  constructor(svg) {
    this.svg = svg;

    this.hoverLine = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "line"
    );
    this.hoverCircle = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "circle"
    );

    this.hoverLine.setAttribute("stroke", "grey");
    this.hoverLine.setAttribute("class", "hidden");
    this.hoverCircle.setAttribute("stroke", "grey");
    this.hoverCircle.setAttribute("r", "3");
    this.hoverCircle.setAttribute("class", "hidden");

    this.svg.appendChild(this.hoverLine);
    this.svg.appendChild(this.hoverCircle);

    svg.addEventListener("mousemove", (e) => {
      this.hoverLine.setAttribute("x1", `${e.offsetX}`);
      this.hoverLine.setAttribute("y1", "0");
      this.hoverLine.setAttribute("x2", `${e.offsetX}`);
      this.hoverLine.setAttribute("y2", "600");

      this.hoverCircle.setAttribute("cx", `${e.offsetX}`);
      this.hoverCircle.setAttribute(
        "cy",
        `${this.getInterpolatedY(e.offsetX)}`
      );
    });

    svg.addEventListener("mouseenter", (_) => {
      this.hoverLine.setAttribute("class", "");
      this.hoverCircle.setAttribute("class", "");
    });

    svg.addEventListener("mouseleave", (_) => {
      this.hoverLine.setAttribute("class", "hidden");
      this.hoverCircle.setAttribute("class", "hidden");
    });
  }

  getMinMax(values) {
    let max = Number.MIN_VALUE;
    let min = Number.MAX_VALUE;

    for (const v of values) {
      max = Math.max(max, v);
      min = Math.min(min, v);
    }

    return [min, max];
  }

  toScreenSpaceHeight(val) {
    return (
      this.paddingSpace +
      this.paddedHeight *
        ((val - this.valueMin) / (this.valueMax - this.valueMin))
    );
  }

  draw(values) {
    this.values = values;
    const bbox = this.svg.getBoundingClientRect();
    this.width = bbox.width;
    this.height = bbox.height;
    this.verticalPadding = 0.05; // 5%
    this.paddingSpace = this.height * this.verticalPadding;
    this.paddingRoom = this.paddingSpace * 2;
    this.paddedHeight = this.height - this.paddingRoom;

    if (values.length === 0) {
      return 0;
    }

    const [min, max] = this.getMinMax(values);
    console.log(min, max);
    this.valueMin = min;
    this.valueMax = max;

    this.horizontalScaling = this.width / values.length;
    const pointsAttribValue = values
      .map(
        (val, i) =>
          `${i * this.horizontalScaling}, ${this.toScreenSpaceHeight(val)}`
      )
      .join(" ");

    let polyline = this.svg.getElementById("data");
    if (polyline === null) {
      polyline = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "polyline"
      );
      this.svg.appendChild(polyline);
    }

    polyline.setAttribute("points", pointsAttribValue);
    polyline.setAttribute("stroke", "pink");
    polyline.setAttribute("fill", "none");
    // TODO(radomski): Generate unique ID
    polyline.setAttribute("id", "data");
  }

  getInterpolatedY(x) {
    const i = Math.floor(x / this.horizontalScaling) + 1;

    if (i === 0) {
      return this.toScreenSpaceHeight(values[0]);
    }

    return lerp(
      this.toScreenSpaceHeight(values[i - 1]),
      this.toScreenSpaceHeight(values[i]),
      (x - (i - 1) * this.horizontalScaling) / Math.abs(this.horizontalScaling)
    );
  }
}

const testGraph = new LineGraph(svg);

window.onload = () => {
  testGraph.draw(values);
};

window.addEventListener("resize", (_) => {
  testGraph.draw(values);
});
