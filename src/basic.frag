precision mediump float;

uniform float vpw; // Width, in pixels
uniform float vph; // Height, in pixels

uniform vec2 offset; // offsets are for nerds
uniform vec2 pitch; // idk like the cell size or something

uniform sampler2D uSampler; // give me cells

void main() {
	vec2 newCoord = vec2(vpw * gl_FragCoord.x, vph * gl_FragCoord.y);

	if (int(mod(newCoord.x, pitch[0])) == 0 || int(mod(newCoord.y, pitch[1])) == 0) {
		gl_FragColor = vec4(0.0, 0.0, 0.0, 0.5);
	} else {
		gl_FragColor = texture2D(uSampler, gl_FragCoord.xy / vec2(vpw, vph));
	}
}
