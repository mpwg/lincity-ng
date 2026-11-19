shader_type spatial;
render_mode unshaded, cull_disabled, shadows_disabled;

varying float age;

void vertex() {
	float phase = INSTANCE_CUSTOM.r;
	age = fract(TIME * 0.25 + phase);
	vec3 center = MODEL_MATRIX[3].xyz;
	center += vec3(0.54 * age * age, age * 1.25, 0.12 * sin(age * 4.0 + phase));
	vec4 eye = VIEW_MATRIX * vec4(center, 1.0);
	eye.xy += VERTEX.xy * (0.17 + age * 0.49);
	POSITION = PROJECTION_MATRIX * eye;
}

void fragment() {
	float radius = length((UV - 0.5) * 2.0);
	ALBEDO = vec3(0.49, 0.48, 0.45);
	ALPHA = (1.0 - smoothstep(0.20, 1.0, radius)) * smoothstep(0.0, 0.15, age) * (1.0 - age) * 0.33;
}
