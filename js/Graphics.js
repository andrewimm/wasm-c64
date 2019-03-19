(function() {

const COLORS = [
  [0x00, 0x00, 0x00],
  [0xff, 0xff, 0xff],
  [0x68, 0x37, 0x2b],
  [0x70, 0xa4, 0xb2],
  [0x6f, 0x3d, 0x86],
  [0x58, 0x8d, 0x43],
  [0x35, 0x28, 0x79],
  [0xb8, 0xc7, 0x6f],
  [0x6f, 0x4f, 0x25],
  [0x43, 0x39, 0x00],
  [0x9a, 0x67, 0x59],
  [0x44, 0x44, 0x44],
  [0x6c, 0x6c, 0x6c],
  [0x9a, 0xd2, 0x84],
  [0x6c, 0x5e, 0xb5],
  [0x95, 0x95, 0x95],
];

const textVertexShaderSource = `#version 300 es
in vec4 a_position;
out vec2 v_texcoord;

uniform vec2 u_offset;

void main() {
  gl_Position = a_position * 2.0 - vec4(1, 1, 0, 1);
  v_texcoord = vec2(a_position.x * 384. - 32., (1.0 - a_position.y) * 272. - 36.);
}
`;

const textFragmentShaderSource = `#version 300 es
precision highp float;
precision highp usampler2D;

in vec2 v_texcoord;
uniform int u_mode;
uniform int u_framecolor;
uniform int u_bgcolor;
uniform int u_bgcolor_2;
uniform int u_bgcolor_3;
uniform float u_charmem_length;
uniform usampler2D u_charmem;
uniform usampler2D u_screenmem;
uniform usampler2D u_colormem;
uniform usampler2D u_colors;

out vec4 outColor;

void main() {
  int index = u_bgcolor;
  vec2 tile = vec2(floor(v_texcoord.x / 8.), floor(v_texcoord.y / 8.));
  vec2 coords = vec2(v_texcoord.x / 8. / 40., v_texcoord.y / 8. / 25.);
  uint charindex = texture(u_screenmem, coords).r;
  int offsetx = 7 - int(v_texcoord.x - (floor(v_texcoord.x / 8.0) * 8.0));
  float offsety = (v_texcoord.y - (floor(v_texcoord.y / 8.0) * 8.0)) / 8.0;
  vec2 memcoord = vec2(offsety, (float(charindex) + 0.5) / u_charmem_length);
  uint line = texture(u_charmem, memcoord).r;
  float tileIndex = tile.y * 40. + tile.x;
  if (u_mode == 0) {
    if (((line >> offsetx) & 1u) > 0u) {
      index = int(texture(u_colormem, vec2(tileIndex / 1024., 0.5)).r);
    }
  } else if (u_mode == 1) {
    uint color = (line >> ((offsetx / 2) * 2)) & 3u;
    if (color == 1u) {
      index = u_bgcolor_2;
    } else if (color == 2u) {
      index = u_bgcolor_3;
    } else if (color == 3u) {
      index = int(texture(u_colormem, vec2(tileIndex / 1024., 0.5)).r);
    }
  }
  if (v_texcoord.x < 0. || v_texcoord.y < 0. || v_texcoord.x > 320. || v_texcoord.y > 200.) {
    index = u_framecolor;
  }
  uvec4 color = texture(u_colors, vec2(0, float(index) / 16.));

  outColor = vec4(
    float(color.r) / 255.0,
    float(color.g) / 255.0,
    float(color.b) / 255.0,
    1.0
  );
}
`;

function createShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    return shader;
  }
  console.error(gl.getShaderInfoLog(shader));
  gl.deleteShader(shader);
}

function createProgram(gl, vertexShader, fragmentShader) {
  const program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  if (gl.getProgramParameter(program, gl.LINK_STATUS)) {
    return program;
  }
  console.error(gl.getProgramInfoLog(program));
  gl.deleteProgram(program);
}

class Graphics {
  constructor(gl) {
    this.gl = gl;
    const textProgram = createProgram(
      gl,
      createShader(gl, gl.VERTEX_SHADER, textVertexShaderSource),
      createShader(gl, gl.FRAGMENT_SHADER, textFragmentShaderSource),
    );
    this.text = {
      program: textProgram,
      attributes: {
        position: gl.getAttribLocation(textProgram, 'a_position'),
      },
      uniforms: {
        charMemLength: gl.getUniformLocation(textProgram, 'u_charmem_length'),
        colors: gl.getUniformLocation(textProgram, 'u_colors'),
        offset: gl.getUniformLocation(textProgram, 'u_offset'),
        charMem: gl.getUniformLocation(textProgram, 'u_charmem'),
        screenMem: gl.getUniformLocation(textProgram, 'u_screenmem'),
        colorMem: gl.getUniformLocation(textProgram, 'u_colormem'),
        frameColor: gl.getUniformLocation(textProgram, 'u_framecolor'),
        bgColor: gl.getUniformLocation(textProgram, 'u_bgcolor'),
        bgColor2: gl.getUniformLocation(textProgram, 'u_bgcolor_2'),
        bgColor3: gl.getUniformLocation(textProgram, 'u_bgcolor_3'),
        mode: gl.getUniformLocation(textProgram, 'u_mode'),
      },
    };

    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
      0, 0, 0, 1, 1, 0,
      1, 0, 0, 1, 1, 1,
    ]), gl.STATIC_DRAW);

    this.vao = gl.createVertexArray();
    gl.bindVertexArray(this.vao);
    gl.enableVertexAttribArray(this.text.attributes.position);
    gl.vertexAttribPointer(this.text.attributes.position, 2, gl.FLOAT, false, 0, 0);
    this.charMemTexture = gl.createTexture();
    this.screenMemTexture = gl.createTexture();
    this.colorMemTexture = gl.createTexture();
    this.colorTexture = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0 + 0);
    gl.bindTexture(gl.TEXTURE_2D, this.charMemTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.activeTexture(gl.TEXTURE0 + 2);
    gl.bindTexture(gl.TEXTURE_2D, this.screenMemTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
    gl.activeTexture(gl.TEXTURE0 + 4);
    gl.bindTexture(gl.TEXTURE_2D, this.colorTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.activeTexture(gl.TEXTURE0 + 6);
    gl.bindTexture(gl.TEXTURE_2D, this.colorMemTexture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    this.initColorTexture();

    this._frameColor = 14;
    this._bgColor = 6;
    this._bgColor2 = 0;
    this._bgColor3 = 0;
    this._mode = 0;
  }

  initColorTexture() {
    const data = new Uint8Array(new ArrayBuffer(3 * 16));
    for (let i = 0; i < COLORS.length; i++) {
      data[i * 3 + 0] = COLORS[i][0];
      data[i * 3 + 1] = COLORS[i][1];
      data[i * 3 + 2] = COLORS[i][2];
    }
    gl.activeTexture(gl.TEXTURE0 + 4);
    gl.bindTexture(gl.TEXTURE_2D, this.colorTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB8UI, 1, 16, 0, gl.RGB_INTEGER, gl.UNSIGNED_BYTE, data);
  }

  loadCharMem(data) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + 0);
    gl.bindTexture(gl.TEXTURE_2D, this.charMemTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8UI, 8, 256, 0, gl.RED_INTEGER, gl.UNSIGNED_BYTE, data);
  }

  loadScreenMem(data) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + 2);
    gl.bindTexture(gl.TEXTURE_2D, this.screenMemTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8UI, 40, 25, 0, gl.RED_INTEGER, gl.UNSIGNED_BYTE, data);
  }

  loadColorMem(data) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + 6);
    gl.bindTexture(gl.TEXTURE_2D, this.colorMemTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8UI, 1024, 1, 0, gl.RED_INTEGER, gl.UNSIGNED_BYTE, data);
  }

  setColors(border, bg, bg2, bg3) {
    this._frameColor = border;
    this._bgColor = bg;
    this._bgColor2 = bg2;
    this._bgColor3 = bg3;
  }

  setMode(mode) {
    this._mode = mode;
  }

  draw() {
    gl.viewport(0, 0, 384, 272);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.useProgram(this.text.program);
    gl.bindVertexArray(this.vao);
    gl.uniform1f(this.text.uniforms.charMemLength, 256);
    gl.uniform1i(this.text.uniforms.charMem, 0);
    gl.uniform1i(this.text.uniforms.screenMem, 2);
    gl.uniform1i(this.text.uniforms.colors, 4);
    gl.uniform1i(this.text.uniforms.colorMem, 6);
    gl.uniform2f(this.text.uniforms.offset, 0, 0);

    gl.uniform1i(this.text.uniforms.frameColor, this._frameColor);
    gl.uniform1i(this.text.uniforms.bgColor, this._bgColor);
    gl.uniform1i(this.text.uniforms.bgColor2, this._bgColor2);
    gl.uniform1i(this.text.uniforms.bgColor3, this._bgColor3);

    gl.uniform1i(this.text.uniforms.mode, this._mode);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }
}

window.Graphics = Graphics;
})();