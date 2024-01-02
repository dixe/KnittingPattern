use gl_lib::imode_gui::Rect;
use gl_lib::{gl, helpers, na};
use gl_lib::imode_gui::drawer2d::*;
use gl_lib::imode_gui::ui::*;
use gl_lib::color::Color;
use gl_lib::collision2d::polygon;

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
    unsafe {
        gl.ClearColor(0.9, 0.9, 0.9, 1.0);
    }


    let mut pattern = Pattern::default();

    for r in 0..10 {
        pattern.add_row(5 + ((10-r) / 2) );
    }

    let mut ctx = Context { pattern,
                            thick : 4,
                            start_x : 200,
                            start_y : 30,
                            grid_width : 30,
                            grid_height : 30,
                            base_color: Color::Rgb(30,30, 240),
                            current_color: Color::Rgb(250, 250, 250),
                            draw_grid: true,
                            mode: Mode::Edit,
                            render_center : V2::new(200.0, 40.0)
                            };


    loop {

        // Basic clear gl stuff and get events to UI
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }


        match ctx.mode {
            Mode::Edit => edit_view(&mut ctx, &mut ui),
            Mode::Render => render_view(&mut ctx, &mut ui),
        };

        ui.consume_events(&mut event_pump);

        window.gl_swap_window();
    }
}

fn edit_view(ctx: &mut Context, ui: &mut Ui) {

    edit_pattern(ctx,ui);
    ui.color_picker(&mut ctx.base_color);
    ui.newline();
    ui.color_picker(&mut ctx.current_color);
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
}


fn create_polygon (ctx: &Context) -> (polygon::Polygon, f32, f32) {
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

    (poly, angle, large_w - small_w)

}



fn render_view(ctx: &mut Context, ui: &mut Ui) {

    // render view
    render_pattern(ctx, ui);
    ui.color_picker(&mut ctx.base_color);
    ui.newline();
    ui.color_picker(&mut ctx.current_color);
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


fn render_pattern(ctx: &mut Context, ui: &mut Ui) {

    // use the pattern poly to render

    let (poly, angle, small_w) = create_polygon(ctx);

    ui.drag_point(&mut ctx.render_center, 10.0);

    let mut transform = polygon::PolygonTransform {
        translation: ctx.render_center,
        rotation: 0.0,
        scale: 1.0,
        flip_y: false
    };


    ui.view_polygon(&poly, &transform);

    transform.translation.x += small_w;


    transform.rotation -= angle;

    ui.view_polygon(&poly, &transform);





}

fn edit_pattern(ctx: &mut Context, ui: &mut Ui) {

    ui.style.button.color = Color::white();
    ui.style.button.hover_color = Color::Rgb(230, 230, 230);

    for r in 0..ctx.pattern.rows() {


        // subtract column button
        let mut sub_rect = rect_for_cell(ctx, r as i32, -1);

        if ui.button_at_text_fixed("-", sub_rect) {
            for i in 0..(r + 1) {
                ctx.pattern.remove_col(i);
            }
        }

        for c in 0..ctx.pattern.cols(r) {

            let ci = c as i32;
            let ri = r as i32;

            ui.style.button.color = ctx.color(r, c);
            ui.style.button.hover_color = Color::Rgb(230, 230, 230);


            if ctx.draw_grid {
                // draw grid in black
                ui.drawer2D.rounded_rect_color(ctx.start_x + ci *ctx.grid_width,
                                               ctx.start_y + ri * ctx.grid_height,
                                               ctx.grid_width,
                                               ctx.grid_height, Color::black());
            }

            let btn_rect = rect_for_cell(ctx, r as i32, c as i32);

            if ctx.pattern.cell(r, c).is_color() {
                ui.style.button.color = ctx.color(r,c);
                ui.style.button.hover_color = Color::Rgb(30, 30, 30);
            }

            if ui.button_at_empty(btn_rect) {
                if ctx.pattern.cell(r, c).is_base() {
                    *ctx.pattern.cell_mut(r, c) = Cell::Color(ctx.current_color);

                } else {
	            *ctx.pattern.cell_mut(r, c) = Cell::Base;
                }
            }
        }

        ui.style.button.color = Color::white();
        ui.style.button.hover_color = Color::Rgb(230, 230, 230);

        // add column button
        let mut add_rect = rect_for_cell(ctx, r as i32, ctx.pattern.cols(r) as i32);

        if ui.button_at_text_fixed("+", add_rect) {

            for i in 0..(r + 1) {
                ctx.pattern.add_col(i);
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
    current_color: Color,
    draw_grid: bool,
    mode: Mode,
    render_center: V2
}

impl Context {
    pub fn color(&self, r: usize, c: usize) -> Color {
        match self.pattern.cell(r, c) {
            Cell::Color(c) => *c,
            _ => self.base_color
        }
    }
}
