const SVG_HTML_NAMESPACE = "http://www.w3.org/2000/svg";

function lerp(k0, k1, t) {
  return k0 + t * (k1 - k0);
}

function setAttributes(elem, attrs) {
  for (const key in attrs) {
    elem.setAttribute(key, attrs[key]);
  }
}

function wrapSvgAndAppendToGlobalContainer(insertDiv, svg) {
  const div = document.createElement("div");
  div.setAttribute("class", "graph");

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
  const response = await fetch("data.tgph.gz", { cache: "no-store" });
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

const monthNames = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

function ifFloatNarrow(val) {
  return val % 1 === 0 ? val : val.toFixed(2);
}

class HoverInfo {
  constructor() {
    this.topElement = document.createElement("div");
    this.hide();
    this.timeParagraph = document.createElement("span");
    this.valueParagraphs = [];

    this.timeParagraph.setAttribute(
      "style",
      "line-height: 1.25rem; color: #F8F8FA;"
    );

    this.topElement.appendChild(this.timeParagraph);
  }

  show() {
    this.topElement.setAttribute("class", "");
  }

  padWithZero(val) {
    return val < 10 ? `0${val}` : `${val}`;
  }

  updateInformation(
    dataArrays,
    pointIndex,
    timestamp,
    names,
    x,
    y,
    parentWidth,
    parentHeight
  ) {
    this.setPosition(x, y, parentWidth, parentHeight);
    this.createValueParagraphs(dataArrays, pointIndex, names);
    this.valueParagraphs.textContent = dataArrays[0][pointIndex];
    const date = new Date(timestamp * 1000);
    const yy = date.getFullYear();
    const mm = monthNames[date.getMonth()];
    const dd = this.padWithZero(date.getDate());
    const HH = this.padWithZero(date.getHours());
    const MM = this.padWithZero(date.getMinutes());
    const SS = this.padWithZero(date.getSeconds());
    this.timeParagraph.textContent = `${yy} ${mm} ${dd} ${HH}:${MM}:${SS}`;
  }

  createValueParagraphs(dataArrays, pointIndex, names) {
    if (this.valueParagraphs.length !== dataArrays.length) {
      this.valueParagraphs.forEach((p) => this.topElement.removeChild(p));
      this.valueParagraphs = [];

      for (let i = 0; i < dataArrays.length; i++) {
        this.valueParagraphs.push(document.createElement("span"));
        this.topElement.appendChild(this.valueParagraphs[i]);
      }
    }

    this.valueParagraphs.forEach((p) => {
      p.setAttribute("style", "line-height: 1.25rem; color: #F8F8FA; text-align: left;");
    });

    for (let i = 0; i < dataArrays.length; i++) {
      const text = `${names[i]} : ${ifFloatNarrow(dataArrays[i][pointIndex])}`;
      this.valueParagraphs[i].textContent = text;
    }
  }

  setPosition(x, y, parentWidth, parentHeight) {
    const height = convertRemToPixels(5);
    const padding = convertRemToPixels(1);
    const halfHeight = height / 2;

    let horizontalStyle = "";
    if (x > parentWidth / 2) {
      const value = parentWidth - x + padding;
      horizontalStyle = `right: ${value}px`;
    } else {
      const value = x + padding;
      horizontalStyle = `left: ${value}px`;
    }

    let verticalStyle = "";
    if (y - halfHeight > parentHeight / 2) {
      const value = Math.max(padding, parentHeight - y - halfHeight);
      verticalStyle = `bottom: ${value}px`;
    } else {
      const value = Math.max(padding, y - halfHeight);
      verticalStyle = `top: ${value}px`;
    }

    this.topElement.setAttribute(
      "style",
      `${verticalStyle}; ${horizontalStyle}; border-radius: 1rem; background: #424850; z-index: 50; min-height: 5rem; min-width: 10rem; position: absolute; text-align: right; padding-left: 1rem; display: flex; justify-content: center; flex-direction: column; padding-right: 0.6rem;`
    );
  }

  hide() {
    this.topElement.setAttribute("class", "hidden");
  }
}

// Very poorly translated from stb_ds.h
function STBDS_ROTATE_LEFT(val, n) {
  return val << n;
}

function STBDS_ROTATE_RIGHT(val, n) {
  return val >> n;
}

function hashString(str) {
  var hash = 0xcafebabe;

  for (let i = 0; i < str.length; i++) {
    hash += STBDS_ROTATE_LEFT(hash, 9) + str.charCodeAt(i);
  }

  hash ^= 0xcafebabe;
  hash = ~hash + (hash << 18);
  hash ^= STBDS_ROTATE_RIGHT(hash, 31);
  hash = hash * 21;
  hash ^= STBDS_ROTATE_RIGHT(hash, 11);
  hash += hash << 6;
  hash ^= STBDS_ROTATE_RIGHT(hash, 22);

  return hash + 0xcafebabe;
}

// TODO(radomski): Multilines, add nonce
function generateColorFromString(name) {
  const random = hashString(name);
  return `hsl(${random % 360.0}, 100%, 65%)`;
}

class TitleAndLegend {
  constructor(titleText, legendeNames) {
    this.text = titleText;
    this.legendeNames = legendeNames;
    this.legendeColors = legendeNames.map((name) =>
      generateColorFromString(name)
    );

    this.textElement = document.createElement("span");
    this.textElement.setAttribute("style", "font: 2rem serif; color: #F8F8FA;");
    this.textElement.textContent = titleText;

    this.topElement = document.createElement("div");
    this.topElement.setAttribute(
      "style",
      "display: flex; justify-content: space-between; height: 3rem;"
    );
    this.spanDiv = document.createElement("div");
    this.spanDiv.setAttribute(
      "style",
      "text-align: right; padding-right: 0.5rem; flex-shrink: 0;"
    );
    this.spanDiv.appendChild(this.textElement);

    this.legendeDiv = document.createElement("div");
    this.legendeDiv.setAttribute("style", "display: flex; flex-wrap: wrap;");
    for (const legendeName of this.legendeNames) {
      this.legendeDiv.appendChild(this.createLegendeElement(legendeName));
    }

    this.topElement.appendChild(this.legendeDiv);
    this.topElement.appendChild(this.spanDiv);
  }

  createLegendeElement(legendeName) {
    const div = document.createElement("div");
    const svg = document.createElementNS(SVG_HTML_NAMESPACE, "svg");
    const line = document.createElementNS(SVG_HTML_NAMESPACE, "line");
    const nameText = document.createElement("span");
    nameText.textContent = legendeName;
    nameText.setAttribute("style", "color: #F8F8FA;");

    div.setAttribute("style", "padding-left: 0.5rem; padding-right: 0.5rem;");

    setAttributes(svg, {
      width: "20",
      height: "10",
    });
    setAttributes(line, {
      x1: "0",
      y1: "5",
      x2: "20",
      y2: "5",
      stroke: generateColorFromString(legendeName),
      "stroke-width": "2px",
    });

    svg.appendChild(line);
    div.appendChild(svg);
    div.appendChild(nameText);

    return div;
  }

  getElement() {
    return this.topElement;
  }
}

class LineGraph {
  constructor(valueArray, times, names, title) {
    this.topElement = document.createElement("div");
    this.title = new TitleAndLegend(title, names);
    this.svg = document.createElementNS(SVG_HTML_NAMESPACE, "svg");
    this.svg.setAttribute("class", "full_svgs");
    this.hoverInfo = new HoverInfo();

    this.svgWrapper = document.createElement("div");
    this.svgWrapper.setAttribute("class", "svg_wrapper");
    this.svgWrapper.appendChild(this.svg);

    this.topElement.appendChild(this.title.getElement());
    this.topElement.appendChild(this.svgWrapper);
    this.topElement.appendChild(this.hoverInfo.topElement);

    this.times = times;
    this.valueArray = valueArray;
    this.names = names;

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
      cap.setAttribute("style", "font: 1em serif; fill: #848484;");
      this.svg.appendChild(cap);
    });

    this.polylines = [];
    for (let i = 0; i < this.valueArray.length; i++) {
      this.polylines.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "polyline")
      );
    }

    this.polylines.forEach((polyline, index) => {
      setAttributes(polyline, {
        id: "data",
        stroke: `${generateColorFromString(this.names[index])}`,
        "stroke-width": "2px",
        fill: "none",
      });
      this.svg.appendChild(polyline);
    });

    this.hoverLine = document.createElementNS(SVG_HTML_NAMESPACE, "line");
    this.hoverCircles = [];
    for (let i = 0; i < this.valueArray.length; i++) {
      this.hoverCircles.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "circle")
      );
    }

    setAttributes(this.hoverLine, {
      stroke: "white",
      class: "hidden",
      "stroke-width": "2px",
    });
    this.hoverCircles.forEach((circle) => {
      setAttributes(circle, {
        stroke: "white",
        class: "hidden",
        "stroke-width": "2px",
        r: "3",
      });
    });

    this.svg.appendChild(this.hoverLine);
    this.hoverCircles.forEach((circle) => this.svg.appendChild(circle));

    this.svg.addEventListener("mousemove", (e) => {
      const pointIndex = this.getClosestPointIndex(e.offsetX);
      const screenX = this.getClosestPointScreenSpaceX(pointIndex);
      const screenY = this.getClosestPointScreenSpaceYAverage(pointIndex);

      const scaling = Math.floor(this.valueArray[0].length / this.width);

      this.hoverInfo.updateInformation(
        this.approximatedValues,
        pointIndex,
        this.times[scaling * pointIndex],
        this.names,
        screenX,
        screenY,
        this.width,
        this.height
      );

      setAttributes(this.hoverLine, {
        x1: `${screenX}`,
        y1: "0",
        x2: `${screenX}`,
        y2: "600",
      });
      this.hoverCircles.forEach((circle, index) =>
        setAttributes(circle, {
          cx: `${screenX}`,
          cy: `${this.getClosestPointScreenSpaceY(index, pointIndex)}`,
        })
      );
    });

    this.svg.addEventListener("mouseenter", (_) => {
      this.hoverLine.setAttribute("class", "");
      this.hoverCircles.forEach((circle) => circle.setAttribute("class", ""));
      this.hoverInfo.show();
    });

    this.svg.addEventListener("mouseleave", (_) => {
      this.hoverLine.setAttribute("class", "hidden");
      this.hoverCircles.forEach((circle) =>
        circle.setAttribute("class", "hidden")
      );
      this.hoverInfo.hide();
    });
  }

  getTopElement() {
    return this.topElement;
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

    this.approximatedValues = this.valueArray.map((arr) => {
      const scaling = Math.floor(arr.length / this.width);
      if (scaling === 0) {
        return arr;
      } else {
        const result = [];
        for (let i = 0; i < arr.length / scaling; i++) {
          result.push(
            arr
              .slice(i * scaling, (i + 1) * scaling)
              .reduce((l, r) => Math.max(l, r))
          );
        }
        return result;
      }
    });

    this.verticalPadding = 0.05; // 5%
    this.paddingSpace = this.height * this.verticalPadding;
    this.paddingRoom = this.paddingSpace * 2;
    this.paddedHeight = this.height - this.paddingRoom;

    const [min, max] = this.approximatedValues
      .map((values) => this.getMinMax(values))
      .reduce(([lmin, lmax], [rmin, rmax]) => [
        Math.min(lmin, rmin),
        Math.max(lmax, rmax),
      ]);

    this.valueMin = min;
    this.valueMax = max;

    this.approximatedValues.forEach((values, index) => {
      if (values.length === 0) {
        return 0;
      }

      this.horizontalScaling = this.width / (values.length - 1);
      const pointsAttribValue = values
        .map(
          (val, i) =>
            `${i * this.horizontalScaling}, ${this.toScreenSpaceHeight(val)}`
        )
        .join(" ");

      this.polylines[index].setAttribute("points", pointsAttribValue);

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
    });
  }

  getClosestPointIndex(x) {
    const i = Math.floor(x / this.horizontalScaling);

    if (i >= this.approximatedValues[0].length) {
      return this.approximatedValues[0].length - 1;
    }

    const dist = [i, i + 1].map((v) =>
      Math.abs(x - v * this.horizontalScaling)
    );
    return dist[0] < dist[1] ? i : i + 1;
  }

  getClosestPointScreenSpaceX(pointIndex) {
    return pointIndex * this.horizontalScaling;
  }

  getClosestPointScreenSpaceY(valueIndex, pointIndex) {
    return this.toScreenSpaceHeight(
      this.approximatedValues[valueIndex][pointIndex]
    );
  }

  getClosestPointScreenSpaceYAverage(pointIndex) {
    const numerator = this.approximatedValues
      .map((val) => {
        return this.toScreenSpaceHeight(val[pointIndex]);
      })
      .reduce((l, r) => l + r);

    return numerator / this.approximatedValues.length;
  }
}

const insertDiv = document.getElementById("global_insert_space");

function createLineGraphForContainer(containers, timeContainer, title) {
  const elements = [];
  const names = [];
  containers.forEach((c) => elements.push(c.elements));
  containers.forEach((c) => names.push(c.name));
  const graph = new LineGraph(elements, timeContainer.elements, names, title);
  wrapSvgAndAppendToGlobalContainer(insertDiv, graph.getTopElement());
  return graph;
}

let containers = undefined;
let graphs = [];

window.onload = async () => {
  containers = await fetchAndParseTGPH();

  const timeContainer = containers.filter(
    (c) => c.name === "Unix timestamp"
  )[0];

  const co2TimeContainer = containers.filter(
    (c) => c.name === "Unix timestamp CO2"
  )[0];


  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("CO2 Concentration [ppm]")),
      co2TimeContainer,
      "Air quality"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("Interface enp1s0")),
      timeContainer,
      "Network usage"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("memory")),
      timeContainer,
      "RAM usage"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.startsWith("coretemp Core")),
      timeContainer,
      "CPU Temperature"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter(
        (c) => c.name.startsWith("CPU") && c.name.includes("Usage")
      ),
      timeContainer,
      "CPU Usage"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("mmcblk0")),
      timeContainer,
      "Internal disk usage"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("sda")),
      timeContainer,
      "Disk [sda] usage"
    )
  );
  graphs.push(
    createLineGraphForContainer(
      containers.filter((c) => c.name.includes("sdb")),
      timeContainer,
      "Disk [sdb] usage"
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
