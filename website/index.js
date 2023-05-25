const SVG_HTML_NAMESPACE = "http://www.w3.org/2000/svg";

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

function setAttributes(elem, attrs) {
  for (const key in attrs) {
    elem.setAttribute(key, attrs[key]);
  }
}

function wrapSvgAndAppendToGlobalContainer(insertDiv, isHalfSize, svg) {
  const div = document.createElement("div");
  if (isHalfSize) {
    div.setAttribute("class", "half_graph");
    div.setAttribute("style", "position: relative;");
  } else {
    div.setAttribute("class", "graph");
    div.setAttribute("style", "position: relative;");
  }

  div.appendChild(svg);
  insertDiv.appendChild(div);
}

async function decompressToByteArray(compressedData) {
  const ds = new DecompressionStream("gzip");
  const stream = compressedData.stream().pipeThrough(ds);
  const reader = stream.getReader();
  const chunks = [];

  let totalSize = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    totalSize += value.length;
  }

  const concatenatedChunks = new Uint8Array(totalSize);
  let offset = 0;
  for (const chunk of chunks) {
    concatenatedChunks.set(chunk, offset);
    offset += chunk.length;
  }

  return concatenatedChunks;
}

async function fetchAndParseTGPH() {
  const response = await fetch("data.tgph.gz");
  const blob = await decompressToByteArray(await response.blob());
  const bytes = blob.buffer;
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

  if (magic !== 0x48504754) {
    throw new Error("Invalid magic at the start of fetched file");
  }
  if (version !== 1) {
    throw new Error("Unexpected version in the fetched TGPH file");
  }

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
        throw new Error(`Unexpected elementType = ${elementType}`);
    }

    containers.push({
      name: name,
      type: elementType,
      elements: elements,
    });
  }

  return containers;
}

function convertRemToPixels(rem) {
  return rem * parseFloat(getComputedStyle(document.documentElement).fontSize);
}

class HoverInfo {
  constructor() {
    this.topElement = document.createElement("div");
    this.hide();
  }

  show() {
    this.topElement.setAttribute("class", "");
  }

  setPosition(x, y, parentWidth, parentHeight) {
    const height = convertRemToPixels(5);
    const padding = convertRemToPixels(1);
    const half_height = height / 2;

    let horizontalStyle = "";
    if (x > parentWidth / 2) {
      horizontalStyle = `right: ${parentWidth - x + padding}px`;
    } else {
      horizontalStyle = `left: ${x + padding}px`;
    }

    let verticalStyle = "";
    if (y - half_height > parentHeight / 2) {
      verticalStyle = `bottom: ${Math.max(
        padding,
        parentHeight - y - half_height
      )}px`;
    } else {
      verticalStyle = `top: ${Math.max(padding, y - half_height)}px`;
    }

    this.topElement.setAttribute(
      "style",
      `${verticalStyle}; ${horizontalStyle}; background: pink; z-index: 50; height: 5rem; width: 10rem; position: absolute;`
    );
  }

  hide() {
    this.topElement.setAttribute("class", "hidden");
  }
}

class LineGraph {
  constructor(values, name) {
    this.topElement = document.createElement("div");
    this.svg = document.createElementNS(SVG_HTML_NAMESPACE, "svg");
    this.hoverInfo = new HoverInfo();

    this.topElement.appendChild(this.svg);
    this.topElement.appendChild(this.hoverInfo.topElement);

    this.values = values;
    this.name = name;

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
      r.setAttribute("stroke-dasharray", "5,5");
      this.svg.appendChild(r);
    });

    this.rulerCaptions.forEach((cap) => {
      cap.setAttribute("style", "font: 1em serif;");
      this.svg.appendChild(cap);
    });

    this.polyline = document.createElementNS(SVG_HTML_NAMESPACE, "polyline");

    setAttributes(this.polyline, {
      id: "data",
      stroke: `${this.generateColorForLine()}`,
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

    this.svg.addEventListener("mousemove", (e) => {
      const screenX = this.getClosestPointScreenSpaceX(e.offsetX);
      const screenY = this.getClosestPointScreenSpaceY(e.offsetX);
      this.hoverInfo.setPosition(screenX, screenY, this.width, this.height);
      setAttributes(this.hoverLine, {
        x1: `${screenX}`,
        y1: "0",
        x2: `${screenX}`,
        y2: "600",
      });
      setAttributes(this.hoverCircle, {
        cx: `${screenX}`,
        cy: `${screenY}`,
      });
    });

    this.svg.addEventListener("mouseenter", (_) => {
      this.hoverLine.setAttribute("class", "");
      this.hoverCircle.setAttribute("class", "");
      this.hoverInfo.show();
    });

    this.svg.addEventListener("mouseleave", (_) => {
      this.hoverLine.setAttribute("class", "hidden");
      this.hoverCircle.setAttribute("class", "hidden");
      this.hoverInfo.hide();
    });
  }

  getTopElement() {
    return this.topElement;
  }

  // TODO(radomski): Multilines, add nonce
  generateColorForLine() {
    const encoder = new TextEncoder();
    const data = encoder.encode(this.name);
    // TODO(radomski): This really should be awaited
    crypto.subtle.digest("SHA-256", data);
    const random = data[0] | (data[1] << 8) | (data[2] << 16);
    return `hsl(${random % 360.0}, 100%, 65%)`;
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

  draw() {
    const bbox = this.svg.getBoundingClientRect();
    this.width = bbox.width;
    this.height = bbox.height;
    this.verticalPadding = 0.05; // 5%
    this.paddingSpace = this.height * this.verticalPadding;
    this.paddingRoom = this.paddingSpace * 2;
    this.paddedHeight = this.height - this.paddingRoom;

    if (this.values.length === 0) {
      return 0;
    }

    const [min, max] = this.getMinMax(this.values);
    this.valueMin = min;
    this.valueMax = max;

    this.horizontalScaling = this.width / (this.values.length - 1);
    const pointsAttribValue = this.values
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

  getClosestPointIndex(x) {
    const i = Math.floor(x / this.horizontalScaling);

    if (i >= this.values.length) {
      return this.values.length - 1;
    }

    const dist = [i, i + 1].map((v) =>
      Math.abs(x - v * this.horizontalScaling)
    );
    return dist[0] < dist[1] ? i : i + 1;
  }

  getClosestPointScreenSpaceX(x) {
    return this.getClosestPointIndex(x) * this.horizontalScaling;
  }

  getClosestPointScreenSpaceY(x) {
    return this.toScreenSpaceHeight(this.values[this.getClosestPointIndex(x)]);
  }
}

const insertDiv = document.getElementById("global_insert_space");

function createLineGraphForContainer(container, halfSize) {
  const graph = new LineGraph(container.elements, container.name);
  wrapSvgAndAppendToGlobalContainer(insertDiv, halfSize, graph.getTopElement());
  return graph;
}

let containers = undefined;
let graphs = [];

window.onload = async () => {
  containers = await fetchAndParseTGPH();

  console.log(containers);
  graphs.push(
    createLineGraphForContainer(
      containers.filter(
        (c) => c.name === "Interface enp1s0 Received [bytes]"
      )[0],
      true
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter(
        (c) => c.name === "Interface enp1s0 Transmitted [bytes]"
      )[0],
      true
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name === "Unix timestamp")[0],
      false
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name === "Used memory [MB]")[0],
      false
    )
  );
  graphs.forEach((g) => g.draw());
};

window.addEventListener("resize", (_) => {
  if (graphs === undefined) {
    return;
  }

  graphs.forEach((g) => g.draw());
});
