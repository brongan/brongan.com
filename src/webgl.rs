use crate::Timer;
use crate::{UniverseRenderer, Universe};

use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

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
        let context = self
            .canvas
            .get_context("webgl").unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().unwrap();

        let _timer = Timer::new("Rendering Frame");
        self.update_texture(&context, universe);

        let vertices: [f32; 12] = [
            1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0,
        ];

        let buffer = context.create_buffer().ok_or("failed to create buffer").unwrap();
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

        context.use_program(Some(&self.program));

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

            context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        context.enable_vertex_attrib_array(0);

        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        context.draw_arrays(
            WebGlRenderingContext::TRIANGLE_STRIP,
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

    pub fn update_texture(&mut self, context: &WebGlRenderingContext, universe: &Universe) {
        let level = 0;
        let internal_format = WebGlRenderingContext::RGBA;
        let border = 0;
        let src_format = WebGlRenderingContext::RGBA;
        let src_type = WebGlRenderingContext::UNSIGNED_BYTE;
        let width = self.width;
        let height = self.height;
        let pixel = update_universe_image(&mut self.texture, &universe);
        assert!(pixel.len() == (width * height * 4) as usize);
        context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGlRenderingContext::TEXTURE_2D,
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
            context.generate_mipmap(WebGlRenderingContext::TEXTURE_2D);
        } else {
            context.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_WRAP_S,
                WebGlRenderingContext::CLAMP_TO_EDGE as i32,
            );
            context.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_WRAP_T,
                WebGlRenderingContext::CLAMP_TO_EDGE as i32,
            );
            context.tex_parameteri(
                WebGlRenderingContext::TEXTURE_2D,
                WebGlRenderingContext::TEXTURE_MIN_FILTER,
                WebGlRenderingContext::LINEAR as i32,
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

        let context = canvas
            .get_context("webgl").unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().unwrap();

        let vert_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            attribute vec4 aVertexPosition;

            void main(void) {
              gl_Position = aVertexPosition;
            }
    "#,
        ).unwrap();

        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
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
    "#,
        ).unwrap();

        let program = link_program(&context, &vert_shader, &frag_shader).unwrap();
        context.use_program(Some(&program));
        let pitch_loc = context.get_uniform_location(&program, "pitch");
        context.uniform2fv_with_f32_array(
            pitch_loc.as_ref(),
            &[
                (WebGLRenderer::CELL_SIZE + 1) as f32,
                (WebGLRenderer::CELL_SIZE + 1) as f32,
            ],
        );
        let offset_loc = context.get_uniform_location(&program, "offset");
        context.uniform2fv_with_f32_array(offset_loc.as_ref(), &[1.0, 1.0]);
        let vpw_loc = context.get_uniform_location(&program, "vpw");
        context.uniform1f(vpw_loc.as_ref(), canvas.width() as f32);
        let vph_loc = context.get_uniform_location(&program, "vph");
        context.uniform1f(vph_loc.as_ref(), canvas.height() as f32);
        let usampler_loc = context.get_uniform_location(&program, "uSampler");

        // Tell WebGL we want to affect texture unit 0
        context.active_texture(WebGlRenderingContext::TEXTURE0);

        // Tell the shader we bound the texture to texture unit 0
        context.uniform1i(usampler_loc.as_ref(), 0);

        // Create and bind the texture to texture unit 0
        let webgl_texture = context.create_texture();
        context.bind_texture(WebGlRenderingContext::TEXTURE_2D, webgl_texture.as_ref());

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
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    log::info!("Compiling Shader");
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
