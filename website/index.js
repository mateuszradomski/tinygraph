const svg = document.getElementById("main");

const scale = 8;
let values = new Array(1000).fill(0)
values = values.map((_, i) => Math.sin(i / 50))

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

class LineGraph {
  constructor(svg) {
    this.svg = svg;

    let polyline = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "polyline"
    );
    // TODO(radomski): Generate unique ID
    polyline.setAttribute("id", "data");
    this.svg.appendChild(polyline);

    this.hoverLine = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "line"
    );
    this.hoverCircle = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "circle"
    );

    this.hoverLine.setAttribute("stroke", "white");
    this.hoverLine.setAttribute("stroke-width", "2px");
    this.hoverLine.setAttribute("class", "hidden");
    this.hoverCircle.setAttribute("stroke", "white");
    this.hoverCircle.setAttribute("stroke-width", "2px");
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

    this.horizontalScaling = this.width / (values.length - 1);
    const pointsAttribValue = values
      .map(
        (val, i) =>
          `${i * this.horizontalScaling}, ${this.toScreenSpaceHeight(val)}`
      )
      .join(" ");

    let polyline = this.svg.getElementById("data");

    polyline.setAttribute("points", pointsAttribValue);
    polyline.setAttribute("stroke", "pink");
    polyline.setAttribute("fill", "none");
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
