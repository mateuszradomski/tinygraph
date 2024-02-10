const TGPH_FORMAT_MAGIC = 0x48504754;
const TGPH_FORMAT_VERSION = 1;
const SVG_HTML_NAMESPACE = "http://www.w3.org/2000/svg";

const SHORT_MONTH_NAMES = [
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

async function getContainerData() {
  const response = await fetch("data.tgph.gz", { cache: "no-store" });
  const blob = await decompressToByteArray(await response.blob());
  return blob.buffer;
}

class TGPHFormatDecoder {
  constructor(bytes) {
    this.bytes = bytes;
    this.dataView = new DataView(bytes);
    this.offset = 0;
  }

  readF32() {
    const res = this.dataView.getFloat32(this.offset, true);
    this.offset += 4;
    return res;
  }

  readU32() {
    const res = this.dataView.getUint32(this.offset, true);
    this.offset += 4;
    return res;
  }

  readU16() {
    const res = this.dataView.getUint16(this.offset, true);
    this.offset += 2;
    return res;
  }

  readU8() {
    const res = this.dataView.getUint8(this.offset, true);
    this.offset += 1;
    return res;
  }

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
  }
}

function parseContainer(decoder) {
  const name = decoder.readString();
  const elementType = decoder.readU8();
  const elementCount = decoder.readU32();

  const elements = [];
  switch (elementType) {
    case 1:
      for (let j = 0; j < elementCount; j++) {
        elements.push(decoder.readU32());
      }
      break;
    case 2:
      for (let j = 0; j < elementCount; j++) {
        elements.push(decoder.readF32());
      }
      break;
    case 3:
      for (let j = 0; j < elementCount; j++) {
        elements.push(decoder.readString());
      }
      break;
    default:
      throw new Error(`Unexpected elementType = ${elementType}`);
  }

  return {
    name,
    type: elementType,
    elements,
  };
}

function parseTGPH(bytes) {
  const decoder = new TGPHFormatDecoder(bytes);

  const magic = decoder.readU32();
  const version = decoder.readU8();

  if (magic !== TGPH_FORMAT_MAGIC) {
    throw new Error("Invalid magic at the start of fetched file");
  }
  if (version !== TGPH_FORMAT_VERSION) {
    throw new Error("Unexpected version in the fetched TGPH file");
  }

  let containers = [];
  const containerCount = decoder.readU16();
  for (let i = 0; i < containerCount; i++) {
    containers.push(parseContainer(decoder));
  }

  return containers;
}

function unpackContainers(containers) {
  return {
    elements: containers.map((c) => c.elements),
    names: containers.map((c) => c.name),
  };
}

function getContainersNamedLike(name) {
  return containers.filter((c) => c.name.includes(name));
}

function getContainerNamedExactly(name) {
  const matching = containers.filter((c) => c.name === name);
  if (matching.length !== 1) {
    throw new Error(`Expected to find exactly one container with name ${name}`);
  }
  return matching[0];
}

function hashString(string) {
  let hash = 0xcafebabe;
  for (let i = 0; i < string.length; i++) {
    hash = (hash * 33) ^ string.charCodeAt(i);
  }
  return hash + 0xcafebabe;
}

function generateColorFromString(name) {
  const stringAsANumber = hashString(name);
  return `hsl(${stringAsANumber % 360.0}, 100%, 65%)`;
}

function setAttributes(elem, attrs) {
  for (const key in attrs) {
    elem.setAttribute(key, attrs[key]);
  }
}

function wrapSvgAndAppendToGlobalContainer(svg) {
  const div = document.createElement("div");
  div.setAttribute("class", "graph");

  div.appendChild(svg);

  const insertDiv = document.getElementById("global_insert_space");
  insertDiv.appendChild(div);
}

function convertRemToPixels(rem) {
  return rem * parseFloat(getComputedStyle(document.documentElement).fontSize);
}

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
      "line-height: 1.25rem; color: #F8F8FA;",
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
    parentHeight,
  ) {
    this.setPosition(x, y, parentWidth, parentHeight);
    this.createValueParagraphs(dataArrays, pointIndex, names);
    this.valueParagraphs.textContent = dataArrays[0][pointIndex];
    const date = new Date(timestamp * 1000);
    const yy = date.getFullYear();
    const mm = SHORT_MONTH_NAMES[date.getMonth()];
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
      p.setAttribute(
        "style",
        "line-height: 1.25rem; color: #F8F8FA; text-align: left;",
      );
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
      `${verticalStyle}; ${horizontalStyle}; border-radius: 1rem; background: #424850; z-index: 50; min-height: 5rem; min-width: 10rem; position: absolute; text-align: right; padding-left: 1rem; display: flex; justify-content: center; flex-direction: column; padding-right: 0.6rem;`,
    );
  }

  hide() {
    this.topElement.setAttribute("class", "hidden");
  }
}

class TitleAndLegend {
  constructor(titleText, legendeNames) {
    this.text = titleText;
    this.legendeNames = legendeNames;
    this.legendeColors = legendeNames.map((name) =>
      generateColorFromString(name),
    );

    this.textElement = document.createElement("span");
    this.textElement.setAttribute("style", "font: 2rem serif; color: #F8F8FA;");
    this.textElement.textContent = titleText;

    this.topElement = document.createElement("div");
    this.topElement.setAttribute(
      "style",
      "display: flex; justify-content: space-between; height: 3rem;",
    );
    this.spanDiv = document.createElement("div");
    this.spanDiv.setAttribute(
      "style",
      "text-align: right; padding-right: 0.5rem; flex-shrink: 0;",
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
    this.verticalPaddingPercentage = 0.05;

    if (valueArray.every((arr) => arr.length !== valueArray[0].length)) {
      throw new Error("All arrays must have the same length");
    }

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
    this.seriesArray = valueArray;
    this.names = names;

    this.rulers = [];
    this.rulerCaptions = [];
    for (let i = 0; i < 5; i++) {
      this.rulers.push(document.createElementNS(SVG_HTML_NAMESPACE, "line"));
      this.rulerCaptions.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "text"),
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
    for (let i = 0; i < this.seriesArray.length; i++) {
      this.polylines.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "polyline"),
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
    for (let i = 0; i < this.seriesArray.length; i++) {
      this.hoverCircles.push(
        document.createElementNS(SVG_HTML_NAMESPACE, "circle"),
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

      const scaling = Math.floor(this.seriesArray[0].length / this.width);

      this.hoverInfo.updateInformation(
        this.compressedSeriesArray,
        pointIndex,
        this.times[scaling * pointIndex],
        this.names,
        screenX,
        screenY,
        this.width,
        this.height,
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
        }),
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
        circle.setAttribute("class", "hidden"),
      );
      this.hoverInfo.hide();
    });
  }

  getClosestPointIndex(x) {
    const i = Math.floor(x / this.horizontalScaling);

    if (i >= this.compressedSeriesArray[0].length) {
      return this.compressedSeriesArray[0].length - 1;
    }

    const dist = [i, i + 1].map((v) =>
      Math.abs(x - v * this.horizontalScaling),
    );
    return dist[0] < dist[1] ? i : i + 1;
  }

  getClosestPointScreenSpaceX(pointIndex) {
    return pointIndex * this.horizontalScaling;
  }

  getClosestPointScreenSpaceYAverage(pointIndex) {
    const numerator = this.compressedSeriesArray
      .map((val) => val[pointIndex])
      .reduce((l, r) => l + r);

    return numerator / this.compressedSeriesArray.length;
  }

  getClosestPointScreenSpaceY(valueIndex, pointIndex) {
    return this.mappedSeriesArray[valueIndex][pointIndex];
  }

  updateRulers() {
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
  }

  updateRulerCaptions() {
    this.rulerCaptions.forEach((cap, i) => {
      const denom1 = this.rulerCaptions.length - 1;
      const y =
        (this.rulerCaptions.length - i - 1) * (this.paddedHeight / denom1) +
        this.paddingSpace;

      const denom2 =
        (this.valueMax - this.valueMin) / (this.rulerCaptions.length - 1);
      const rulerValue = i * denom2 + this.valueMin;
      cap.textContent = `${rulerValue.toFixed(2)}`;
      setAttributes(cap, {
        x: "0",
        y: `${y - 2}`,
      });
    });
  }

  getTopElement() {
    return this.topElement;
  }

  updateMinMaxOfGraph(seriesArray) {
    let max = Number.MIN_VALUE;
    let min = Number.MAX_VALUE;

    for (const series of seriesArray) {
      for (const v of series) {
        max = Math.max(max, v);
        min = Math.min(min, v);
      }
    }

    this.valueMin = min;
    this.valueMax = max;
  }

  mapRange(value, fromMin, fromMax, toMax) {
    const valueFraction = (value - fromMin) / (fromMax - fromMin);
    const result = toMax * valueFraction;
    const inverted = toMax - result;
    return inverted + this.paddingSpace;
  }

  handleResize() {
    const bbox = this.svg.getBoundingClientRect();
    this.width = bbox.width;
    this.height = bbox.height;
    this.paddingSpace = this.height * this.verticalPaddingPercentage;
    this.paddedHeight = this.height - this.paddingSpace * 2;

    this.compressedSeriesArray = this.seriesArray.map((series) =>
      this.compressSeries(series),
    );

    // This ia bad, because i required this being called before anything else
    this.updateMinMaxOfGraph(this.compressedSeriesArray);

    this.mappedSeriesArray = this.compressedSeriesArray.map((series) => {
      return series.map((value) => {
        const result = this.mapRange(
          value,
          this.valueMin,
          this.valueMax,
          this.paddedHeight,
        );
        return result;
      });
    });

    this.horizontalScaling = this.width / this.compressedSeriesArray[0].length;
  }

  compressSeries(series) {
    const datumsPerPixel = Math.floor(series.length / this.width);
    if (datumsPerPixel <= 1) {
      return series;
    }

    const result = [];
    for (let i = 0; i < series.length / datumsPerPixel; i++) {
      const windowPosition = i * datumsPerPixel;
      result.push(
        Math.max(
          ...series.slice(windowPosition, windowPosition + datumsPerPixel),
        ),
      );
    }
    return result;
  }

  updatePolylines() {
    this.mappedSeriesArray.forEach((values, index) => {
      if (values.length === 0) {
        return;
      }

      const pointsAttribValue = values
        .map((val, i) => {
          const x = i * this.horizontalScaling;
          const y = val;
          return `${x}, ${y}`;
        })
        .join(" ");

      this.polylines[index].setAttribute("points", pointsAttribValue);
    });
  }

  draw() {
    this.handleResize();

    this.updatePolylines();
    this.updateRulers();
    this.updateRulerCaptions();
  }
}

function createLineGraph(containers, timeContainer, title) {
  const { elements, names } = unpackContainers(containers);
  const graph = new LineGraph(elements, timeContainer.elements, names, title);
  wrapSvgAndAppendToGlobalContainer(graph.getTopElement());
  return graph;
}

let containers = undefined;
let graphs = [];

window.onload = async () => {
  const tgphBytes = await getContainerData();
  containers = parseTGPH(tgphBytes);

  const timeContainer = getContainerNamedExactly("Unix timestamp");
  const co2TimeContainer = getContainerNamedExactly("Unix timestamp CO2");

  const graphConfigurations = [
    {
      title: "Air quality",
      dataContainerNamePart: "CO2 Concentration [ppm]",
      timeContainer: co2TimeContainer,
    },
    {
      title: "Network usage",
      dataContainerNamePart: "Interface enp1s0",
      timeContainer: timeContainer,
    },
    {
      title: "RAM usage",
      dataContainerNamePart: "memory",
      timeContainer: timeContainer,
    },
    {
      title: "CPU Temperature",
      dataContainerNamePart: "coretemp Core",
      timeContainer: timeContainer,
    },
    {
      title: "CPU Usage",
      dataContainerNamePart: "CPU",
      timeContainer: timeContainer,
    },
    {
      title: "Internal disk usage",
      dataContainerNamePart: "mmcblk0",
      timeContainer: timeContainer,
    },
    {
      title: "Disk [sda] usage",
      dataContainerNamePart: "sda",
      timeContainer: timeContainer,
    },
    {
      title: "Disk [sdb] usage",
      dataContainerNamePart: "sdb",
      timeContainer: timeContainer,
    },
  ];

  graphs = graphConfigurations.map((config) =>
    createLineGraph(
      getContainersNamedLike(config.dataContainerNamePart),
      config.timeContainer,
      config.title,
    ),
  );

  graphs.forEach((g) => g.draw());
};

window.addEventListener("resize", (_) => {
  if (graphs === undefined) {
    return;
  }

  graphs.forEach((g) => g.draw());
});
