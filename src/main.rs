use gl_lib::imode_gui::Rect;
use gl_lib::{gl, helpers, na, buffer};
use gl_lib::imode_gui::drawer2d::*;
use gl_lib::imode_gui::ui::*;
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

    for r in 0..10 {
        pattern.add_row(5 + ((10-r) / 2) );
    }


    // maybe easier to just you know add support for at transform to the render code
    // instead of doing the framebuffer stuff
    // TODO: set size of viewport as size of the
    let mut framebuffer = buffer::FrameBuffer::new(&gl, &viewport);

    framebuffer.r = 0.0;
    framebuffer.g = 0.0;
    framebuffer.b = 0.0;
    framebuffer.a = 0.0;
    framebuffer.depth_test = false;

    let mut ctx = Context { pattern,
                            thick : 4,
                            start_x : 200,
                            start_y : 30,
                            grid_width : 30,
                            grid_height : 30,
                            base_color: Color::Rgb(30,30, 240),
                            color_1: Color::Rgb(250, 250, 250),
                            draw_grid: true,
                            mode: Mode::Edit,
                            render_center : V2::new(200.0, 40.0),
                            framebuffer,
    };




    loop {

        // Basic clear gl stuff and get events to UI
        unsafe {
            // set clear color every frame, since bind and clear in framebuffer also sets clear color
            // and opengl is state full, so the clear color is not per frame buffer, but global
            gl.ClearColor(0.9, 0.9, 0.9, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        ui.consume_events(&mut event_pump);

        ui.body_text(&format!("{}", ui.fps()));
        ui.newline();


        match ctx.mode {
            Mode::Edit => edit_view(&mut ctx, &mut ui),
            Mode::Render => render_view(&mut ctx, &mut ui),
        };

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

        for r in 0..10 {
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

    render_pattern(ctx, ui, false);

    ctx.start_x = start_x;
    ctx.start_y = start_y;

    // unbind framebuffer to render to regular viewport again
    ctx.framebuffer.unbind();
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


fn render_polys(ctx: &mut Context, ui: &mut Ui) {

    // use the pattern poly to render

    let (poly, angle, small_w) = create_polygon(ctx);

    ui.drag_point(&mut ctx.render_center, 10.0);


    let size = poly.bounding_box_size();

    // flip top and bottom coords to flip image on its head
    // update texture coords to match poly
    ui.drawer2D.texture_square.sub_tex_coord_data(&ui.drawer2D.gl,
                                                  0.0,
                                                  size.x / ui.drawer2D.viewport.w as f32,
                                                  1.0 - (size.y / ui.drawer2D.viewport.h as f32),
                                                  1.0,
                                                  );




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

            if r == 9 && c == 4 {
                //println!("r=1 c =1 {:?}", btn_rect);
            }

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
                ui.drawer2D.rounded_rect_color(btn_rect.x + 1, btn_rect.y + 1 , btn_rect.w - 2, btn_rect.h - 2, 0.0, ui.style.button.color);
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
        let mut sub_rect = rect_for_cell(ctx, r as i32,  -2);

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
        let mut add_rect = rect_for_cell(ctx, r as i32, -1);

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
        let mut sub_rect = rect_for_cell(ctx, r as i32,  ctx.pattern.cols(r) as i32);

        if ui.button_at_text_fixed("-", sub_rect) {
            for i in 0..(r + 1) {
                ctx.pattern.remove_col_right(i);
            }
        }

        // add column button
        let mut add_rect = rect_for_cell(ctx, r as i32, ctx.pattern.cols(r) as i32 + 1);

        if ui.button_at_text_fixed("+", add_rect) {

            for i in 0..(r + 1) {
                ctx.pattern.add_col_right(i);
            }
        }
    }
}

fn rect_for_cell(ctx: &Context, row: i32, col: i32) -> Rect {


    let x = ctx.start_x + ctx.thick/2 + col * ctx.grid_width;
    let y = ctx.start_y + ctx.thick/2 + row * ctx.grid_height;
    let w = ctx.grid_width - ctx.thick;
    let h = ctx.grid_height - ctx.thick;

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
}

impl Context {
    pub fn color(&self, r: usize, c: usize) -> Color {
        match self.pattern.cell(r, c) {
            Cell::Color1 => self.color_1,
            _ => self.base_color
        }
    }
}
