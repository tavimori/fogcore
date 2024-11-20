export class TrackLayer {
    constructor(map, fogMap) {
        this.map = map;
        this.fogMap = fogMap;
        this.id = 'custom-layer';
        this.type = 'custom';
        this.renderingMode = '2d';
    }

    onAdd(map, gl) {
        // create GLSL source for vertex shader
        const vertexSource = `
        precision mediump float;
        uniform mat4 u_matrix;
        attribute vec2 a_pos;
        void main() {
            gl_Position = u_matrix * vec4(a_pos, 0.0, 1.0);
            gl_PointSize = 5.0;
        }`;

        // create GLSL source for fragment shader
        const fragmentSource = `
        precision mediump float;
        void main() {
            vec2 center = gl_PointCoord - vec2(0.5);
            float dist = length(center);
            if (dist > 0.5) {
                discard;
            } else {
                gl_FragColor = vec4(1.0);
            }
        }`;

        // Create shader program
        this.program = createShaderProgram(gl, vertexSource, fragmentSource);
        this.aPos = gl.getAttribLocation(this.program, 'a_pos');
        this.buffer = gl.createBuffer();

        // Create mask texture and framebuffer
        this.setupMaskRendering(gl);

        // Setup final composite rendering
        this.setupCompositeRendering(gl);
    }

    render(gl, matrix) {
        // First pass: render points to mask
        this.renderPointsToMask(gl, matrix);

        // Second pass: render final composite
        this.renderFinalComposite(gl);
    }

    // Helper methods
    setupMaskRendering(gl) {
        this.maskTexture = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);
        gl.texImage2D(
            gl.TEXTURE_2D, 0, gl.RGBA,
            gl.canvas.width, gl.canvas.height,
            0, gl.RGBA, gl.UNSIGNED_BYTE, null
        );
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

        this.maskFramebuffer = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.maskFramebuffer);
        gl.framebufferTexture2D(
            gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
            gl.TEXTURE_2D, this.maskTexture, 0
        );
    }

    setupCompositeRendering(gl) {
        const finalVertexSource = `
        precision mediump float;
        attribute vec2 a_position;
        varying vec2 v_texCoord;
        void main() {
            gl_Position = vec4(a_position, 0.0, 1.0);
            v_texCoord = a_position * 0.5 + 0.5;
        }`;

        const finalFragmentSource = `
        precision mediump float;
        uniform sampler2D u_texture;
        varying vec2 v_texCoord;
        void main() {
            vec4 mask = texture2D(u_texture, v_texCoord);
            gl_FragColor = vec4(0.0, 0.0, 0.0, 0.5 * (1.0 - mask.r));
        }`;

        this.finalProgram = createShaderProgram(gl, finalVertexSource, finalFragmentSource);
        this.finalPositionAttribute = gl.getAttribLocation(this.finalProgram, 'a_position');

        this.quadBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuffer);
        gl.bufferData(
            gl.ARRAY_BUFFER,
            new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]),
            gl.STATIC_DRAW
        );
    }

    renderPointsToMask(gl, matrix) {
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.maskFramebuffer);
        gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);

        const bounds = this.map.getBounds();
        const sw = mapboxgl.MercatorCoordinate.fromLngLat(bounds.getSouthWest(), 0);
        const ne = mapboxgl.MercatorCoordinate.fromLngLat(bounds.getNorthEast(), 0);
        const pixels = this.fogMap.get_bounding_mercator_pixels(sw.x, sw.y, ne.x, ne.y);

        const numPoints = pixels.length / 2;
        if (numPoints === 0) return;

        gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(pixels), gl.DYNAMIC_DRAW);
        gl.useProgram(this.program);
        gl.uniformMatrix4fv(
            gl.getUniformLocation(this.program, 'u_matrix'),
            false,
            matrix
        );
        gl.enableVertexAttribArray(this.aPos);
        gl.vertexAttribPointer(this.aPos, 2, gl.FLOAT, false, 0, 0);
        gl.drawArrays(gl.POINTS, 0, numPoints);
    }

    renderFinalComposite(gl) {
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
        gl.useProgram(this.finalProgram);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
        gl.bindTexture(gl.TEXTURE_2D, this.maskTexture);
        gl.bindBuffer(gl.ARRAY_BUFFER, this.quadBuffer);
        gl.enableVertexAttribArray(this.finalPositionAttribute);
        gl.vertexAttribPointer(this.finalPositionAttribute, 2, gl.FLOAT, false, 0, 0);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    }
}

function createShaderProgram(gl, vertexSource, fragmentSource) {
    const vertexShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertexShader, vertexSource);
    gl.compileShader(vertexShader);
    if (!gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS)) {
        console.error('Vertex shader compilation failed:', gl.getShaderInfoLog(vertexShader));
        return null;
    }

    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fragmentShader, fragmentSource);
    gl.compileShader(fragmentShader);
    if (!gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS)) {
        console.error('Fragment shader compilation failed:', gl.getShaderInfoLog(fragmentShader));
        return null;
    }

    const program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error('Program linking failed:', gl.getProgramInfoLog(program));
        return null;
    }

    return program;
}
