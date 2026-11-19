shader_type spatial;
render_mode unshaded, cull_disabled, shadows_disabled;
uniform sampler2D leaf_spray : filter_linear_mipmap, repeat_disable;
uniform bool has_atlas = false;
uniform sampler2D atlas : source_color, filter_linear_mipmap_anisotropic;
varying float height;
varying float exposure;

void vertex() {
	height = -(MODEL_MATRIX * vec4(VERTEX,1.0)).y;
	vec3 n = normalize(mat3(MODEL_MATRIX) * NORMAL);
	n.y *= -1.0;
	exposure = 0.61 + 0.33 * max(0.0,dot(n, normalize(vec3(-0.44,0.77,0.46))));
}

void fragment() {
	// Lots have integral ground plates. Keep the reflected elevations above them.
	if (height < 0.045) discard;
	vec3 base = COLOR.rgb;
	if (abs(UV2.x - 11.0) < .5) {
		vec4 leaf = texture(leaf_spray,UV);
		if (leaf.a < .34) discard;
		base *= .59 + leaf.r*.66;
	}
	if (has_atlas) base *= texture(atlas,UV).rgb;
	ALBEDO = base*exposure;
}
