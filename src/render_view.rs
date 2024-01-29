use super::{gl, Color, Ui, Context, Pattern, rect_for_cell, RotationWithOrigin, V2, polygon};

pub fn view(ctx: &mut Context, ui: &mut Ui) {

    // TODO: Only update this when needed. That is if the pattern has changes like inc or dec, of change color of square
    // other wise we can keep the current image

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

    let vh = ui.drawer2D.viewport.h as f32;
    let base = vh;

    for r in 0..ctx.pattern.rows() {
        for c in ctx.pattern.left_start(r)..ctx.pattern.cols(r) {

            let re = rect_for_cell(ctx, r as i32, c as i32);
            let (top_left, top_right) = calc_left_and_right(ctx, r, c, true);
            let (bot_left, bot_right) = calc_left_and_right(ctx, r , c, false);

            // since sdl is inverse, so y=0 is top, and y = view.h is bottom, we have to inverse the y
            // br -> tr -> tl -> bl SAME AS
            // rb -> rt -> lt -> lb
            ctx.quad.sub_data_all(&[bot_right, base - re.y as f32,
                                    top_right, base - (re.y + re.h) as f32,
                                    top_left,  base - (re.y + re.h) as f32,
                                    bot_left,  base - re.y as f32
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



fn calc_left_and_right(ctx: &Context, r: usize, c: usize, top: bool) -> (f32, f32) {

    // this might seem inverted, but i think it is something with sdl going from top.y < bottom.y
    // and open gl is top.y > bottom.y
    let extra_row = if top { 1 } else { 0 };
    let grid_h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;
    let row_h = ((r + extra_row) as i32 * ctx.grid_height) as f32;

    // find the left side x

    // left side triangle bottom is
    let ltb = (ctx.pattern.left_start(0) as i32 * ctx.grid_width) as f32;

    let left_x = ltb * (1.0 - row_h / grid_h);


    // find the right side x

    // right side triangle bottom is
    let w_top = ctx.pattern.cols(0) as i32 * ctx.grid_width; // right x of the square part of the pattern
    let w_bot = ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width; // max width
    let rtb = (w_bot - w_top) as f32;

    // only width of the triangle part.
    let mut right_x = rtb * row_h / grid_h;

    // add the with of the square part too
    right_x += ctx.pattern.cols(0) as f32 * ctx.grid_width as f32;

    // find how long each mask is

    let masks = (ctx.pattern.cols(r) - ctx.pattern.left_start(r)) as f32;
    let mask_width = (right_x - left_x) / masks;

    // offset with number(col num)
    let col = c - ctx.pattern.left_start(r);
    let left = left_x + mask_width * col as f32;
    let right = left_x + mask_width * (col + 1) as f32;

    // return
    (left, right)
}




fn render_polys(ctx: &mut Context, ui: &mut Ui) {

    // use the pattern poly to render

    let poly = create_polygon(ctx);

    ui.drag_point(&mut ctx.render_center, 10.0);


    let (a_left, a_right) = draw_middle_polygon(ctx, ui, &poly);

    draw_left_polygon(ctx, ui, a_left, &poly);
/*
    draw_single_polygon(ctx, ui, 1, &poly);

    draw_single_polygon(ctx, ui, -1, &poly);
*/
}


#[derive(Clone, Copy, Debug)]
struct AnchorPoint {
    pos: V2,
    angle: f32 // angle of the seam line
}

fn draw_middle_polygon(ctx: &mut Context, ui: &mut Ui, pp: &PatternPoly) -> (AnchorPoint, AnchorPoint) {

    let transform = polygon::PolygonTransform {
        translation: ctx.render_center,
        rotation: 0.0,
        scale: 1.0,
        flip_y: false
    };

    ui.view_polygon(&pp.poly, &transform);

    let final_pos = ctx.render_center;
    let s = V2::new(ui.drawer2D.viewport.w as f32, ui.drawer2D.viewport.h as f32);
    ui.drawer2D.render_img_custom_obj(ctx.framebuffer.color_tex,
                                      &ctx.texture_square,
                                      final_pos.x as i32,
                                      final_pos.y as i32,
                                      RotationWithOrigin::TopLeft(0.0),
                                      s);


    let left_anchor = AnchorPoint {
        pos: ctx.render_center + V2::new((ctx.pattern.left_start(0) as i32 * ctx.grid_width) as f32, 0.0),
        angle: angle_left(ctx, &ctx.pattern)
    };

    let right_anchor = AnchorPoint{
        pos: ctx.render_center + V2::new((ctx.pattern.cols(0) as i32 * ctx.grid_width) as f32, 0.0),
        angle: angle_right(ctx, &ctx.pattern)
    };

    // debug show points
    ui.drawer2D.rect_color(left_anchor.pos.x, left_anchor.pos.y, 5, 5, Color::red());

    ui.drawer2D.rect_color(right_anchor.pos.x, right_anchor.pos.y, 5, 5, Color::red());

    (left_anchor, right_anchor)
}


fn draw_left_polygon(ctx: &mut Context, ui: &mut Ui, anchor: AnchorPoint, pp: &PatternPoly) -> (AnchorPoint, AnchorPoint) {


    // move polygon to the left, so out right cornor matches anchor point. Rotate around the top right corner
    let offset = -V2::new((ctx.pattern.cols(0) as i32 * ctx.grid_width) as f32, 0.0);

    let final_angle = anchor.angle + angle_right(ctx, &ctx.pattern);

    let transform = polygon::PolygonTransform {
        translation: anchor.pos + offset,
        rotation: final_angle,
        scale: 1.0,
        flip_y: false
    };

    ui.view_polygon(&pp.poly, &transform);

    let final_pos = anchor.pos + offset;
    let s = V2::new(ui.drawer2D.viewport.w as f32, ui.drawer2D.viewport.h as f32);
    ui.drawer2D.render_img_custom_obj(ctx.framebuffer.color_tex,
                                      &ctx.texture_square,
                                      final_pos.x as i32,
                                      final_pos.y as i32,
                                      RotationWithOrigin::Point(-offset, -final_angle),
                                      //RotationWithOrigin::Point(V2::new(0.0, 0.0), -std::f32::consts::PI / 2.0), //:-final_angle * 2.4),
                                      //RotationWithOrigin::TopLeft(-std::f32::consts::PI / 2.0), //:-final_angle * 2.4,)
                                      s);


    let left_anchor = AnchorPoint {
        pos: anchor.pos,
        angle: 0.0
    };

    let right_anchor = AnchorPoint{
        pos: V2::new((ctx.pattern.cols(0) as i32 * ctx.grid_width) as f32, 0.0),
        angle: 0.0
    };

    // debug show points
    //ui.drawer2D.rect_color(left_anchor.pos.x, left_anchor.pos.y, 5, 5, Color::green());

    //ui.drawer2D.rect_color(right_anchor.pos.x, right_anchor.pos.y, 5, 5, Color::green());

    (left_anchor, right_anchor)
}


/// Offset 0 is middle, negative is to the left, positive to the right
/// returns the left and right top anchor points
fn draw_single_polygon(ctx: &mut Context, ui: &mut Ui, offset_i: i32, pp: &PatternPoly) {

    let small_w = ctx.top_length();
    let angle = 0.0; //ctx.angle_right();

    let offset = offset_i as f32;
    let transform = polygon::PolygonTransform {
        translation: ctx.render_center + V2::new(offset * small_w, 0.0),
        rotation: angle * -offset as f32,
        scale: 1.0,
        flip_y: false
    };

    ui.view_polygon(&pp.poly, &transform);

    // when rotation we do it around the TopLeft corner
    // when offset is >= 0 we want the top left corner to match up to the prev, so we do nothing
    // when < 0, we want the top right corner to match up, and need to compensate, since it was pushed down, and a bit to the
    // left by the rotation

    let mut correction = V2::new(small_w * offset, 0.0);

    // calc how much the top left corner got pushed left and down / up and right, and compensate for it

    let final_pos = ctx.render_center + correction;

    let s = V2::new(ui.drawer2D.viewport.w as f32, ui.drawer2D.viewport.h as f32);

    ui.drawer2D.render_img_custom_obj(ctx.framebuffer.color_tex,
                                      &ctx.texture_square,
                                      final_pos.x as i32,
                                      final_pos.y as i32,
                                      RotationWithOrigin::TopLeft(angle * offset),
                                      s);

    ui.view_polygon(&pp.poly, &transform);
}





fn create_polygon(ctx: &Context) -> PatternPoly {

    // top left is left offset of first row in x and 0 in y

    let top_x = (ctx.pattern.left_start(0) as i32 * ctx.grid_width) as f32;
    let large_w = (ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width) as f32;

    let small_w = (ctx.pattern.cols(0) as i32 * ctx.grid_width) as f32;
    let h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    let poly = polygon::Polygon { vertices: vec![V2::new(top_x, 0.0),
                                                 V2::new(small_w, 0.0),
                                                 V2::new(large_w, h),
                                                 V2::new(0.0, h)]
    };

    let a = (large_w - small_w) as f32;
    let b = h as f32;



    PatternPoly {
        poly
    }
}


pub struct PatternPoly {
    poly: polygon::Polygon
}




fn angle_right(ctx: &Context, pattern: &Pattern) -> f32 {
    // width of right triangle bottom
    let a = ((pattern.cols(pattern.rows() - 1) - pattern.cols(0)) as i32 * ctx.grid_width) as f32;

    // height of grid
    let b = (pattern.rows() as i32 * ctx.grid_height) as f32;

    (a / b).atan()
}


pub fn angle_left(ctx: &Context, pattern: &Pattern) -> f32 {
    let a = (pattern.left_start(0) as i32 * ctx.grid_width) as f32;

    // height of grid
    let b = (pattern.rows() as i32 * ctx.grid_height) as f32;

    (a / b).atan()
}
