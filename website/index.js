const svg = document.getElementById("main");

const drawPolyline = (svg, points) => {
    const pointsAttribValue = points.map(([x, y]) => `${x}, ${y}`).join(" ")

    const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
    polyline.setAttribute("points", pointsAttribValue);
    polyline.setAttribute("stroke", "pink");
    polyline.setAttribute("fill", "none");
    svg.appendChild(polyline)
}

const scale = 8;
let points = [
    [scale * 0, 347],
    [scale * 1, 350],
    [scale * 2, 289],
    [scale * 3, 252],
    [scale * 4, 329],
    [scale * 5, 253],
    [scale * 6, 277],
    [scale * 7, 314],
    [scale * 8, 279],
    [scale * 9, 255],
    [scale * 10, 278],
    [scale * 11, 289],
    [scale * 12, 261],
    [scale * 13, 289],
    [scale * 14, 336],
    [scale * 15, 261],
    [scale * 16, 315],
    [scale * 17, 251],
    [scale * 18, 283],
    [scale * 19, 337],
    [scale * 20, 260],
    [scale * 21, 258],
    [scale * 22, 296],
    [scale * 23, 271],
    [scale * 24, 294],
    [scale * 25, 269],
    [scale * 26, 261],
    [scale * 27, 326],
    [scale * 28, 323],
    [scale * 29, 257],
    [scale * 30, 257],
    [scale * 31, 259],
    [scale * 32, 296],
    [scale * 33, 256],
    [scale * 34, 324],
    [scale * 35, 268],
    [scale * 36, 321],
    [scale * 37, 281],
    [scale * 38, 342],
    [scale * 39, 301],
    [scale * 40, 253],
    [scale * 41, 277],
    [scale * 42, 284],
    [scale * 43, 332],
    [scale * 44, 333],
    [scale * 45, 312],
    [scale * 46, 252],
    [scale * 47, 329],
    [scale * 48, 315],
    [scale * 49, 313],
    [scale * 50, 340],
    [scale * 51, 280],
    [scale * 52, 275],
    [scale * 53, 323],
    [scale * 54, 286],
    [scale * 55, 286],
    [scale * 56, 325],
    [scale * 57, 290],
    [scale * 58, 313],
    [scale * 59, 297],
    [scale * 60, 340],
    [scale * 61, 305],
    [scale * 62, 342],
    [scale * 63, 256],
    [scale * 64, 310],
    [scale * 65, 287],
    [scale * 66, 300],
    [scale * 67, 346],
    [scale * 68, 314],
    [scale * 69, 261],
    [scale * 70, 251],
    [scale * 71, 281],
    [scale * 72, 279],
    [scale * 73, 278],
    [scale * 74, 261],
    [scale * 75, 319],
    [scale * 76, 313],
    [scale * 77, 311],
    [scale * 78, 331],
    [scale * 79, 300],
    [scale * 80, 250],
    [scale * 81, 291],
    [scale * 82, 266],
    [scale * 83, 280],
    [scale * 84, 307],
    [scale * 85, 287],
    [scale * 86, 273],
    [scale * 87, 279],
    [scale * 88, 345],
    [scale * 89, 328],
    [scale * 90, 302],
    [scale * 91, 311],
    [scale * 92, 338],
    [scale * 93, 263],
    [scale * 94, 288],
    [scale * 95, 276],
    [scale * 96, 265],
    [scale * 97, 258],
    [scale * 98, 338],
    [scale * 99, 323],
]

function lerp(k0, k1, t) {
    return k0 + t * (k1 - k0)
}

const getInterpolatedY = (x, points) => {
    let i = 0;
    for (i = 0; i < points.length; i++) {
        if (points[i][0] > x) { break; }
    }

    if (i === 0) { return points[0][1]; }

    return lerp(points[i - 1][1], points[i][1], (x - points[i - 1][0]) / Math.abs(points[i][0] - points[i - 1][0]))
}

const chartHoverLine = document.createElementNS("http://www.w3.org/2000/svg", "line");
const chartHoverCircle = document.createElementNS("http://www.w3.org/2000/svg", "circle");

chartHoverLine.setAttribute("stroke", "grey");
chartHoverCircle.setAttribute("stroke", "grey");
chartHoverCircle.setAttribute("r", "3");

svg.appendChild(chartHoverLine)
svg.appendChild(chartHoverCircle)

svg.addEventListener("mousemove", (e) => {
    chartHoverLine.setAttribute("x1", `${e.offsetX}`)
    chartHoverLine.setAttribute("y1", "0")
    chartHoverLine.setAttribute("x2", `${e.offsetX}`)
    chartHoverLine.setAttribute("y2", "600")

    chartHoverCircle.setAttribute("cx", `${e.offsetX}`)
    chartHoverCircle.setAttribute("cy", `${getInterpolatedY(e.offsetX, points)}`)
})

svg.addEventListener("mouseenter", (_) => {
    chartHoverLine.setAttribute("class", "")
    chartHoverCircle.setAttribute("class", "")
})

svg.addEventListener("mouseleave", (_) => {
    chartHoverLine.setAttribute("class", "hidden")
    chartHoverCircle.setAttribute("class", "hidden")
})

drawPolyline(svg, points)
