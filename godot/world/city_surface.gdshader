shader_type spatial;
render_mode cull_back, diffuse_burley, specular_schlick_ggx;

uniform sampler2D leaf_spray : filter_linear_mipmap, repeat_disable;
uniform sampler2D water_mask : filter_linear, repeat_disable;
uniform bool reflection_active = false;
uniform sampler2D wave_slopes : filter_linear_mipmap_anisotropic, repeat_enable;
uniform sampler2D reflection_map : filter_linear, repeat_disable;
uniform float map_width = 1.0;
varying vec3 world_pos;
uniform bool bridge = false;
uniform bool use_atlas = false;
uniform sampler2D atlas : source_color, filter_linear_mipmap_anisotropic, repeat_enable;
uniform bool overlay_active = false;
uniform sampler2D overlay_map : filter_nearest, repeat_disable;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	if (bridge) {
		float coverage = textureLod(water_mask, world_pos.xz / map_width, 0.0).r;
		// Lower the deck where it meets the bank; foliage and terrain keep their height.
		float ramp = 1.0 - smoothstep(0.50, 0.85, coverage);
		VERTEX.y -= min(0.092, max(0.0, VERTEX.y) * 0.83) * ramp;
		world_pos.y = (MODEL_MATRIX * vec4(VERTEX, 1.0)).y;
	}
}

float hash2(vec2 p) {
	return fract(sin(dot(p, vec2(23.17, 61.39))) * 4192.71);
}

float noise2(vec2 p) {
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	vec2 i = floor(p);
	return mix(mix(hash2(i), hash2(i + vec2(1, 0)), f.x),
		mix(hash2(i + vec2(0, 1)), hash2(i + vec2(1)), f.x), f.y);
}

float rect(vec2 p, vec2 half_size, vec2 aa) {
	vec2 edge = 1.0 - smoothstep(half_size - aa, half_size + aa, abs(p));
	return edge.x * edge.y;
}

void fragment() {
	vec3 color = COLOR.rgb;
	int style = int(UV2.x + 0.5);
	if (use_atlas) {
		vec3 sample_color = texture(atlas, UV).rgb;
		color *= OUTPUT_IS_SRGB ? sample_color : pow(sample_color, vec3(1.0 / 2.2));
	}
	ROUGHNESS = 0.89;
	if (style == 1 || style == 2 || style == 7) {
		vec2 brick = UV * vec2(13.0, 27.0);
		brick.x += mod(floor(brick.y), 2.0) * 0.5;
		float brick_face = rect(fract(brick) - 0.5, vec2(0.46, 0.43), min(fwidth(brick), vec2(0.49)));
		if (style != 1) {
			color *= mix(1.13, 0.94 + 0.13 * hash2(floor(brick)), brick_face);
		}
		if (style != 7) {
			vec2 bays = UV * vec2(style == 1 ? 3.55 : 2.7, style == 1 ? 3.15 : 2.22);
			vec2 p = fract(bays) - 0.5;
			vec2 aa = clamp(fwidth(bays) * 0.55, vec2(0.002), vec2(0.45));
			vec2 dims = style == 1 ? vec2(0.215, 0.285) : vec2(0.33, 0.31);
			float rim = rect(p, dims + vec2(0.045, 0.039), aa);
			float pane = rect(p, dims, aa);
			float bars = (1.0 - smoothstep(0.012, 0.032 + aa.x, abs(p.x))) +
				(1.0 - smoothstep(0.012, 0.032 + aa.y, abs(p.y - 0.07)));
			float reflection = hash2(floor(bays) + floor(world_pos.xz) * 0.07);
			vec3 glass = mix(vec3(0.16, 0.24, 0.25), vec3(0.45, 0.54, 0.54), reflection * 0.82 + p.y * 0.2);
			color = mix(color, style == 1 ? vec3(0.89, 0.82, 0.69) : vec3(0.30, 0.30, 0.27), rim);
			color = mix(color, mix(glass, vec3(0.68, 0.65, 0.54), clamp(bars, 0.0, 1.0) * 0.78), pane);
		}
	} else if (style == 3) {
		vec2 shingles = UV * vec2(15.0, 18.0);
		shingles.x += floor(shingles.y) * 0.5;
		float detail = 1.0 - smoothstep(0.25, 0.95, max(fwidth(shingles.x), fwidth(shingles.y)));
		color *= 1.0 + detail * (hash2(floor(shingles)) * 0.10 - 0.045 - 0.15 * step(0.83, fract(shingles.y)));
	} else if (style == 4) {
		float patches = noise2(world_pos.xz * 0.47) * 0.66 + noise2(world_pos.xz * 2.4) * 0.34;
		float sand = texture(water_mask,world_pos.xz / map_width).g;
		color = mix(vec3(0.45,0.51,0.32),vec3(0.62,0.62,0.38),patches);
		color = mix(color,vec3(0.70,0.66,0.48),sand*(0.55+patches*.18));
	}
	if (style == 4 || style == 5) {
		// Bilinear coverage rounds the corners of the simulation's shoreline.
		float shore = texture(water_mask, world_pos.xz / map_width).r;
		float irregularity = noise2(world_pos.xz * 3.3);
		shore += (irregularity - 0.5) * 0.13;
		vec3 dry = style == 5 ? vec3(0.52,0.55,0.36) : color;
		vec3 shallows = vec3(0.32,0.39,0.31);
		vec3 deep = vec3(0.22,0.31,0.32);
		float silt = noise2(world_pos.xz * 12.1);
		dry = mix(dry,vec3(0.47,0.45,0.35)*(0.86+0.25*silt),smoothstep(0.27,0.54,shore));
		vec3 water = mix(shallows,deep,smoothstep(0.48,0.90,shore));
		float wet = smoothstep(0.475,0.52,shore);
		if (wet > 0.0) {
			vec2 p = world_pos.xz;
			vec2 swell = texture(wave_slopes, p*.28+TIME*vec2(.016,.009)).rg*2.-1.;
			mat2 cross_wind = mat2(vec2(.80,.60),vec2(-.60,.80));
			vec2 capillary = cross_wind* (texture(wave_slopes,cross_wind*p*.73-TIME*vec2(.012,.023)).rg*2.-1.);
			vec2 tilt = .68*swell + .32*capillary;
			float fresnel = pow(1.0-abs(normalize(CAMERA_POSITION_WORLD-world_pos).y),2.1);
			vec3 sky = vec3(.47,.54,.56);
			water = mix(water,sky,.18+fresnel*.35);
			if (reflection_active) {
				vec2 distortion = tilt * vec2(.0045,.006);
				vec4 reflected = texture(reflection_map,SCREEN_UV+distortion);
				float distance_fade = 1.0-smoothstep(44.0,57.0,distance(CAMERA_POSITION_WORLD,world_pos));
				water = mix(water,reflected.rgb*vec3(.80,.87,.88),reflected.a*(.66+fresnel*.28)*distance_fade);
			}
			// Tiny wave faces reflect sunlight over a limited range of view directions.
			vec3 to_eye = normalize(CAMERA_POSITION_WORLD-world_pos);
			vec3 half_light = normalize(to_eye + normalize(vec3(-.49,.77,.39)));
			vec3 slope = normalize(vec3(tilt.x*.61,1.0,tilt.y*.62));
			float glint = pow(max(dot(slope,half_light),0.0),112.0);
			water += vec3(.51,.49,.43) * glint;
			water += vec3(.042,.049,.051) * tilt.y;
		}
		color = mix(dry,water,wet);
		ROUGHNESS = mix(.94,.36,wet);
	} else if (style == 11) {
		vec4 leaves = texture(leaf_spray, UV);
		if (leaves.a < 0.34) discard;
		color *= 0.59 + leaves.r * 0.66;
	} else if (style == 9 || style == 10) {
		float grain = noise2(world_pos.xz * 38.0);
		float fade = 1.0 - smoothstep(0.012, 0.14, length(fwidth(world_pos.xz)));
		color *= 1.0 + (grain - 0.5) * (style == 9 ? 0.23 : 0.35) * fade;
	} else if (style == 8) {
		float leaves = noise2(world_pos.xz * 12.0 + world_pos.y * vec2(4.9, 6.2));
		float flecks = noise2(world_pos.xz*39.0+world_pos.y*vec2(11.7,29.4));
		color *= mix(vec3(0.82,0.88,0.82),vec3(1.12,1.08,1.01),leaves);
		color *= 0.94+0.14*flecks;
	} else if (style == 6) {
		vec2 stones = world_pos.xz * vec2(6.0, 10.0);
		stones.x += 0.5 * mod(floor(stones.y), 2.0);
		color *= mix(0.92, 1.04, rect(fract(stones) - 0.5, vec2(0.45), min(fwidth(stones), vec2(0.49))));
	}
	if (overlay_active) {
		vec4 overlay_color = texture(overlay_map, world_pos.xz / map_width);
		float luminance = dot(color, vec3(0.299, 0.587, 0.114));
		color = mix(vec3(luminance), color, 0.3);
		color = mix(color, overlay_color.rgb, overlay_color.a * 0.88);
	}
	ALBEDO = OUTPUT_IS_SRGB ? color * 0.64 : pow(max(color, vec3(0.0)), vec3(2.2));
}
