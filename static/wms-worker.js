self.addEventListener('install', event => {
    event.waitUntil(self.skipWaiting());
});

self.addEventListener('activate', event => {
    event.waitUntil(self.clients.claim());
});

self.addEventListener('fetch', event => {
    const url = new URL(event.request.url);
    if (url.pathname.startsWith('/custom-tile/')) {
        event.respondWith(generateCustomTile(url));
    }
});

async function generateCustomTile(url) {
    const [, , z, x, y] = url.pathname.split('/');
    const canvas = new OffscreenCanvas(256, 256);
    const ctx = canvas.getContext('2d');

    // Set background color with 50% opacity
    ctx.fillStyle = 'rgba(200, 200, 200, 0.5)';
    ctx.fillRect(0, 0, 256, 256);

    // Add border
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.5)';
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, 256, 256);

    // Write tile parameters
    ctx.fillStyle = 'black';
    ctx.font = '16px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(`Tile: ${x}, ${y}`, 128, 100);
    ctx.fillText(`Zoom: ${z}`, 128, 130);

    // Convert canvas to blob
    const blob = await canvas.convertToBlob({type: 'image/png'});
    return new Response(blob, {
        headers: {'Content-Type': 'image/png'}
    });
}