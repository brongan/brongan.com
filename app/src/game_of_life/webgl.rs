use super::universe::DomBounds;
use crate::game_of_life::universe::Universe;
use crate::game_of_life::universe::UniverseRenderer;
use crate::game_of_life::Timer;
use crate::mandelbrot::Bounds;
use crate::point2d::Point2D;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext as GL, WebGlShader};

pub struct WebGLRenderer {
    canvas: HtmlCanvasElement,
    program: WebGlProgram,
    width: u32,
    height: u32,
    texture: Vec<u8>,
    gl: GL,
    buffer: web_sys::WebGlBuffer,
}

impl UniverseRenderer for WebGLRenderer {
    fn get_cell_index(bounding_rect: DomBounds, bounds: Bounds, p: Point2D<i32>) -> (u32, u32) {
        let scale_x = bounds.width as f64 / bounding_rect.width;
        let scale_y = bounds.height as f64 / bounding_rect.height;

        let canvas_left = (p.x as f64 - bounding_rect.origin.x) * scale_x;
        let canvas_top = (p.y as f64 - bounding_rect.origin.y) * scale_y;

        let row = canvas_top.floor().min((bounds.height - 1) as f64);
        let col = canvas_left.floor().min((bounds.width - 1) as f64);
        (bounds.height - 1 - row as u32, col as u32)
    }

    fn render(&mut self, universe: &Universe) {
        let gl = self.gl.clone();

        // Ensure the viewport matches the drawing buffer size
        gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );

        let _timer = Timer::new("Rendering Frame");
        self.update_texture(universe);

        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));

        gl.use_program(Some(&self.program));

        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        gl.draw_arrays(GL::TRIANGLE_STRIP, 0, 4);
    }
}

fn update_universe_image<'a>(image: &'a mut Vec<u8>, universe: &'_ &Universe) -> &'a [u8] {
    for i in 0..((universe.width() * universe.height()) as usize) {
        if universe.is_alive(i) {
            image[i] = 0;
        } else {
            image[i] = 255;
        }
    }
    image.as_slice()
}

impl WebGLRenderer {
    const CELL_SIZE: u32 = 8;
    const _GRID_COLOR: &'static str = "#CCCCCC";
    const _DEAD_COLOR: &'static str = "#FFFFFF";
    const _ALIVE_COLOR: &'static str = "#000000";

    pub fn update_texture(&mut self, universe: &Universe) {
        let level = 0;
        let internal_format = GL::LUMINANCE;
        let border = 0;
        let src_format = GL::LUMINANCE;
        let src_type = GL::UNSIGNED_BYTE;
        let width = self.width;
        let height = self.height;
        let pixel = update_universe_image(&mut self.texture, &universe);
        assert!(pixel.len() == (width * height) as usize);
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                GL::TEXTURE_2D,
                level,
                internal_format as i32,
                width as i32,
                height as i32,
                border,
                src_format,
                src_type,
                Some(pixel),
            )
            .ok();

        if (self.width & (self.width - 1)) == 0 && (self.height & (self.height - 1)) == 0 {
            self.gl.generate_mipmap(GL::TEXTURE_2D);
        } else {
            self.gl
                .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
            self.gl
                .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
            self.gl
                .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        }

        // Change magnification filter to nearest neighbor to prevent fuzzies
        self.gl
            .tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
    }
}

impl WebGLRenderer {
    pub fn new(canvas: HtmlCanvasElement, width: u32, height: u32) -> WebGLRenderer {
        canvas.set_height((WebGLRenderer::CELL_SIZE + 1) * height + 1);
        canvas.set_width((WebGLRenderer::CELL_SIZE + 1) * width + 1);

        let size = (width * height) as usize;

        let gl = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();

        let vert_code = include_str!("./basic.vert");
        let frag_code = include_str!("./basic.frag");

        let vert_shader = compile_shader(&gl, GL::VERTEX_SHADER, vert_code).unwrap();
        let frag_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, frag_code).unwrap();

        let program = link_program(&gl, &vert_shader, &frag_shader).unwrap();
        gl.use_program(Some(&program));
        let pitch_loc = gl.get_uniform_location(&program, "pitch");
        gl.uniform2fv_with_f32_array(
            pitch_loc.as_ref(),
            &[
                (WebGLRenderer::CELL_SIZE + 1) as f32,
                (WebGLRenderer::CELL_SIZE + 1) as f32,
            ],
        );

        let vpw_loc = gl.get_uniform_location(&program, "vpw");
        gl.uniform1f(vpw_loc.as_ref(), canvas.width() as f32);
        let vph_loc = gl.get_uniform_location(&program, "vph");
        gl.uniform1f(vph_loc.as_ref(), canvas.height() as f32);
        let usampler_loc = gl.get_uniform_location(&program, "uSampler");

        // Tell WebGL we want to affect texture unit 0
        gl.active_texture(GL::TEXTURE0);

        // Tell the shader we bound the texture to texture unit 0
        gl.uniform1i(usampler_loc.as_ref(), 0);

        // Create and bind the texture to texture unit 0
        let webgl_texture = gl.create_texture();
        gl.bind_texture(GL::TEXTURE_2D, webgl_texture.as_ref());

        // Create and upload vertex buffer once
        let vertices: [f32; 12] = [
            1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0,
        ];
        let buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        // Safety: We are creating a raw view into WASM memory (`Float32Array::view`).
        // This view is valid ONLY as long as the WASM memory buffer does not grow.
        // We perform no allocations between creating the view and using it to populate
        // the WebGL buffer, so this is safe.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);
        }

        WebGLRenderer {
            canvas,
            program,
            width,
            height,
            texture: vec![0; size],
            gl,
            buffer,
        }
    }
}

pub fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    log::info!("Compiling Shader");
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
