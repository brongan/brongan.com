
use crate::Timer;
use crate::{UniverseRenderer, Universe};

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext as GL, WebGlShader};


pub struct WebGLRenderer {
    canvas: HtmlCanvasElement,
    program: WebGlProgram,
    width: u32,
    height: u32,
    texture: Vec<u8>,
}

impl UniverseRenderer for WebGLRenderer {
    fn get_cell_index(&self, x: u32, y: u32) -> (u32, u32) {
        let bounding_rect = self.canvas.get_bounding_client_rect();
        let scale_x = self.canvas.width() as f64 / bounding_rect.width();
        let scale_y = self.canvas.height() as f64 / bounding_rect.height();

        let canvas_left = (x as f64 - bounding_rect.left()) * scale_x;
        let canvas_top = (y as f64 - bounding_rect.top()) * scale_y;

        let row = (canvas_top / (WebGLRenderer::CELL_SIZE + 1) as f64)
            .floor()
            .min((self.height - 1) as f64);
        let col = (canvas_left / (WebGLRenderer::CELL_SIZE + 1) as f64)
            .floor()
            .min((self.width - 1) as f64);
        (self.height - 1 - row as u32, col as u32)
    }

    fn render(&mut self, universe: &Universe) {
        let gl = self
            .canvas
            .get_context("webgl").unwrap()
            .unwrap()
            .dyn_into::<GL>().unwrap();

        let _timer = Timer::new("Rendering Frame");
        self.update_texture(&gl, universe);

        let vertices: [f32; 12] = [
            1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0,
        ];

        let buffer = gl.create_buffer().ok_or("failed to create buffer").unwrap();
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

        gl.use_program(Some(&self.program));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);

            gl.buffer_data_with_array_buffer_view(
                GL::ARRAY_BUFFER,
                &vert_array,
                GL::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(GL::COLOR_BUFFER_BIT);

        gl.draw_arrays(
            GL::TRIANGLE_STRIP,
            0,
            (vertices.len() / 3) as i32,
        );
    }
}

fn update_universe_image<'a>(image: &'a mut Vec<u8>, universe: &'_ &Universe) -> &'a [u8] {
    for elem in image.iter_mut() {
        *elem = 0;
    }

    for i in 0..((universe.width * universe.height) as usize) {
        if universe.is_alive(i) {
        } else {
            image[4 * i] = 255;
            image[4 * i + 1] = 255;
            image[4 * i + 2] = 255;
        }
        image[4 * i + 3] = 255;
    }
    image.as_slice()
}

impl WebGLRenderer {
    const CELL_SIZE: u32 = 8;
    const _GRID_COLOR: &'static str = "#CCCCCC";
    const _DEAD_COLOR: &'static str = "#FFFFFF";
    const _ALIVE_COLOR: &'static str = "#000000";

    pub fn update_texture(&mut self, gl: &GL, universe: &Universe) {
        let level = 0;
        let internal_format = GL::RGBA;
        let border = 0;
        let src_format = GL::RGBA;
        let src_type = GL::UNSIGNED_BYTE;
        let width = self.width;
        let height = self.height;
        let pixel = update_universe_image(&mut self.texture, &universe);
        assert!(pixel.len() == (width * height * 4) as usize);
        gl
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
            gl.generate_mipmap(GL::TEXTURE_2D);
        } else {
            gl.tex_parameteri(
                GL::TEXTURE_2D,
                GL::TEXTURE_WRAP_S,
                GL::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameteri(
                GL::TEXTURE_2D,
                GL::TEXTURE_WRAP_T,
                GL::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameteri(
                GL::TEXTURE_2D,
                GL::TEXTURE_MIN_FILTER,
                GL::LINEAR as i32,
            );
        }
    }
}

impl WebGLRenderer {
    pub fn new(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Option<WebGLRenderer> {
        canvas.set_height((WebGLRenderer::CELL_SIZE + 1) * height + 1);
        canvas.set_width((WebGLRenderer::CELL_SIZE + 1) * width + 1);

        let size = (width * height) as usize;
        let texture: Vec<u8> = vec![0; size * 4];

        let gl = canvas
            .get_context("webgl").unwrap()
            .unwrap()
            .dyn_into::<GL>().unwrap();

        let vert_code = include_str!("./basic.vert");
        let frag_code = include_str!("./basic.frag");

        let vert_shader = compile_shader(
            &gl,
            GL::VERTEX_SHADER,
            vert_code).unwrap();

        let frag_shader = compile_shader(
            &gl,
            GL::FRAGMENT_SHADER,
            frag_code).unwrap();

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
        let offset_loc = gl.get_uniform_location(&program, "offset");
        gl.uniform2fv_with_f32_array(offset_loc.as_ref(), &[1.0, 1.0]);
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

        Some(WebGLRenderer {
            canvas,
            program,
            width,
            height,
            texture,
        })
    }
}

pub fn compile_shader(
    gl: &GL,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
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

