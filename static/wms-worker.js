import init, { FogMap, lng_to_tile_x, lat_to_tile_y } from "../pkg/fogcore.js";

let fogMap;

const TILE_WIDTH = 512;

async function loadFile(filename) {
    const response = await fetch(`tiles/${filename}`);
    const arrayBuffer = await response.arrayBuffer();
    return new Uint8Array(arrayBuffer);
}

async function initializeFogMap() {
    await init();
    let fogMap = await FogMap.new();

    const files = [
        '0091rktirom',
        '3652llhsjrdm',
        '564allwokhkz',
        '81callwstjnd',
        'b271lllswoxe',
        'e013loiiijod',
        '024dloriklki',
        '371bikohtzn',
        '56c7llwskrkm',
        '81e0ihissww',
        'b2abllwstlni',
        'e136lotiwlxi',
        '0380lorjsiwo',
        '3741rrwjode',
        '5828rrwwixo',
        '83bdlorjjldi',
        'b5b0lookjode',
        'e179loiirimo',
        '05c4lotijsdw',
        '38f0rsttjnd',
        '5951lorithnz',
        '8597rtrhtzn',
        'b615loijttnn',
        'e43crrrshwz',
        '08e8loritlni',
        '3a24lorjjkdk',
        '596fiksjldi',
        '87darkrowex',
        'b707llhsskwk',
        'e555itskjkd',
        '0a68llhsshwz',
        '3a45lotiwixo',
        '5adalotjwlxi',
        '87f7lljorjmd',
        'b8c8ihjjjdd',
        'e5e3lotjwhxz',
        '0a82lotiwhxz',
        '3a96iliwhxz',
        '5b0eiiklkik',
        '8882lotiwwxx',
        'b982llhsjtdn',
        'e782rrrswwx',
        '0de2loijrhmz',
        '3af8iksjhdz',
        '5bfartrhrzm',
        '8a0ellhowtxn',
        'bc3clotjwjxd',
        'e9b2rsttrnm',
        '0fd2isoosew',
        '3befloritone',
        '5c66lotiwrxm',
        '8b71lotihizo',
        'bc87rshkwkx',
        'eb5bllosooee',
        '10bditskkkk',
        '3cd0lorjsowe',
        '5cc8lorjjtdn',
        '8cc8lllolkik',
        'bddditosjwd',
        'eb8eisslsiw',
        '10e9rkhrkmk',
        '3d75iliwoxe',
        '5cccllosohez',
        '8e24lotjhoze',
        'bf39llhowkxk',
        'ed74loijrjmd',
        '120dirorome',
        '3f13lllolrim',
        '5d5ersttino',
        '8e7clljsiwox',
        'c03alorjwixo',
        'edbbijjihoz',
        '1310loijrlmi',
        '3f6ailjltin',
        '5f57lollsjwd',
        '8ee7riwktkn',
        'c115lollswwx',
        'eebdllhsswwx',
        '13a4lhrhthnz',
        '3fc6iilojed',
        '6045llhowsxw',
        '8ff3itoslwi',
        'c146iksjode',
        'f170rrrssww',
        '1532lotiwoxe',
        '3fc9lorjsrwm',
        '60e6lookjldi',
        '9804lorjkoke',
        'c150rshklki',
        'f194llosowex',
        '16fcloijrome',
        '3fd0lowwkkkk',
        '6113rrrjjdd',
        '980fiwirome',
        'c179loriktkn',
        'f249llhsjido',
        '191floijrwmx',
        '4124iljlkik',
        '62falllolsiw',
        '9ba9llhojldi',
        'c3a4llwokoke',
        'f39blorjjode',
        '1b15lotjhlzi',
        '41ccitskwkx',
        '6ae6lotijode',
        '9bf2lotijhdz',
        'c49allosolei',
        'f3c8rshkoke',
        '1b81lllshszw',
        '41d1ikswixo',
        '6b64llljwhxz',
        '9c6bllosokek',
        'c4b3lotiwtxn',
        'f4acllhojode',
        '1c5bllwskiko',
        '429alotiwkxk',
        '6dacloijtino',
        '9f8elokihoze',
        'c524lotjwwxx',
        'f4f3isoojed',
        '1c5drkhiooe',
        '4374llwsthnz',
        '6e5cllwoklki',
        '9f96lorjstwn',
        'c559rsttwnx',
        'f7a2loijtknk',
        '1c70lotjhhzz',
        '4406iiklsiw',
        '6e7bloijtrnm',
        'a017loolhtzn',
        'cab3lotijkdk',
        'f7d6llwosiwo',
        '1d81lorikrkm',
        '44d7lookwixo',
        '6f03lokjorem',
        'a441lllshtzn',
        'cd7elotjliio',
        'f851lljorome',
        '1f7arshkhkz',
        '45bflljsisow',
        '70bfioiojed',
        'a7a6lotjhrzm',
        'd023lllshkzk',
        'f87dlllshwzx',
        '1fe1ijjiwox',
        '48dcrtwloie',
        '7380lllshjzd',
        'a8a6lotiwsxw',
        'd358lotijldi',
        'f8c4lljorsmw',
        '2434iowihoz',
        '48dfllwokwkx',
        '738dllwstwnx',
        'a9aflllollii',
        'd4bflorikjkd',
        'f9aertwlsiw',
        '24beloiirrmm',
        '49darirsiwo',
        '7459rirtino',
        'aa72iowiloi',
        'd5adllhsslwi',
        'fa15lorikiko',
        '253aiwjktkn',
        '4a29llosojed',
        '772cllwstone',
        'aba8rjtsowe',
        'd612itsktkn',
        'fc20lorjshwz',
        '2573lljsijod',
        '4b7ciliwlxi',
        '792ciljlrim',
        'acc4lorjslwi',
        'd6b6lotijtdn',
        'fc72lorjjido',
        '2689rkhrsmw',
        '4befloiiihoz',
        '79aalllolhiz',
        'ade9ihiskwk',
        'd818iowtlni',
        'fc85itoswwx',
        '26b1isslkik',
        '4fb7llojiioo',
        '79e3lorikoke',
        'ae68loiiiooe',
        'd836lotiwjxd',
        'fcb3irotino',
        '2bb9rirklki',
        '5158lljorlmi',
        '7a72llhowrxm',
        'ae93iwjkrkm',
        'd94aikohrzm',
        'febdirsihoz',
        '2dd3lotjhizo',
        '541elljorwmx',
        '7b20lojtsjwd',
        'afedlokihhzz',
        'dc13lllshizo',
        'ff73lokihlzi',
        '2ecerkhiloi',
        '543bllloltin',
        '7c1alllolwix',
        'b0a4rrrsjwd',
        'dd48itoshwz',
        '33c1lljorhmz',
        '545florjjrdm',
        '806clotjwoxe',
        'b21blotijwdx',
        'dd79lllshrzm',
        '35c5iikljid',
        '54abiljliio',
        '80abrirkoke',
        'b25clojhwixo',
        'dd96lotjlrim',
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
    const time = new URL(url).searchParams.get('t');
    const canvas = new OffscreenCanvas(TILE_WIDTH, TILE_WIDTH);
    const ctx = canvas.getContext('2d');

    // Set background color with 50% opacity
    ctx.fillStyle = 'rgba(200, 200, 200, 0.5)';
    ctx.fillRect(0, 0, TILE_WIDTH, TILE_WIDTH);

    // Render the fog image
    let imageBufferRaw = await fogMap.render_image(x, y, z);

    // Create an ImageData object from the PNG data
    let uint8Array = new Uint8ClampedArray(imageBufferRaw);
    let imageData = new ImageData(uint8Array, TILE_WIDTH, TILE_WIDTH);

    // Draw the fog image onto the canvas
    ctx.putImageData(imageData, 0, 0);

    // Add border
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.5)';
    ctx.lineWidth = 2;
    ctx.strokeRect(0, 0, TILE_WIDTH, TILE_WIDTH);

    // Write tile parameters and time
    ctx.fillStyle = 'black';
    ctx.font = '36px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(`TileX: ${x}`, TILE_WIDTH / 2, 100);
    ctx.fillText(`TileY: ${y}`, TILE_WIDTH / 2, 150);
    ctx.fillText(`Zoom: ${z}`, TILE_WIDTH / 2, 200);
    // ctx.fillText(`Time: ${time}`, 512, 484);

    // Convert canvas to blob
    const blob = await canvas.convertToBlob({ type: 'image/png' });
    return new Response(blob, {
        headers: { 'Content-Type': 'image/png' }
    });
}