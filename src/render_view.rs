use super::{gl, Ui, Context, rect_for_cell, RotationWithOrigin, V2, polygon};

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



fn calc_extra_width_left(ctx: &Context, r: usize, top: bool) -> f32 {

    let extra_row = if top { 1 } else { 0 };

    // how far right at the top
    let w_end = (ctx.pattern.left_start(0) as i32 * ctx.grid_width) as f32;


    let grid_h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    // height of this row
    let row_h = ((r + extra_row) as i32 * ctx.grid_height) as f32;

    // scale difference bottom
    let extra_width = w_end * row_h / grid_h;

    // calc target width for row
    // w_min is the min length, then add extra_width
    let target_s = w_end as f32 - extra_width as f32;

    // calc actual start
    let actual_s = (ctx.pattern.left_start(r)  as i32 * ctx.grid_width) as f32;

    // calc how much stretch this row needs

    target_s - actual_s
}



fn calc_extra_width_right(ctx: &Context, r: usize, w_diff: f32, top: bool) -> f32 {

    let extra_row = if top { 1 } else { 0 };

    let w_min = ctx.pattern.cols(0) as i32 * ctx.grid_width;
    let grid_h = (ctx.pattern.rows() as i32 * ctx.grid_height) as f32;

    // height of this row
    let row_h = ((r + extra_row) as i32 * ctx.grid_height) as f32;

    // scale difference bottom
    let extra_width = w_diff * row_h / grid_h;

    // calc target width for row
    // w_min is the min length, then add extra_width
    let target_w = extra_width as f32 + w_min as f32;

    // calc actual width
    let actual_w = (ctx.pattern.cols(r)  as i32 * ctx.grid_width) as f32;

    // calc how much stretch this row needs

    target_w - actual_w
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
    let res = (left, right);
    //println!("res = {:?}", res);
    return res;

    // triangle bottom width left
    let tri_bot_w_l = ctx.pattern.left_start(0) as f32;

    // triangle bottom width right
    let tri_bot_w_r = (ctx.pattern.cols(ctx.pattern.rows() - 1) - ctx.pattern.cols(0)) as f32;


    // both side should slant and gradually become vertical
    // find out where that vertical wall is

    // if either is 0, them the other side can use the whole pattern to slant

    let mut vertical_row_l = 0;
    let mut vertical_row_r = 0;
    if tri_bot_w_l == 0.0 {
        // left side can be our vertical for the right side
        vertical_row_r = 0;
    }

    if tri_bot_w_r == 0.0 {
        // right side can be our vertical for the left side
        vertical_row_l = ctx.pattern.cols(ctx.pattern.rows() - 1);
    }

    if tri_bot_w_r > 0.0 && tri_bot_w_l > 0.0 {
        let tri_total = (tri_bot_w_l + tri_bot_w_r) as f32;

        // find number of rows not in the two triangles
        let square_rows = (ctx.pattern.cols(0) - ctx.pattern.left_start(0)) as f32;

        vertical_row_l = ((square_rows / tri_total) * tri_bot_w_l).round() as usize;
        vertical_row_r = vertical_row_l;
    }
    // we can now calc now know know

    if c < vertical_row_l as usize {
        // slant left
        // calc target x on left line


        let extra_w = calc_extra_width_left(ctx, r, top);
        let re = rect_for_cell(ctx, r as i32, c as i32);
        let e_w = extra_w / ctx.pattern.cols(r) as f32;

        let prev_offset = c as f32 * e_w;
        let this_offset = e_w;

        let left_offset = prev_offset;
        let right_offset = prev_offset + this_offset;

        let left = left_offset + re.x as f32;
        let right = right_offset + (re.x + re.w) as f32;

        return (re.x as f32 - e_w, (re.x + re.w) as f32)

    } else {

        // slant left
        let w_top = ctx.pattern.cols(0) as i32 * ctx.grid_width;
        let w_bot = ctx.pattern.cols(ctx.pattern.rows() - 1) as i32 * ctx.grid_width;
        let w_diff = (w_bot - w_top) as f32; // length of bottom of triangle

        let extra_w = calc_extra_width_right(ctx, r, w_diff, top);

        let re = rect_for_cell(ctx, r as i32, c as i32);

        // extra width should be added gradually
        let e_w = extra_w / ctx.pattern.cols(r) as f32;
        let prev_offset = c as f32 * e_w;

        let this_offset = e_w;

        let left_offset = prev_offset;
        let right_offset = prev_offset + this_offset;

        let left = left_offset + re.x as f32;
        let right = right_offset + (re.x + re.w) as f32;

        //println!("old: {:?}", (left, right));
        return (left, right)
    }


    // vertical for right

    // calc verticalc left lean

    // calc right lean


    // calc vertical line from left and right

}




fn render_polys(ctx: &mut Context, ui: &mut Ui) {

    // use the pattern poly to render

    let poly = create_polygon(ctx);

    ui.drag_point(&mut ctx.render_center, 10.0);

    draw_single_polygon(ctx, ui, 0, &poly);

    draw_single_polygon(ctx, ui, 1, &poly);

    //draw_single_polygon(ctx, ui, -1, &poly, angle, small_w);

}



/// Offset 0 is middle, negative is to the left, positive to the right
fn draw_single_polygon(ctx: &mut Context, ui: &mut Ui, offset_i: i32, pp: &PatternPoly) {

    let small_w = 1.0;
    let angle = 0.0;
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

    let angle = (a / b).atan();

    PatternPoly {
        poly
    }
}


pub struct PatternPoly {
    poly: polygon::Polygon
}
