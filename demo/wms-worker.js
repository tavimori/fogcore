import init, { FogMap, FogRenderer, GpuFogRenderer, lng_to_tile_x, lat_to_tile_y } from "../pkg/fogcore.js";

let fogMap, fogRenderer, gpuFogRenderer;

const TILE_WIDTH = 1024;

async function loadFile(filename) {
    const response = await fetch(`tiles/${filename}`);
    const arrayBuffer = await response.arrayBuffer();
    return new Uint8Array(arrayBuffer);
}

async function initializeFogMap() {
    await init();
    let fogMap = new FogMap();
    
    const files = [
        '2573lljsijod',    
        '33c1lljorhmz',
        '3fc9lorjsrwm',
        '5158lljorlmi',
        'cab3lotijkdk',
    ];

    for (const file of files) {
        try {
            const data = await loadFile(file);
            fogMap.add_fow_file(file, data);
            console.log(`Loaded ${file}`);
        } catch (error) {
            console.error(`Failed to load ${file}:`, error);
        }
    }

    return fogMap;
}

export async function initializeWorker() {
    return navigator.serviceWorker.register('wms-worker.js', { type: 'module' });
}

self.addEventListener('install', event => {
    event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', event => {
    event.waitUntil((async () => {
        await self.clients.claim();
        fogMap = await initializeFogMap();
        fogRenderer = new FogRenderer();
        console.log(`current fog renderer: ${fogRenderer}`);
        gpuFogRenderer = await GpuFogRenderer.create(1024, 1024);
    })());
});

self.addEventListener('fetch', event => {
    const url = new URL(event.request.url);
    if (url.pathname.startsWith('/custom-tile/')) {
        event.respondWith(generateCustomTile(url));
    }
});

async function generateCustomTile(url) {
    const [, , z, x, y] = url.pathname.split('/');
    const canvas = new OffscreenCanvas(TILE_WIDTH, TILE_WIDTH);
    const ctx = canvas.getContext('2d');

    // Set background color with 50% opacity
    ctx.fillStyle = 'rgba(200, 200, 200, 0.5)';
    ctx.fillRect(0, 0, TILE_WIDTH, TILE_WIDTH);

    // Render the fog image
    let cpuPngData = fogRenderer.render_image_raw(fogMap, x, y, z);

    // Create an ImageData object from the PNG data
    let uint8Array = new Uint8ClampedArray(cpuPngData);
    let imageData = new ImageData(uint8Array, TILE_WIDTH, TILE_WIDTH);

    // // Draw the fog image onto the canvas
    ctx.putImageData(imageData, 0, 0);

    // Add border
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.5)';
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, TILE_WIDTH, TILE_WIDTH);

    // Write tile parameters
    ctx.fillStyle = 'black';
    ctx.font = '96px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(`Tile: ${x}, ${y}`, 512, 356);
    ctx.fillText(`Zoom: ${z}`, 512, 600);

    // Convert canvas to blob
    const blob = await canvas.convertToBlob({type: 'image/png'});
    return new Response(blob, {
        headers: {'Content-Type': 'image/png'}
    });
}