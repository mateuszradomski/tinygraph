const svg = document.getElementById("main");

const SVG_HTML_NAMESPACE = "http://www.w3.org/2000/svg";
let values = new Array(1000).fill(0);
values = values.map((_, i) => Math.sin(i / 35));

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

function setAttributes(elem, attrs) {
  for (const key in attrs) {
    elem.setAttribute(key, attrs[key]);
  }
}

// TODO(radomski): Error reporting...
async function parseTGPH() {
  const readString = (bytes, dataView, offset) => {};

  const response = await fetch("data.tgph");
  const content = await response.blob();
  const bytes = await content.arrayBuffer();
  const dataView = new DataView(bytes);

  const parser = {
    bytes: bytes,
    dataView: dataView,
    offset: 0,

    readF32() {
      const res = this.dataView.getFloat32(this.offset, true);
      this.offset += 4;
      return res;
    },

    readU32() {
      const res = this.dataView.getUint32(this.offset, true);
      this.offset += 4;
      return res;
    },

    readU16() {
      const res = this.dataView.getUint16(this.offset, true);
      this.offset += 2;
      return res;
    },

    readU8() {
      const res = this.dataView.getUint8(this.offset, true);
      this.offset += 1;
      return res;
    },

    readString() {
      let length = this.readU8();
      if (length === 0xff) {
        length = this.readU16();
      }

      const stringBytes = new Uint8Array(this.bytes, this.offset, length);
      const decoder = new TextDecoder("utf-8");
      const string = decoder.decode(stringBytes);
      this.offset += length;
      return string;
    },
  };

  const magic = parser.readU32();
  const version = parser.readU8();
  const containerCount = parser.readU16();

  console.assert(0x48504754 === magic);
  console.assert(1 === version);

  let containers = [];
  for (let i = 0; i < containerCount; i++) {
    let name = parser.readString();
    const elementType = parser.readU8();
    const elementCount = parser.readU32();

    const elements = [];
    switch (elementType) {
      case 1:
        for (let j = 0; j < elementCount; j++) {
          elements.push(parser.readU32());
        }
        break;
      case 2:
        for (let j = 0; j < elementCount; j++) {
          elements.push(parser.readF32());
        }
        break;
      case 3:
        for (let j = 0; j < elementCount; j++) {
          elements.push(parser.readString());
        }
        break;
      default:
        console.assert(false);
        break;
    }

    containers.push({
      name: name,
      type: elementType,
      elements: elements,
    });
  }

  return containers;
}

class LineGraph {
  constructor(svg) {
    this.svg = svg;

    this.rulers = [];
    this.rulerCaptions = [];
    for (let i = 0; i < 5; i++) {
      this.rulers.push(document.createElementNS(SVG_HTML_NAMESPACE, "line"));
      this.rulerCaptions.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "text")
      );
    }

    this.rulers.forEach((r) => {
      r.setAttribute("stroke", "grey");
      r.setAttribute("stroke-opacity", "0.25");
      this.svg.appendChild(r);
    });

    this.rulerCaptions.forEach((cap) => {
      cap.setAttribute("style", "font: 1em serif;");
      this.svg.appendChild(cap);
    });

    this.polyline = document.createElementNS(SVG_HTML_NAMESPACE, "polyline");

    setAttributes(this.polyline, {
      id: "data",
      stroke: "pink",
      "stroke-width": "2px",
      fill: "none",
    });
    this.svg.appendChild(this.polyline);

    this.hoverLine = document.createElementNS(SVG_HTML_NAMESPACE, "line");
    this.hoverCircle = document.createElementNS(SVG_HTML_NAMESPACE, "circle");

    setAttributes(this.hoverLine, {
      stroke: "white",
      class: "hidden",
      "stroke-width": "2px",
    });
    setAttributes(this.hoverCircle, {
      stroke: "white",
      class: "hidden",
      "stroke-width": "2px",
      r: "3",
    });

    this.svg.appendChild(this.hoverLine);
    this.svg.appendChild(this.hoverCircle);

    svg.addEventListener("mousemove", (e) => {
      setAttributes(this.hoverLine, {
        x1: `${e.offsetX}`,
        y1: "0",
        x2: `${e.offsetX}`,
        y2: "600",
      });
      setAttributes(this.hoverCircle, {
        cx: `${e.offsetX}`,
        cy: `${this.getInterpolatedY(e.offsetX)}`,
      });
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
    let result =
      this.paddedHeight *
      ((val - this.valueMin) / (this.valueMax - this.valueMin));
    let inverted = this.paddedHeight - result;
    return inverted + this.paddingSpace;
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

    this.polyline.setAttribute("points", pointsAttribValue);

    //
    // Rulers and their captions
    //
    this.rulers.forEach((r, i) => {
      const denom = this.rulers.length - 1;
      const y = i * (this.paddedHeight / denom) + this.paddingSpace;
      setAttributes(r, {
        x1: "0",
        y1: `${y}`,
        x2: `${this.width}`,
        y2: `${y}`,
      });
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

window.onload = async () => {
  containers = await parseTGPH();
  testGraph.draw(values);
};

window.addEventListener("resize", (_) => {
  testGraph.draw(values);
});
