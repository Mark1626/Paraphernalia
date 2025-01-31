// Voronoi-based Road Generation
const GRID_SIZE = 500; // Increased for better SVG visibility
const NUM_POINTS = 10; // Number of city centers
const CITY_RADIUS = 5;
const ROAD_WIDTH = 2;

// Generate random points
function generatePoints() {
    const points = [];
    for (let i = 0; i < NUM_POINTS; i++) {
        points.push([
            Math.floor(Math.random() * GRID_SIZE),
            Math.floor(Math.random() * GRID_SIZE)
        ]);
    }
    return points;
}

// Create SVG path for a road
function createRoadPath(p1, p2) {
    return `M ${p1[0]} ${p1[1]} L ${p2[0]} ${p2[1]}`;
}

// Draw roads for a triangle
function drawTriangleRoads(p1, p2, p3) {
    return [
        createRoadPath(p1, p2),
        createRoadPath(p2, p3),
        createRoadPath(p3, p1)
    ];
}

// Generate roads using Delaunay triangulation
function generateRoads() {
    const points = generatePoints();
    const triangles = Delaunator.from(points.map(p => [p[0], p[1]]));
    
    // Create SVG content
    const svgPaths = [];
    const cityCenters = [];
    
    // Add city centers
    for (let i = 0; i < points.length; i++) {
        cityCenters.push(`<circle cx="${points[i][0]}" cy="${points[i][1]}" r="${CITY_RADIUS}" fill="red"/>`);
    }
    
    // Draw roads between connected points
    for (let i = 0; i < triangles.triangles.length; i += 3) {
        const p1 = points[triangles.triangles[i]];
        const p2 = points[triangles.triangles[i + 1]];
        const p3 = points[triangles.triangles[i + 2]];
        
        const roads = drawTriangleRoads(p1, p2, p3);
        svgPaths.push(...roads);
    }
    
    // Combine all paths into a single SVG path element
    const roadPaths = `<path d="${svgPaths.join(' ')}" stroke="black" stroke-width="${ROAD_WIDTH}" fill="none"/>`;
    
    // Create the final SVG
    const svg = `<svg width="${GRID_SIZE}" height="${GRID_SIZE}" xmlns="http://www.w3.org/2000/svg">
        <rect width="100%" height="100%" fill="#f0f0f0"/>
        ${roadPaths}
        ${cityCenters.join('\n        ')}
    </svg>`;
    
    return svg;
}

// Generate and output SVG
const svg = generateRoads();
console.log(svg);