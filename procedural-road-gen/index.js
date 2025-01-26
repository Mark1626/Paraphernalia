import Delaunator from 'delaunator';

// Voronoi-based Road Generation
const GRID_SIZE = 50;
const NUM_POINTS = 15; // Number of city centers
const ROAD_CHAR = 'x';

// Create empty grid
const grid = Array(GRID_SIZE).fill().map(() => Array(GRID_SIZE).fill('0'));

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

// Draw line using Bresenham's algorithm
function drawLine(x0, y0, x1, y1) {
    let dx = Math.abs(x1 - x0);
    let dy = Math.abs(y1 - y0);
    let sx = (x0 < x1) ? 1 : -1;
    let sy = (y0 < y1) ? 1 : -1;
    let err = dx - dy;

    while (true) {
        if (x0 >= 0 && x0 < GRID_SIZE && y0 >= 0 && y0 < GRID_SIZE) {
            grid[y0][x0] = ROAD_CHAR;
        }

        if (x0 === x1 && y0 === y1) break;
        let e2 = 2 * err;
        if (e2 > -dy) {
            err -= dy;
            x0 += sx;
        }
        if (e2 < dx) {
            err += dx;
            y0 += sy;
        }
    }
}

// Generate roads using Delaunay triangulation
function generateRoads() {
    const points = generatePoints();
    
    // Create Delaunay triangulation
    const triangles = Delaunator.from(points.map(p => [p[0], p[1]]));
    console.log("Generating trianges")
    console.log(points)

    for (let i = 0; i < points.length; i++) {
        grid[points[i][1]][points[i][0]] = '●';
    }

    console.log(triangles)
    
    // Draw roads between connected points
    for (let i = 0; i < triangles.triangles.length; i += 3) {
        const p1 = points[triangles.triangles[i]];
        const p2 = points[triangles.triangles[i + 1]];
        const p3 = points[triangles.triangles[i + 2]];

        console.log(p1, p2, p3)
        
        // Draw roads between triangle vertices
        drawLine(p1[0], p1[1], p2[0], p2[1]);
        drawLine(p2[0], p2[1], p3[0], p3[1]);
        drawLine(p3[0], p3[1], p1[0], p1[1]);
    }
}

// Print the grid
function printGrid() {
    for (let y = 0; y < GRID_SIZE; y++) {
        console.log(grid[y].join(''));
    }
}

// Generate and print
generateRoads();
printGrid();