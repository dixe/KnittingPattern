use gl_lib::imode_gui::Rect;
use gl_lib::{ sdl2,
              gl, helpers, na, buffer};
use gl_lib::imode_gui::drawer2d::*;
use gl_lib::imode_gui::ui::*;
use gl_lib::imode_gui::widgets::*;
use gl_lib::objects::{square, texture_quad};
use gl_lib::color::Color;
use gl_lib::typedef::{V2, V3};
use gl_lib::collision2d::polygon;
use gl_lib::imode_gui::style::BorderRadius;
mod pattern;
use pattern::*;

mod render_view;


fn main() -> Result<(), failure::Error> {
    let sdl_setup = helpers::setup_sdl()?;
    let window = sdl_setup.window;
    let sdl = sdl_setup.sdl;
    let viewport = sdl_setup.viewport;
    let gl = &sdl_setup.gl;

    let drawer_2d = Drawer2D::new(&gl, viewport).unwrap();
    let mut ui = Ui::new(drawer_2d);

    let mut event_pump = sdl.event_pump().unwrap();

    // Set background color to white


    let mut pattern = Pattern::default();

    let mut cols = 20;
    for r in 0..27 {
        pattern.add_row(cols);

        if r == 7 || r == 15 || r == 23 {
            cols += 1;
        }
    }


    // maybe easier to just you know add support for at transform to the render code
    // instead of doing the framebuffer stuff
    let mut framebuffer = buffer::FrameBuffer::new(&gl, &viewport);

    framebuffer.r = 0.0;
    framebuffer.g = 0.0;
    framebuffer.b = 0.0;
    framebuffer.a = 0.0;
    framebuffer.depth_test = false;

    let mut ctx = Context { pattern,
                            thick : 4,
                            start_x : 250,
                            start_y : 50,
                            grid_width : 20,
                            grid_height : 20,
                            base_color: Color::Rgb(30,30, 240),
                            color_1: Color::Rgb(250, 250, 250),
                            draw_grid: false,
                            mode: Mode::Render,
                            render_center : V2::new(800.0, 80.0),
                            framebuffer,
                            texture_square: texture_quad::TextureQuad::new(gl),
                            quad: square::Square::new(gl),
                            clear_color: Color::RgbAf32(0.9, 0.9, 0.9, 1.0),
                            grid: CellGrid::new(30, 30),
    };




    update_render_obj_data(&mut ctx);

    loop {

        // Basic clear gl stuff and get events to UI
        unsafe {
            // set clear color every frame, since bind and clear in framebuffer also sets clear color
            // and opengl is state full, so the clear color is not per frame buffer, but global
            let cc = ctx.clear_color.as_vec4();
            gl.ClearColor(cc.x, cc.y, cc.z, cc.w);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        ui.consume_events(&mut event_pump);

        for e in &ui.frame_events {
            match e {
                sdl2::event::Event::Window {win_event: sdl2::event::WindowEvent::Resized(w, h), .. } => {
                    ctx.framebuffer.update_viewport(&gl, &gl::viewport::Viewport {x: 0, y: 0, w: *w, h: *h });
                },
                _ => {}
            };
        }


        //grid_ui(&mut ctx, &mut ui);
        old_ui(&mut ctx, &mut ui);


        window.gl_swap_window();
    }
}


fn grid_ui(ctx: &mut Context, ui: &mut Ui) {

    let rect = Rect { x: ctx.start_x, y: ctx.start_y, w: 500, h: 700 };
    ui.cell_grid(rect, &mut ctx.grid, PaintType::Activate);



}

fn old_ui(ctx: &mut Context, ui: &mut Ui) {

    ui.body_text(&format!("{}", ui.fps()));
    ui.newline();


    render_view::view(ctx, ui);

    edit_view(ctx, ui);




    ui.newline();
    if ui.button("Reload shaders") {
        ui.drawer2D.reload_all_shaders();
    }

    ui.newline();


    ui.color_picker(&mut ctx.clear_color);
    ui.newline();
}

fn edit_view(ctx: &mut Context, ui: &mut Ui) {

    render_pattern(ctx, ui, true);
    edit_pattern(ctx,ui);
    ui.color_picker(&mut ctx.base_color);
    ui.newline();
    ui.color_picker(&mut ctx.color_1);
    ui.newline();
    ui.checkbox(&mut ctx.draw_grid);
    ui.newline();
    ui.slider(&mut ctx.grid_width, 5, 60);
    ui.newline();
    ui.slider(&mut ctx.grid_height, 5, 60);
    ui.newline();


    if ui.button("Render") {
        ctx.mode = Mode::Render
    }

    ui.newline();

    if ui.button("Rest") {
        let mut pattern = Pattern::default();

        let mut cols = 20;
        for r in 0..27 {
            pattern.add_row(cols);

            if r == 7 || r == 15 || r == 23 {
                cols += 1;
            }
        }

        ctx.pattern = pattern;
    }

    ui.newline();

}


// just flip on y axis
fn update_render_obj_data(ctx: &mut Context) {
    let pos_data: [f32; 12] = [
        // positions
        0.5,  0.5, 0.0, // tr
        -0.5,  0.5, 0.0, // tl
        -0.5, -0.5, 0.0,// bl
        0.5, -0.5, 0.0, // br
    ];

    let l = 0.0;
    let b = 0.0;
    let r = 1.0;
    let t = 1.0;

    let tex_data = [
        r, t,
        l, t,
        l, b,
        r, b
    ];


    // flip top and bottom coords to flip image on its head
    // update texture coords to match poly
    ctx.texture_square.sub_data(&pos_data, &tex_data);

}



fn render_pattern(ctx: &mut Context, ui: &mut Ui, editable: bool) {
    // store original button style so we can set them back
    let btn_c = ui.style.button.color;
    let btn_h = ui.style.button.hover_color;
    let btn_radius = ui.style.button.radius;
    ui.style.button.radius = BorderRadius::Fixed(0);

    for r in 0..ctx.pattern.rows() {
        for c in ctx.pattern.left_start(r)..ctx.pattern.cols(r) {

            let ci = c as i32;
            let ri = r as i32;

            ui.style.button.color = ctx.color(r, c);
            ui.style.button.hover_color = ctx.color(r, c);

            if ctx.draw_grid {
                // draw grid in black
                ui.drawer2D.rect_color(ctx.start_x + ci *ctx.grid_width,
                                       ctx.start_y + ri * ctx.grid_height,
                                       ctx.grid_width,
                                       ctx.grid_height, Color::black());
            }

            let btn_rect = rect_for_cell(ctx, r as i32, c as i32);

            if ctx.pattern.cell(r, c).is_color() {
                ui.style.button.color = ctx.color(r,c);
                ui.style.button.hover_color = ctx.color(r,c);
            }

            if editable { // render as buttons so we can edit pattern
                if ui.button_at_empty(btn_rect) {
                    if ctx.pattern.cell(r, c).is_base() {
                        *ctx.pattern.cell_mut(r, c) = Cell::Color1;

                    } else {
	                *ctx.pattern.cell_mut(r, c) = Cell::Base;
                    }
                }
            } else { // render as rects
                ui.drawer2D.rounded_rect_color(btn_rect.x, btn_rect.y , btn_rect.w, btn_rect.h, 0.0, ui.style.button.color);
            }
        }

    }

    // set styles back
    ui.style.button.color = btn_c;
    ui.style.button.hover_color = btn_h;
    ui.style.button.radius = btn_radius;

}

fn edit_pattern(ctx: &mut Context, ui: &mut Ui) {

    // add columns,
    // subtract columns

    // shift right
    // shift left

    // shift up
    // shift down


    for r in 0..ctx.pattern.rows() {
        // LEFT BUTTONS

        if ctx.pattern.left_start(r) > 0 {
            // increase column button
            let inc_rect = rect_for_cell(ctx, r as i32, -2);

            if ui.button_at_text_fixed("<", inc_rect) {
                for i in 0..=r {
                    ctx.pattern.add_col_right(i);
                    ctx.pattern.shift_left(i);
                }
            }

        }
        // decrease column button
        let dec_rect = rect_for_cell(ctx, r as i32, -1);

        if ui.button_at_text_fixed(">", dec_rect) {
            for i in 0..=r {
                // remove
                ctx.pattern.remove_col_left(i);
            }
        }

        // RIGHT BUTTONS

        // subtract column button
        let sub_rect = rect_for_cell(ctx, r as i32,  ctx.pattern.cols(r) as i32);

        if ui.button_at_text_fixed("<", sub_rect) {
            for i in 0..(r + 1) {
                ctx.pattern.remove_col_right(i);
            }
        }

        // add column button
        let add_rect = rect_for_cell(ctx, r as i32, ctx.pattern.cols(r) as i32 + 1);

        if ui.button_at_text_fixed(">", add_rect) {
            for i in 0..(r + 1) {
                ctx.pattern.add_col_right(i);
            }
        }
    }
}


fn rect_for_cell(ctx: &Context, row: i32, col: i32) -> Rect {


    let thick = if ctx.draw_grid { ctx.thick } else { 0 };

    let x = ctx.start_x + thick/2 + col * ctx.grid_width;
    let y = ctx.start_y + thick/2 + row * ctx.grid_height;
    let w = ctx.grid_width - thick;
    let h = ctx.grid_height - thick;

    Rect {x, y , w , h: h }

}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Mode {
    Edit,
    Render
}

pub struct Context {
    pattern: Pattern,
    thick: i32,

    start_x: i32,
    start_y: i32,
    grid_height: i32,
    grid_width: i32,

    base_color: Color,
    color_1: Color,

    draw_grid: bool,
    mode: Mode,
    render_center: V2,

    framebuffer: buffer::FrameBuffer,
    texture_square: texture_quad::TextureQuad,
    quad: square::Square,

    clear_color: Color,
    grid: CellGrid<pattern::Cell>

}

impl Context {
    pub fn color(&self, r: usize, c: usize) -> Color {
        match self.pattern.cell(r, c) {
            Cell::Color1 => self.color_1,
            _ => self.base_color
        }
    }
}
