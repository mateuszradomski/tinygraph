const svg = document.getElementById("main");

const scale = 8;
let values = new Array(1000).fill(0);
values = values.map((_, i) => (i/50));
console.log(values)

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

class LineGraph {
  constructor(svg) {
    this.svg = svg;

    this.rulers = [];
    this.rulerCaptions = [];
    for (let i = 0; i < 5; i++) {
      this.rulers.push(
        document.createElementNS("http://www.w3.org/2000/svg", "line")
      );
      this.rulerCaptions.push(
        document.createElementNS("http://www.w3.org/2000/svg", "text")
      );
      console.log(this.rulerCaptions);
    }

    this.rulers.forEach((r) => {
      r.setAttribute("stroke", "grey");
      r.setAttribute("stroke-opacity", "0.25");
      this.svg.appendChild(r);
    });

    this.rulerCaptions.forEach((cap) => {
      cap.setAttribute("style", "font: bold 1em serif;");
      this.svg.appendChild(cap);
    });

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
    polyline.setAttribute("stroke-width", "2px");
    polyline.setAttribute("fill", "none");

    //
    // Rulers and their captions
    //
    this.rulers.forEach((r, i) => {
      const denom = this.rulers.length - 1;
      const y = i * (this.paddedHeight / denom) + this.paddingSpace;
      r.setAttribute("x1", "0");
      r.setAttribute("y1", `${y}`);
      r.setAttribute("x2", `${this.width}`);
      r.setAttribute("y2", `${y}`);
    });

    this.rulerCaptions.forEach((cap, i) => {
      const denom1 = this.rulerCaptions.length - 1;
      const denom2 =
        (this.valueMax - this.valueMin) / (this.rulerCaptions.length - 1);
      const y =
        (this.rulerCaptions.length - i - 1) * (this.paddedHeight / denom1) +
        this.paddingSpace;

      const rulerValue = i * denom2 + this.valueMin;
      cap.textContent = `${rulerValue.toFixed(2)}`;
      cap.setAttribute("x", "0");
      cap.setAttribute("y", `${y - 2}`);
    });
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
