const svg = document.getElementById("main");

// Create a line element
var line = document.createElementNS("http://www.w3.org/2000/svg", "line");
line.setAttribute("x1", "100");
line.setAttribute("y1", "100");
line.setAttribute("x2", "300");
line.setAttribute("y2", "300");
line.setAttribute("stroke", "black");

// Append the line to the SVG element
svg.appendChild(line);
