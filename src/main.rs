use gl_lib::imode_gui::Rect;
use gl_lib::{
    gl, helpers, na, buffer};
use gl_lib::imode_gui::drawer2d::*;
use gl_lib::imode_gui::ui::*;
use gl_lib::objects::{square, texture_quad};
use gl_lib::color::Color;
use gl_lib::collision2d::polygon;
use gl_lib::imode_gui::style::BorderRadius;
mod pattern;
use pattern::*;


type V2 = na::Vector2::<f32>;
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

    for r in 0..20 {
        let cols = 5 + ((20-r) / 5) ;
        pattern.add_row(cols);
        /*
        for c in 0..cols {
        if r % 2 == 1 && c % 2 == 1 {
         *pattern.cell_mut(r, c) = Cell::Color1;
    }
    }
         */

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
                            start_x : 400,
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
    };




    update_render_obj_data(&mut ctx);


    let mut clear_color = Color::RgbAf32(0.9, 0.9, 0.9, 1.0);

    loop {

        // Basic clear gl stuff and get events to UI
        unsafe {
            // set clear color every frame, since bind and clear in framebuffer also sets clear color
            // and opengl is state full, so the clear color is not per frame buffer, but global
            let cc = clear_color.as_vec4();
            gl.ClearColor(cc.x, cc.y, cc.z, cc.w);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        ui.consume_events(&mut event_pump);

        ui.body_text(&format!("{}", ui.fps()));
        ui.newline();


        render_view(&mut ctx, &mut ui);
        edit_view(&mut ctx, &mut ui);

        /*
        match ctx.mode {
            Mode::Edit => edit_view(&mut ctx, &mut ui),
            Mode::Render => render_view(&mut ctx, &mut ui),
        };
         */

        ui.newline();
        if ui.button("Reload shaders") {
          ui.drawer2D.reload_all_shaders();
        }

        ui.newline();

        // TODO: find a better way to update the framebuffer viewport when it changes. For now
        // updating every frame is fine, or not, ruins frame rate
        //ctx.framebuffer.update_viewport(&gl, &ui.drawer2D.viewport);

        if ui.button("Change") {

        }
        ui.newline();

        ui.color_picker(&mut clear_color);
        ui.newline();

        window.gl_swap_window();
    }
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

        for r in 0..20 {
            pattern.add_row(5 + ((10-r) / 2) );
        }

        ctx.pattern = pattern;
    }

    ui.newline();

}


fn create_polygon(ctx: &Context) -> (polygon::Polygon, f32, f32) {
    let large_w = (ctx.pattern.cols(
        0) as i32 * ctx.grid_width) as f32;

    let small_w = (ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width) as f32;
    let h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    let poly = polygon::Polygon { vertices: vec![V2::new(0.0, 0.0),
                                                 V2::new(small_w, 0.0),
                                                 V2::new(large_w, h),
                                                 V2::new(0.0, h)]
    };


    let a = (large_w - small_w) as f32;
    let b = h as f32;

    let angle = (a / b).atan();

    (poly, angle, small_w)

}



fn render_to_framebuffer(ctx: &mut Context, ui: &mut Ui) {

    // update framebuffer info
    // render to framebuffer
    ctx.framebuffer.bind_and_clear(gl::COLOR_BUFFER_BIT);

    // store start_x and start_y so we can render the image in top left of texture

    let start_x = ctx.start_x;
    let start_y = ctx.start_y;

    ctx.start_x = 0;
    ctx.start_y = 0;


    //render_pattern(ctx, ui, false);

    let grid_h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    // render flipped
    let vh = ui.drawer2D.viewport.h as f32;
    let base = vh - grid_h;// base offset for drawing flipped and takine sdl inverse coordinate system into consideration


    // render a red square around the base rect
    let w_bot = ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width;

    for r in 0..ctx.pattern.rows() {
        for c in ctx.pattern.left_start(r)..ctx.pattern.cols(r) {

            let re = rect_for_cell(ctx, r as i32, c as i32);
            let (top_left, top_right) = calc_left_and_right(ctx, r, c, true);
            let (bot_left, bot_right) = calc_left_and_right(ctx, r , c, false);

            // since sdl is inverse, so y=0 is top, and y = view.h is bottom, we have to inverse the y
            // br -> tr -> tl -> bl
            ctx.quad.sub_data_all(&[bot_right, base + re.y as f32,
                                    top_right, base + (re.y + re.h) as f32,
                                    top_left,  base + (re.y + re.h) as f32,
                                    bot_left,  base + re.y as f32
            ]);

            //ui.drawer2D.rounded_rect_color(re.x, re.y , re.w, re.h, 0.0, color);
            ui.drawer2D.render_viewport_obj(&ctx.quad, ctx.color(r, c));
        }

    }


    ctx.start_x = start_x;
    ctx.start_y = start_y;

    // unbind framebuffer to render to regular viewport again
    ctx.framebuffer.unbind();
}

fn calc_extra_width(ctx: &Context, r: usize, w_diff: f32, top: bool) -> f32 {

    let extra_row = if top { 1 } else { 0 };
    // width of largest row
    let w_bot = ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width;
    let grid_h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    // height of this row
    let row_h = ((ctx.pattern.rows() - (r + extra_row)) as i32 * ctx.grid_height) as f32;


    // scale difference bottom
    let extra_width = w_diff * row_h / grid_h;

    // calc target width for row
    // w_bot is the min length, then add extra_width
    let target_w = extra_width as f32 + w_bot as f32;

    // calc actual width
    let actual_w = (ctx.pattern.cols(r)  as i32 * ctx.grid_width) as f32;

    // calc how much stretch this row needs

    target_w - actual_w
}

fn calc_left_and_right(ctx: &Context, r: usize, c: usize, top: bool) -> (f32, f32) {

    let w_top = ctx.pattern.cols(0) as i32 * ctx.grid_width;
    let w_bot = ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width;
    let w_diff = (w_top - w_bot) as f32; // length of bottom of triangle

    let extra_w = calc_extra_width(ctx, r, w_diff, top);

    let re = rect_for_cell(ctx, r as i32, c as i32);

    // extra width sould be added gradually
    let e_w = extra_w / ctx.pattern.cols(r) as f32;
    let prev_offset = c as f32 * e_w;

    let this_offset = e_w;

    let left_offset = prev_offset;
    let right_offset = prev_offset + this_offset;

    let left = left_offset + re.x as f32;
    let right = right_offset + (re.x + re.w) as f32;

    (left, right)
}


fn render_view(ctx: &mut Context, ui: &mut Ui) {

    render_to_framebuffer(ctx, ui);

    // render view
    render_polys(ctx, ui);
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

    if ui.button("Edit") {
        ctx.mode = Mode::Edit;
    }
}


// just flip on y axis
fn update_render_obj_data(ctx: &mut Context) {
    let pos_data: [f32; 12] = [
        // positions
        0.5,  0.5, 0.0, // rt
        -0.5,  0.5, 0.0, // lt
        -0.5, -0.5, 0.0,// lb
        0.5, -0.5, 0.0, // rb
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

fn render_polys(ctx: &mut Context, ui: &mut Ui) {

    // use the pattern poly to render

    let (poly, angle, small_w) = create_polygon(ctx);

    ui.drag_point(&mut ctx.render_center, 10.0);

    draw_single_polygon(ctx, ui, 0, &poly, angle, small_w);

    draw_single_polygon(ctx, ui, 1, &poly, angle, small_w);

    //draw_single_polygon(ctx, ui, -1, &poly, angle, small_w);

/*
    let mut transform = polygon::PolygonTransform {
        translation: ctx.render_center,
        rotation: 0.0,
        scale: 1.0,
        flip_y: false
    };


    ui.view_polygon(&poly, &transform);
    ui.drawer2D.render_img(ctx.framebuffer.color_tex, ctx.render_center.x as i32, ctx.render_center.y as i32, size);

    transform.rotation -= angle;
    transform.translation.x += small_w;



    ui.view_polygon(&poly, &transform);
    ui.drawer2D.render_img_rot(ctx.framebuffer.color_tex,
                               (ctx.render_center.x  + small_w) as i32,
                               ctx.render_center.y as i32,
                               RotationWithOrigin::TopLeft(angle),
                               size);


    transform.rotation += angle * 2.0;
    transform.translation.x -= small_w * 2.0;

    ui.view_polygon(&poly, &transform);
    ui.drawer2D.render_img_rot(ctx.framebuffer.color_tex,
                               (ctx.render_center.x  + small_w) as i32,
                               ctx.render_center.y as i32,
                               RotationWithOrigin::TopLeft(angle),
                               size);


*/
}


/// Offset 0 is middle, negative is to thje left, positive to the right
fn draw_single_polygon(ctx: &mut Context, ui: &mut Ui, offset_i: i32, poly: &polygon::Polygon, angle: f32, small_w: f32) {

    let offset = offset_i as f32;


    let transform = polygon::PolygonTransform {
        translation: ctx.render_center + V2::new(offset * small_w, 0.0),
        rotation: angle * -offset as f32,
        scale: 1.0,
        flip_y: false
    };


    ui.view_polygon(&poly, &transform);


    // when rotation we do it around the TopLeft corner
    // when offset is >= 0 we want the top left corner to match up to the prev, so we do nothing
    // when < 0, we want the top right corner to match up, and need to compensate, since it was pushed down, and a bit to the
    // left by the rotation

    let mut correction = V2::new(small_w * offset, 0.0);

    if offset_i < 0 {
        correction.x = angle.cos() * small_w;
    }


    // calc how much the top left corner got pushed left and down / up and right, and compensate for it

    let final_pos = ctx.render_center + correction;

    let s = V2::new(ui.drawer2D.viewport.w as f32, ui.drawer2D.viewport.h as f32);

    ui.drawer2D.render_img_custom_obj(ctx.framebuffer.color_tex,
                                      &ctx.texture_square,
                                      final_pos.x as i32,
                                      final_pos.y as i32,
                                      RotationWithOrigin::TopLeft(angle * offset),
                                      s);

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
        // subtract column button
        let sub_rect = rect_for_cell(ctx, r as i32,  -2);

        if ui.button_at_text_fixed("-", sub_rect) {
            for i in 0..=r {
                // remove
                ctx.pattern.shift_left(i);
            }
            // shift everything below to the right
            for i in (r + 1)..ctx.pattern.rows() {
                ctx.pattern.shift_left(i);
            }
        }

        // add column button
        let add_rect = rect_for_cell(ctx, r as i32, -1);

        if ui.button_at_text_fixed("+", add_rect) {
            for i in 0..=r {
                ctx.pattern.add_col_left(i);
            }

            // shift everything below to the right
            for i in (r + 1)..ctx.pattern.rows() {
                ctx.pattern.shift_right(i);
            }
        }


        // RIGHT BUTTONS

        // subtract column button
        let sub_rect = rect_for_cell(ctx, r as i32,  ctx.pattern.cols(r) as i32);

        if ui.button_at_text_fixed("-", sub_rect) {
            for i in 0..(r + 1) {
                ctx.pattern.remove_col_right(i);
            }
        }

        // add column button
        let add_rect = rect_for_cell(ctx, r as i32, ctx.pattern.cols(r) as i32 + 1);

        if ui.button_at_text_fixed("+", add_rect) {

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
enum Mode {
    Edit,
    Render
}

struct Context {
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
}

impl Context {
    pub fn color(&self, r: usize, c: usize) -> Color {
        match self.pattern.cell(r, c) {
            Cell::Color1 => self.color_1,
            _ => self.base_color
        }
    }
}
