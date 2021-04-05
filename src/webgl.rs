use crate::log;
use crate::{Universe, WebGLRenderer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

impl WebGLRenderer {
    const CELL_SIZE: u32 = 5;
    const _GRID_COLOR: &'static str = "#CCCCCC";
    const _DEAD_COLOR: &'static str = "#FFFFFF";
    const _ALIVE_COLOR: &'static str = "#000000";
}

#[wasm_bindgen]
impl WebGLRenderer {
    pub fn new(canvas: HtmlCanvasElement, universe: Universe) -> WebGLRenderer {
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        canvas.set_height((WebGLRenderer::CELL_SIZE + 1) * universe.height + 1);
        canvas.set_width((WebGLRenderer::CELL_SIZE + 1) * universe.width + 1);

        WebGLRenderer { canvas, universe }
    }

    pub fn render(&self) -> Result<(), JsValue> {
        log!(
            "Rendering Universe ({}, {}) with WebGL!",
            self.universe.width,
            self.universe.height
        );

        let context = self
            .canvas
            .get_context("webgl")?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()?;

        let vert_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            attribute vec4 aVertexPosition;

            void main(void) {
              gl_Position = aVertexPosition;
            }
    "#,
        )?;
        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
            precision mediump float;

            uniform float vpw; // Width, in pixels
            uniform float vph; // Height, in pixels

            uniform vec2 offset;
            uniform vec2 pitch;

            void main() {
              float lX = gl_FragCoord.x / vpw;
              float lY = gl_FragCoord.y / vph;

              float scaleFactor = 10000.0;

              float offX = (scaleFactor * offset[0]) + gl_FragCoord.x;
              float offY = (scaleFactor * offset[1]) + (1.0 - gl_FragCoord.y);

              if (int(mod(offX, pitch[0])) == 0 ||
                  int(mod(offY, pitch[1])) == 0) {
                gl_FragColor = vec4(0.0, 0.0, 0.0, 0.5);
              } else {
                gl_FragColor = vec4(0.5, 0.0, 0.0, 1.0);
              }
            }
    "#,
        )?;

        let program = link_program(&context, &vert_shader, &frag_shader)?;
        context.use_program(Some(&program));
        let pitch_loc = context.get_uniform_location(&program, "pitch");
        context.uniform2fv_with_f32_array(pitch_loc.as_ref(), &[50.0, 50.0]);
        let offset_loc = context.get_uniform_location(&program, "offset");
        context.uniform2fv_with_f32_array(offset_loc.as_ref(), &[0.0, 1.0]);
        let vpw_loc = context.get_uniform_location(&program, "vpw");
        context.uniform1f(vpw_loc.as_ref(), self.canvas.width() as f32);
        let vph_loc = context.get_uniform_location(&program, "vph");
        context.uniform1f(vph_loc.as_ref(), self.canvas.height() as f32);

        let vertices: [f32; 12] = [
            1.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, -1.0, -1.0, 0.0,
        ];

        let buffer = context.create_buffer().ok_or("failed to create buffer")?;
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

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
        Ok(())
    }
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
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
