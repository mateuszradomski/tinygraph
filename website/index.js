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

function getInterpolatedY(x, points) {
  let i = 0;
  for (i = 0; i < points.length; i++) {
    if (points[i][0] > x) {
      break;
    }
  }

  if (i === 0) {
    return points[0][1];
  }

  return lerp(
    points[i - 1][1],
    points[i][1],
    (x - points[i - 1][0]) / Math.abs(points[i][0] - points[i - 1][0])
  );
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
    this.hoverCircle.setAttribute("stroke", "grey");
    this.hoverCircle.setAttribute("r", "3");

    this.svg.appendChild(this.hoverLine);
    this.svg.appendChild(this.hoverCircle);

    svg.addEventListener("mousemove", (e) => {
      this.hoverLine.setAttribute("x1", `${e.offsetX}`);
      this.hoverLine.setAttribute("y1", "0");
      this.hoverLine.setAttribute("x2", `${e.offsetX}`);
      this.hoverLine.setAttribute("y2", "600");

      this.hoverCircle.setAttribute("cx", `${e.offsetX}`);
      // this.hoverCircle.setAttribute(
      //   "cy",
      //   `${getInterpolatedY(e.offsetX, this.values)}`
      // );
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

  draw(values) {
    this.values = values;
    const bbox = this.svg.getBoundingClientRect();
    this.newWidth = bbox.width;
    this.newHeight = bbox.height;

    if (values.length === 0) {
      return 0;
    }

    this.horizontalScaling = this.newWidth / values.length;
    const pointsAttribValue = values
      .map((val, i) => `${i * horizontalScaling}, ${val}`)
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
}

const testGraph = new LineGraph(svg);

window.onload = () => {
  testGraph.draw(values);
};

window.addEventListener("resize", (_) => {
  testGraph.draw(values);
});
