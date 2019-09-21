//#![windows_subsystem = "windows"]

extern crate sdl2;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::mouse::MouseButton;

extern crate gl;

#[macro_use]
extern crate memoffset;

extern crate rand;
use rand::distributions::{Distribution, Uniform};

mod graphics;
use graphics::program::*;
use graphics::texture::*;
use graphics::vao::VertexArrayObject as VAO;
use graphics::vbo::VertexBufferObject as VBO;

use std::default::Default;
use std::ffi::c_void;
use std::mem::size_of;
use std::path::Path;
use std::time::Instant;

mod mine_field;
use mine_field::MineField;

const GROUND_TEXTURE: u32 = 0; // Ground texture unit index
const PROPS_TEXTURE: u32 = 1; // Props texture unit index
const UI_TEXTURE: u32 = 2; // UI texture unit index

const TICK_PER_SEC: u16 = 8;
const TICK_DELAY: u128 = 1e+6 as u128 / TICK_PER_SEC as u128;

const DRAG_THRESHOLD: i32 = 20;
const CLICK_THRESHOLD: u128 = 1e+5 as u128;

#[derive(Clone, Default)]
struct Vertex {
    coord: [f32; 2],
    texture_coord: [f32; 2],
    texture_idx: i32,
}

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?; // Initialize sdl2 crate
    let video_subsystem = sdl.video()?; // Get the video subsystem

    // Setup some opengl attributes
    let gl_attr = video_subsystem.gl_attr(); // Get the attributes
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core); // Profile
    gl_attr.set_context_version(3, 0); // Version

    // Create a new window
    let window = video_subsystem
        .window("Test", 500, 500)
        .resizable()
        .maximized()
        .opengl()
        .build()
        .unwrap();

    // Generate an event pump
    let mut event_pump = sdl.event_pump()?;

    // Make it the current opengl context
    let _glcontext = window.gl_create_context()?;

    // Load OpenGL functions
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const c_void);

    // Create a random number generator
    let rng = &mut rand::thread_rng();

    // Create a program object
    // let mut program = Program::new(Path::new("./shaders/#"), VERTEX_SHADER | FRAGMENT_SHADER)?;
    let program = Program::new(Path::new("./shaders/#"))?;
    Program::make_current(&program); // Tell OpenGL to use this program for rendering

    // Retrive the location of those uniforms
    // program.load_uniforms(vec!["scale", "offset", "aspect", "color_filter"]);
    let scale_loc = program.get_uniform("scale")?;
    let offset_loc = program.get_uniform("offset")?;
    let aspect_loc = program.get_uniform("aspect")?;

    // Give to the shader program the aspect ratio of the screen
    let mut aspect = (1.0, 1.0);
    set_aspect_uniform(
        aspect_loc,
        &mut aspect,
        window.size().0 as i32,
        window.size().1 as i32,
    )?;
    // Pixel size in OpenGL space
    let mut window_px_size = (
        2.0 / (window.size().0 as f32 * aspect.0),
        2.0 / (window.size().1 as f32 * aspect.1),
    );

    // Load the ground texture
    Texture::set_active_unit(GROUND_TEXTURE);
    let ground_texuture = Texture::from_file(Path::new("./textures/Sprite-Sand(big).png"))?;
    let ground_px_size = ground_texuture.pixel_size();
    let ground_tile_size = (ground_px_size.0 * 32.0, ground_px_size.1 * 32.0);

    // Load the props texture
    Texture::set_active_unit(PROPS_TEXTURE);
    let props_texuture = Texture::from_file(Path::new("./textures/Sprite-Props.png"))?;
    let props_px_size = props_texuture.pixel_size();
    let props_tile_size = (props_px_size.0 * 32.0, props_px_size.1 * 32.0);

    // Load the UI texture
    Texture::set_active_unit(UI_TEXTURE);
    let ui_texuture = Texture::from_file(Path::new("./textures/Sprite-UI-new.png"))?;
    let ui_px_size = ui_texuture.pixel_size();
    let ui_tile_size = (ui_px_size.0 * 32.0, ui_px_size.1 * 32.0);

    unsafe {
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
        // Bind the texture units to the 2D samplers
        gl::Uniform1i(
            program.get_uniform("texture0")? as i32,
            GROUND_TEXTURE as i32,
        );
        gl::Uniform1i(
            program.get_uniform("texture1")? as i32,
            PROPS_TEXTURE as i32,
        );
        gl::Uniform1i(program.get_uniform("texture2")? as i32, UI_TEXTURE as i32);
        // Specify the texture units indices to the shader
        gl::Uniform1i(
            program.get_uniform("texture0_idx")? as i32,
            GROUND_TEXTURE as i32,
        );
        gl::Uniform1i(
            program.get_uniform("texture1_idx")? as i32,
            PROPS_TEXTURE as i32,
        );
        gl::Uniform1i(
            program.get_uniform("texture2_idx")? as i32,
            UI_TEXTURE as i32,
        );
    }

    let vao = VAO::new();
    VAO::bind(&vao);

    let menu_size = 4 * 6;
    let mut menu_data: Vec<Vertex> = vec![Default::default(); menu_size];

    loop {
        unsafe {
            gl::Uniform2f(offset_loc as i32, 0.0, 0.0);
            gl::Uniform1f(scale_loc as i32, 1.0);
        }
        apply_texture_rect(
            &mut menu_data,
            0,
            2.0 * ui_tile_size.0,
            0.0,
            4.0 * ui_tile_size.0,
            4.0 * ui_tile_size.1,
            UI_TEXTURE as i32,
        );

        apply_texture_rect(
            &mut menu_data,
            6,
            6.0 * ui_tile_size.0,
            0.0,
            0.5 * ui_tile_size.0,
            1.5 * ui_tile_size.1,
            UI_TEXTURE as i32,
        );
        rotate_tecture_rect(&mut menu_data, 6);

        apply_texture_rect(
            &mut menu_data,
            12,
            ui_px_size.0 * 35.0,
            ui_px_size.1 * 35.0,
            ui_px_size.0 * 9.0,
            ui_px_size.1 * 9.0,
            UI_TEXTURE as i32,
        );

        apply_texture_rect(
            &mut menu_data,
            18,
            ui_px_size.0 * 52.0,
            ui_px_size.1 * 35.0,
            ui_px_size.0 * 9.0,
            ui_px_size.1 * 9.0,
            UI_TEXTURE as i32,
        );

        let _menu_vbo = VBO::new(menu_size, Some(&menu_data));
        VBO::attrib_format(
            program.get_vertex_attrib("coord")?,
            2,
            size_of::<Vertex>(),
            offset_of!(Vertex, coord),
        );
        VBO::attrib_format(
            program.get_vertex_attrib("texture_coord")?,
            2,
            size_of::<Vertex>(),
            offset_of!(Vertex, texture_coord),
        );
        VBO::integer_attrib_format(
            program.get_vertex_attrib("texture_idx")?,
            1,
            size_of::<Vertex>(),
            offset_of!(Vertex, texture_idx),
        );
        unsafe {
            // Set the clear color to the water color
            gl::ClearColor(16.0 / 255.0, 94.0 / 255.0, 1.0, 1.0);
        }

        put_rect(&mut menu_data, 0, -1.0, -1.0, 2.0, 2.0);
        put_rect(&mut menu_data, 6, -0.5, -1.0, 0.75, 0.25);

        let start_down = Instant::now();
        let mut elapsed;
        while {
            elapsed = start_down.elapsed().as_micros() as f32 * 1e-6;
            elapsed < 1.0
        } {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                    }
                    _ => {}
                }
            }
            move_rect(&mut menu_data, 0, None, Some((mix(aspect.1, -1.0, elapsed), 2.0)));
            move_rect(&mut menu_data, 6, None, Some((mix(aspect.1, -1.0, elapsed), 0.25)));
            VBO::write(0, &menu_data[..12]);

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, menu_size as i32);
            }
            window.gl_swap_window();
        }
        move_rect(&mut menu_data, 0, None, Some((-1.0, 2.0)));
        move_rect(&mut menu_data, 6, None, Some((-1.0, 0.25)));
        VBO::write(0, &menu_data[..12]);

        let mut selected = 0;
        let mut size = 0;

        // gl_check()?;

        const PX: f32 = 1.0 / 64.0;

        let mut update = true;
        let mut break_then = false;
        'menu: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                        update = true;
                    }
                    // When a mouse button gets released
                    Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                        // If it's the left button
                        MouseButton::Left => match selected {
                            0 => {}
                            n @ 1..=3 => {
                                size = n;
                                put_rect(
                                    &mut menu_data,
                                    18,
                                    PX * (-28.001 + 22.0 * (n - 1) as f32),
                                    PX * -38.001,
                                    PX * 8.99,
                                    PX * 8.99,
                                );
                                update = true;
                                VBO::write(18, &menu_data[18..24]);
                            }
                            4 => {
                                if size > 0 {
                                    move_rect(&mut menu_data, 12, Some((PX, PX * 8.99)), None);
                                    update = true;
                                    break_then = true;
                                    VBO::write(12, &menu_data[12..18]);
                                }
                            }
                            _ => unreachable!(),
                        },
                        _ => {}
                    },
                    // When the mouse is moved
                    Event::MouseMotion { x, y, .. } if !break_then => {
                        let xx = x as f32 * window_px_size.0 - 1.0 / aspect.0;
                        let yy = 1.0 / aspect.1 - y as f32 * window_px_size.1;

                        if yy < PX * -30.0 && yy > PX * -37.0 {
                            if xx > PX * -28.0 && xx < PX * -21.0 {
                                update = true;
                                selected = 1;
                                put_rect(
                                    &mut menu_data,
                                    12,
                                    PX * -29.001,
                                    PX * -38.001,
                                    PX * 8.99,
                                    PX * 8.99,
                                );
                                VBO::write(12, &menu_data[12..18]);
                            } else if xx > PX * -6.0 && xx < PX {
                                update = true;
                                selected = 2;
                                put_rect(
                                    &mut menu_data,
                                    12,
                                    PX * -7.001,
                                    PX * -38.001,
                                    PX * 8.99,
                                    PX * 8.99,
                                );
                                VBO::write(12, &menu_data[12..18]);
                            } else if xx > PX * 16.0 && xx < PX * 23.0 {
                                update = true;
                                selected = 3;
                                put_rect(
                                    &mut menu_data,
                                    12,
                                    PX * 15.001,
                                    PX * -38.001,
                                    PX * 8.99,
                                    PX * 8.99,
                                );
                                VBO::write(12, &menu_data[12..18]);
                            } else {
                                update = true;
                                selected = 0;
                                reset_rect(&mut menu_data, 12);
                                VBO::write(12, &menu_data[12..18]);
                            }
                        } else if yy < PX * -53.0 && yy > PX * -60.0 && xx > PX && xx < PX * 7.0 {
                            update = true;
                            selected = 4;
                            put_rect(
                                &mut menu_data,
                                12,
                                0.001,
                                PX * -61.001,
                                PX * 8.99,
                                PX * 8.99,
                            );
                            VBO::write(12, &menu_data[12..18]);
                        } else {
                            update = true;
                            selected = 0;
                            reset_rect(&mut menu_data, 12);
                            VBO::write(12, &menu_data[12..18]);
                        }
                    }
                    _ => {}
                }
            }
            // If the screen needs an update
            if update {
                update = false;
                // Update the screen
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::DrawArrays(gl::TRIANGLES, 0, menu_size as i32);
                }
                window.gl_swap_window();
                if break_then {
                    break 'menu;
                }
            }
        }

        apply_texture_rect(
            &mut menu_data,
            12,
            ui_px_size.0 * 52.0,
            ui_px_size.1 * 35.0,
            ui_px_size.0 * 9.0,
            ui_px_size.1 * 9.0,
            UI_TEXTURE as i32,
        );

        let start_up = Instant::now();
        let mut elapsed;
        while {
            elapsed = start_up.elapsed().as_millis() as f32 / 5e2;
            elapsed < 1.0
        } {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                    }
                    _ => {}
                }
            }
            move_rect(&mut menu_data, 0, None, Some((mix(-aspect.1, 1.0, elapsed), 2.0)));
            move_rect(&mut menu_data, 6, None, Some((mix(-aspect.1, 1.0, elapsed), 0.25)));
            move_rect(
                &mut menu_data,
                12,
                None,
                Some((PX * -61.001 + 2.0 * elapsed, PX * 8.99)),
            );
            move_rect(
                &mut menu_data,
                18,
                None,
                Some((PX * -38.001 + 2.0 * elapsed, PX * 8.99)),
            );
            VBO::write(0, &menu_data);

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, menu_size as i32);
            }
            window.gl_swap_window();
        }

        // Create the mine field
        let mut field = match size {
            1 => MineField::new(10, 10),
            2 => MineField::new(20, 20),
            3 => MineField::new(30, 30),
            _ => unreachable!(),
        };
        let w = field.width();
        let h = field.height();
        let max_scale = 2.0 / (if w > h { w } else { h } + 2) as f32;

        // Create a uniform distribution that goes from 0 to 4(excluded)
        // (used for generating variations on the texture)
        let tile_distr = Uniform::from(0..4);

        // Allocate the memory for storing the ground data
        let ground_size = (w + 2) * (h + 2) * 6;
        let mut ground_data: Vec<Vertex> = vec![Default::default(); ground_size];

        // Flags for the animation
        // Format is: ll corner, l side, ul corner, u side, ur corner, r side, lr corner, l side
        let mut border_flags = vec![false; (w + 2) * 2 + h * 2];

        let mut flags: Vec<(usize, usize)> = Vec::new();

        // Allocate the memory for storing the props data
        let props_size = w * h * 6;
        let mut props_data: Vec<Vertex> = vec![Default::default(); props_size];

        // Allocate the memory for storing the ui data
        let ui_size = 12;
        let mut ui_data: Vec<Vertex> = vec![Default::default(); ui_size];

        let mut selected: (usize, usize) = ((w + 2) / 2, (h + 2) / 2); // Selected tile
        let mut cursor: Option<(usize, usize)> = None; // Tile pointed by the cursor

        // Scale and offset of the mine field
        let mut scale = 1.0_f32;
        let mut offset = ((w + 2) as f32 / -2.0, (h + 2) as f32 / -2.0);

        // Set up the mine field ground textures and vertices
        setup_ground(
            &mut ground_data,
            w,
            h,
            GROUND_TEXTURE,
            ground_tile_size,
            rng,
            &tile_distr,
        );

        // Set up the mine field props textures and vertices
        for y in 0..h {
            for x in 0..w {
                let index = (y * w + x) * 6;
                put_unit_square(&mut props_data, index, (x + 1) as f32, (y + 1) as f32);
                apply_texture_rect(
                    &mut props_data,
                    index,
                    0.0,
                    // This if statement reduces the chances of verying the texture
                    // by changing it (on average) only once every four times
                    if random_bool(2) {
                        tile_distr.sample(rng) as f32 * props_tile_size.1
                    } else {
                        0.0
                    },
                    props_tile_size.0,
                    props_tile_size.1,
                    PROPS_TEXTURE as i32,
                );
            }
        }
        // Put the cursor
        put_unit_square(&mut ui_data, 0, selected.0 as f32, selected.1 as f32);
        apply_texture_rect(
            &mut ui_data,
            0,
            0.0,
            0.0,
            ui_tile_size.0,
            ui_tile_size.1,
            UI_TEXTURE as i32,
        );
        // Put the cursor
        put_unit_square(&mut ui_data, 6, selected.0 as f32, selected.1 as f32);
        apply_texture_rect(
            &mut ui_data,
            6,
            ui_tile_size.0,
            0.0,
            ui_tile_size.0,
            ui_tile_size.1,
            UI_TEXTURE as i32,
        );
        // Size of the buffer
        let buffer_size = ground_size + props_size + ui_size;

        // Offsets within the buffer of the various parts
        let ground_offset = 0;
        let props_offset = ground_offset + ground_size;
        let ui_offset = props_offset + props_size;

        let _game_vbo = VBO::new::<Vertex>(buffer_size, None);
        VBO::write(ground_offset, &ground_data);
        VBO::write(props_offset, &props_data);
        VBO::write(ui_offset, &ui_data);

        VBO::attrib_format(
            program.get_vertex_attrib("coord")?,
            2,
            size_of::<Vertex>(),
            offset_of!(Vertex, coord),
        );
        VBO::attrib_format(
            program.get_vertex_attrib("texture_coord")?,
            2,
            size_of::<Vertex>(),
            offset_of!(Vertex, texture_coord),
        );
        VBO::integer_attrib_format(
            program.get_vertex_attrib("texture_idx")?,
            1,
            size_of::<Vertex>(),
            offset_of!(Vertex, texture_idx),
        );

        let mut last_tick = Instant::now();
        let mut second_tick = false;

        let mut left_mouse_button: Option<(i32, i32, Instant)> = None;
        // let mut mouse_cursor: (i32, i32) = (0, 0);
        let mut dragging = false;

        let mut init = true;

        let mut bomb: Option<(usize, usize, Instant, f32, f32, f32)> = None;
        let mut bomb_stage = 0;

        unsafe {
            gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
        }

        let mut update_ground = false;
        let mut update_props = false;
        let mut update_ui = false;
        let mut update_offset = false;
        let mut update_scale = true;
        let mut block_click = false;
        let mut hovering_next = false;

        let result: bool;

        let start_zoom = Instant::now();
        while {
            scale = (start_zoom.elapsed().as_micros() as f32 * 1e-6 * std::f32::consts::FRAC_PI_2)
                .sin()
                * max_scale
                + 0.001;
            scale < max_scale
        } {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                    }
                    _ => {}
                }
            }
            unsafe {
                gl::Uniform1f(scale_loc as i32, scale);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, 0, buffer_size as i32);
            }
            window.gl_swap_window();
        }

        scale = max_scale;

        update = true;
        'game: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                        update = true;
                    }
                    // When the right mouse button gets pressed set its time falg
                    Event::MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => match mouse_btn {
                        MouseButton::Left if bomb == None => {
                            left_mouse_button = Some((x, y, Instant::now()))
                        }
                        _ => {}
                    },
                    // When a mouse button gets released
                    Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                        // If it's the left button
                        MouseButton::Left => {
                            if let Some(t) = left_mouse_button {
                                // If it was a click move the cursor
                                if t.2.elapsed().as_micros() <= CLICK_THRESHOLD && !block_click {
                                    if let Some(c) = cursor {
                                        if selected != c
                                            || field.is_normal(selected.0 - 1, selected.1 - 1)
                                        {
                                            selected = c;
                                            // Move the cursor
                                            put_unit_square(
                                                &mut ui_data,
                                                0,
                                                selected.0 as f32,
                                                selected.1 as f32,
                                            );

                                            if init {
                                                init = false;
                                                // Place the bombs
                                                field.gen_bombs(w * h / 4, (c.0 - 1, c.1 - 1), 3);
                                            }
                                            if let Some(changed) = field.dig(c.0 - 1, c.1 - 1) {
                                                for (x, y, i) in changed {
                                                    move_texture_rect(
                                                        &mut ground_data,
                                                        ((y + 1) * (w + 2) + x + 1) * 6,
                                                        Some((
                                                            field.border_type(x, y) as f32
                                                                * ground_tile_size.0,
                                                            ground_tile_size.0,
                                                        )),
                                                        None,
                                                    );
                                                    if field.has_bomb(x, y) {
                                                        block_click = true;
                                                        cursor = None;
                                                        bomb = Some((
                                                            x,
                                                            y,
                                                            Instant::now(),
                                                            offset.0,
                                                            offset.1,
                                                            scale,
                                                        ));
                                                        move_texture_rect(
                                                            &mut props_data,
                                                            (y * w + x) * 6,
                                                            Some((
                                                                ((field.bombs_near(x, y)
                                                                    + tile_distr.sample(rng))
                                                                    as f32
                                                                    + 1.0)
                                                                    * props_tile_size.0,
                                                                props_tile_size.0,
                                                            )),
                                                            Some((0.0, props_tile_size.1)),
                                                        );
                                                    } else if i {
                                                        move_texture_rect(
                                                            &mut props_data,
                                                            (y * w + x) * 6,
                                                            Some((
                                                                (field.bombs_near(x, y) as f32
                                                                    + 1.0)
                                                                    * props_tile_size.0,
                                                                props_tile_size.0,
                                                            )),
                                                            Some((
                                                                tile_distr.sample(rng) as f32
                                                                    * props_tile_size.1,
                                                                props_tile_size.1,
                                                            )),
                                                        );
                                                    }
                                                }
                                                update_ground = true;
                                                update_props = true;
                                                update = true;
                                            }
                                            if field.check_win() {
                                                result = true;
                                                break 'game;
                                            }
                                        }
                                    }
                                } else if hovering_next {
                                    result = false;
                                    break 'game;
                                }
                                left_mouse_button = None;
                                dragging = false;
                            }
                        }
                        //
                        MouseButton::Right if !block_click && !init => {
                            if let Some(c) = cursor {
                                if field.flag(c.0 - 1, c.1 - 1) {
                                    selected = c;
                                    // Move the cursor
                                    put_unit_square(
                                        &mut ui_data,
                                        0,
                                        selected.0 as f32,
                                        selected.1 as f32,
                                    );

                                    let index = ((c.1 - 1) * w + c.0 - 1) * 6;
                                    if field.is_flagged(c.0 - 1, c.1 - 1) {
                                        flags.push((c.0 - 1, c.1 - 1));

                                        move_texture_rect(
                                            &mut props_data,
                                            index,
                                            Some((16.0 * props_tile_size.0, props_tile_size.0)),
                                            Some((0.0, props_tile_size.1)),
                                        );
                                        VBO::write(
                                            props_offset + index,
                                            &props_data[index..(index + 6)],
                                        );
                                        if field.check_win() {
                                            result = true;
                                            break 'game;
                                        }
                                    } else {
                                        flags = flags
                                            .iter()
                                            .filter(|v| !(**v == (c.0 - 1, c.1 - 1)))
                                            .map(|v| *v)
                                            .collect();

                                        move_texture_rect(
                                            &mut props_data,
                                            index,
                                            Some((0.0, props_tile_size.0)),
                                            Some((0.0, props_tile_size.1)),
                                        );
                                        VBO::write(
                                            props_offset + index,
                                            &props_data[index..(index + 6)],
                                        );
                                    }
                                    update = true;
                                }
                            }
                        }
                        _ => {}
                    },
                    // When the mouse is moved
                    Event::MouseMotion {
                        xrel, yrel, x, y, ..
                    } => {
                        if dragging {
                            // Add the movement to the offset
                            offset.0 += xrel as f32 * window_px_size.0 / scale;
                            offset.1 += -yrel as f32 * window_px_size.1 / scale;

                            // Limit the offset inside the field
                            if offset.0 > -0.5 {
                                offset.0 = -0.5;
                            } else if offset.0 < -(w as f32) - 1.5 {
                                offset.0 = -(w as f32) - 1.5;
                            }
                            if offset.1 > -0.5 {
                                offset.1 = -0.5;
                            } else if offset.1 < -(h as f32) - 1.5 {
                                offset.1 = -(h as f32) - 1.5;
                            }
                            update_offset = true;
                            update = true;
                        } else if let Some(t) = left_mouse_button {
                            // If the mouse moved while clicking it is dragging
                            if (t.0 - x).abs() > DRAG_THRESHOLD || (t.1 - y).abs() > DRAG_THRESHOLD
                            {
                                dragging = true;
                                // Calculate the movement amount relative to OpenGL coordinates
                                offset.0 += (x - t.0) as f32 * window_px_size.0 / scale;
                                offset.1 += -(y - t.1) as f32 * window_px_size.1 / scale;

                                update_offset = true;
                                update = true;
                            }
                        // Prevent the user from clicking when a bomb has been digged
                        } else if !block_click {
                            // Calculate the coordinate in OpenGL space and remove the decimal part
                            // (I can do this because I made each tile 1x1)
                            let xx = ((x as f32 * window_px_size.0 - 1.0 / aspect.0) / scale
                                - offset.0) as usize;
                            let yy = ((1.0 / aspect.1 - y as f32 * window_px_size.1) / scale
                                - offset.1) as usize;
                            // Limit the cursor inside the mine field
                            if xx == 0 || xx > w || yy == 0 || yy > h {
                                if cursor != None {
                                    cursor = None;
                                    // Hide the cursor behind the selected one
                                    put_unit_square(
                                        &mut ui_data,
                                        6,
                                        selected.0 as f32,
                                        selected.1 as f32,
                                    );
                                    VBO::write(ui_offset + 6, &ground_data[6..12]);
                                    update = true;
                                }
                            }
                            // If the cursor changed position
                            else if cursor != Some((xx, yy)) {
                                cursor = Some((xx, yy));
                                // Place the cursor in the new place
                                put_unit_square(&mut ui_data, 6, xx as f32, yy as f32);
                                VBO::write(ui_offset + 6, &ground_data[6..12]);
                                update = true;
                            }
                        } else {
                            // calculate the mouse coordinates relative to the OpenGL workspace
                            let xx = x as f32 * window_px_size.0 - 1.0 / aspect.0;
                            let yy = 1.0 / aspect.1 - y as f32 * window_px_size.1;
                            if yy > 0.75 && xx < 1.0 && xx > -1.0 {
                                if !hovering_next {
                                    put_rect(&mut ui_data, 6, 0.0, 0.5, 2.0, 0.5);
                                    apply_texture_rect(
                                        &mut ui_data,
                                        6,
                                        208.0 * ui_px_size.0,
                                        0.0,
                                        ui_tile_size.0 * 4.0,
                                        ui_tile_size.1,
                                        UI_TEXTURE as i32,
                                    );
                                    hovering_next = true;
                                    update = true;
                                }
                            } else if hovering_next {
                                hovering_next = false;
                                update = true;
                                put_rect(&mut ui_data, 6, 0.0, 0.75, 2.0, 0.25);
                                move_texture_rect(
                                    &mut ui_data,
                                    6,
                                    Some((2.0 * ui_tile_size.0, ui_tile_size.0 * 4.0)),
                                    Some((112.0 * ui_px_size.1, ui_tile_size.1 * 0.5)),
                                );
                            }
                        }
                    }
                    // When scrolling scale up or down based on the scrolling direction
                    Event::MouseWheel { y, .. } if !dragging && bomb == None => {
                        scale *= 1.0 + 0.1 * y as f32;
                        // Limit the scale between 'max_scale' and 1
                        if scale > 1.0 {
                            scale = 1.0;
                        } else if scale < max_scale {
                            scale = max_scale;
                        }
                        update_scale = true;
                        update = true;
                    }
                    _ => {}
                }
            }
            // Check if the the user is dragging
            if !dragging {
                if let Some(t) = left_mouse_button {
                    if t.2.elapsed().as_micros() > CLICK_THRESHOLD {
                        dragging = true;
                    }
                }
            }
            if let Some(b) = bomb {
                let elapsed = b.2.elapsed().as_micros() as f32 * 1e-6;
                if elapsed < 1.5 {
                    update_scale = true;
                    update_offset = true;
                    update = true;

                    if elapsed < 1.0 {
                        offset.0 = mix(b.3, -(b.0 as f32 + 1.5), elapsed);
                        offset.1 = mix(b.4, -(b.1 as f32 + 1.5), elapsed);
                    } else {
                        offset.0 = -(b.0 as f32 + 1.5);
                        offset.1 = -(b.1 as f32 + 1.5);
                    }
                    match bomb_stage {
                        n @ 0..=2 => {
                            if elapsed < 1.0 {
                                scale = mix(b.5, 1.0, elapsed);
                            } else if n == 2 {
                                scale = 1.0;
                            }
                        }
                        3 => {
                            if elapsed < 1.0 {
                                scale = mix(1.0, max_scale, elapsed);
                                move_rect(
                                    &mut ui_data,
                                    6,
                                    None,
                                    Some((aspect.1 - elapsed * 0.25, 0.25)),
                                );
                                update_ui = true;
                            } else {
                                move_rect(&mut ui_data, 6, None, Some((aspect.1 - 0.25, 0.25)));
                                scale = max_scale;
                                bomb = None;
                                update_ui = true;
                            }
                        }
                        _ => return Err("Impossible state!".to_string()),
                    }
                }
            }
            // If a tick has passed
            if last_tick.elapsed().as_micros() > TICK_DELAY {
                last_tick = Instant::now();
                second_tick = !second_tick;

                if let Some(b) = bomb {
                    let index = (b.1 * w + b.0) * 6;
                    match bomb_stage {
                        0 => {
                            if second_tick
                                && advance_frame(&mut props_data, index, props_tile_size.1)
                            {
                                bomb_stage = 1;
                                move_texture_rect(
                                    &mut props_data,
                                    index,
                                    Some((props_tile_size.0 * 14.0, props_tile_size.0)),
                                    None,
                                );
                            }
                        }
                        1 => {
                            if advance_frame(&mut props_data, index, props_tile_size.1) {
                                bomb_stage = 2;
                                move_texture_rect(
                                    &mut props_data,
                                    index,
                                    Some((props_tile_size.0 * 15.0, props_tile_size.0)),
                                    None,
                                );
                            }
                        }
                        2 => {
                            if advance_frame(&mut props_data, index, props_tile_size.1) {
                                put_rect(&mut ui_data, 6, 0.0, aspect.1, 2.0, 0.25);
                                move_texture_rect(
                                    &mut ui_data,
                                    6,
                                    Some((2.0 * ui_tile_size.0, ui_tile_size.0 * 4.0)),
                                    Some((112.0 * ui_px_size.1, ui_tile_size.1 * 0.5)),
                                );
                                bomb_stage = 3;
                                move_texture_rect(
                                    &mut props_data,
                                    index,
                                    None,
                                    Some((props_tile_size.1 * 3.0, props_tile_size.1)),
                                );
                                bomb =
                                    Some((w / 2, h / 2, Instant::now(), offset.0, offset.1, 1.0));
                                flags.clear();
                                field.update_all();
                                for x in 0..w {
                                    for y in 0..h {
                                        if field.has_bomb(x, y) {
                                            move_texture_rect(
                                                &mut props_data,
                                                (y * w + x) * 6,
                                                Some((
                                                    if field.is_flagged(x, y) {
                                                        17.0
                                                    } else {
                                                        18.0
                                                    } * props_tile_size.0,
                                                    props_tile_size.0,
                                                )),
                                                Some((
                                                    tile_distr.sample(rng) as f32
                                                        * props_tile_size.1,
                                                    props_tile_size.1,
                                                )),
                                            );
                                        } else {
                                            move_texture_rect(
                                                &mut props_data,
                                                (y * w + x) * 6,
                                                Some((1.0 * props_tile_size.0, props_tile_size.0)),
                                                None,
                                            );
                                        }
                                        move_texture_rect(
                                            &mut ground_data,
                                            ((y + 1) * (w + 2) + x + 1) * 6,
                                            Some((
                                                field.border_type(x, y) as f32 * ground_tile_size.0,
                                                ground_tile_size.0,
                                            )),
                                            None,
                                        );
                                    }
                                }
                            }
                        }
                        3 => {}
                        _ => unreachable!(),
                    }
                }

                for (x, y) in flags.iter() {
                    let index = (y * w + x) * 6;
                    advance_frame(&mut props_data, index, props_tile_size.1);
                }

                // Update the borders
                update_borders(
                    &mut ground_data,
                    &mut border_flags,
                    w,
                    h,
                    ground_tile_size.1,
                );
                // Play the cursor animation
                advance_frame(&mut ui_data, 0, ui_tile_size.1);

                update_props = true;
                update_ground = true;
                update_ui = true;
                update = true;
            }
            // If the screen needs an update
            if update {
                update = false;

                if update_offset {
                    update_offset = false;
                    unsafe {
                        gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
                    }
                }
                if update_scale {
                    update_scale = false;
                    unsafe {
                        gl::Uniform1f(scale_loc as i32, scale);
                    }
                }
                if update_ground {
                    update_ground = false;
                    VBO::write(ground_offset, &ground_data);
                }
                if update_props {
                    update_props = false;
                    VBO::write(props_offset, &props_data);
                }
                if update_ui {
                    update_ui = false;
                    VBO::write(ui_offset, &ui_data);
                }
                // Update the screen
                if bomb_stage == 3 {
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                        gl::DrawArrays(gl::TRIANGLES, 0, ui_offset as i32 + 6);
                        gl::Uniform2f(offset_loc as i32, -1.0, 0.0);
                        gl::Uniform1f(scale_loc as i32, 1.0);
                        gl::DrawArrays(gl::TRIANGLES, ui_offset as i32 + 6, 6);
                        update_scale = true;
                        update_offset = true;
                    }
                } else {
                    unsafe {
                        gl::Clear(gl::COLOR_BUFFER_BIT);
                        gl::DrawArrays(gl::TRIANGLES, 0, buffer_size as i32);
                    }
                }
                window.gl_swap_window();
            }
        }
        // Setup the page
        unsafe {
            gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
        }
        put_rect(&mut ui_data, 0, 0.0, aspect.1 - 0.25, 2.0, 2.0);
        move_texture_rect(
            &mut ui_data,
            0,
            Some((2.0 * ui_tile_size.0, ui_tile_size.0 * 4.0)),
            Some((0.0, 1.0)),
        );
        put_rect(
            &mut ui_data,
            6,
            44.0 * PX,
            aspect.1 - 0.25 + 12.0 * PX,
            PX * 48.0,
            PX * 16.0,
        );
        if result {
            move_texture_rect(
                &mut ui_data,
                6,
                Some((1.0 * ui_tile_size.0, ui_tile_size.0 * 0.5)),
                Some((1.5 * ui_tile_size.1, ui_tile_size.1 * 1.5)),
            );
        } else {
            move_texture_rect(
                &mut ui_data,
                6,
                Some((1.5 * ui_tile_size.0, ui_tile_size.0 * 0.5)),
                Some((1.5 * ui_tile_size.1, ui_tile_size.1 * 1.5)),
            );
        }
        rotate_tecture_rect(&mut ui_data, 6);
        // Move the page down
        let start_end = Instant::now();
        while {
            elapsed = start_end.elapsed().as_micros() as f32 * 1e-6;
            elapsed < 1.0
        } {
            move_rect(
                &mut ui_data,
                0,
                None,
                Some((mix(aspect.1 - 0.25, -1.0, elapsed), 2.0)),
            );
            move_rect(
                &mut ui_data,
                6,
                None,
                Some((mix(aspect.1 - 0.25 + 12.0 * PX, -52.0 * PX, elapsed), PX * 16.0)),
            );
            VBO::write(ui_offset, &ui_data);

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::Uniform1f(scale_loc as i32, scale * (1.0 - elapsed));
                gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
                gl::DrawArrays(gl::TRIANGLES, 0, ui_offset as i32);
                gl::Uniform2f(offset_loc as i32, -1.0, 0.0);
                gl::Uniform1f(scale_loc as i32, 1.0);
                gl::DrawArrays(gl::TRIANGLES, ui_offset as i32, ui_size as i32);
            }
            window.gl_swap_window();
        }
        // Wait 1sec
        while start_end.elapsed().as_secs() < 2 {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                        update = true;
                    },
                    _ => {}
                }
            }
            if update {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Uniform1f(scale_loc as i32, scale);
                    gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
                    gl::DrawArrays(gl::TRIANGLES, 0, ui_offset as i32);
                    gl::Uniform2f(offset_loc as i32, -1.0, 0.0);
                    gl::Uniform1f(scale_loc as i32, 1.0);
                    gl::DrawArrays(gl::TRIANGLES, ui_offset as i32, ui_size as i32);
                }
                window.gl_swap_window();
            }
        }
        // Move the page up
        while {
            elapsed = (start_end.elapsed().as_micros() as f32 - 2e6) * 1e-6;
            elapsed < 1.0
        } {
            move_rect(
                &mut ui_data,
                0,
                None,
                Some((mix(-1.0,  aspect.1, elapsed), 2.0)),
            );
            move_rect(
                &mut ui_data,
                6,
                None,
                Some((mix(-52.0 * PX, 12.0 * PX + aspect.1, elapsed), PX * 16.0)),
            );
            VBO::write(ui_offset, &ui_data);

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawArrays(gl::TRIANGLES, ui_offset as i32, ui_size as i32);
            }
            window.gl_swap_window();
        }
        // Wait another second
        while start_end.elapsed().as_secs() < 4 {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return Ok(()),
                    // When the window gets resized:
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Update the OpenGL viewport
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                        // Calculate the new aspect ratio and pixel size
                        set_aspect_uniform(aspect_loc, &mut aspect, width, height)?;
                        window_px_size = (
                            2.0 / (width as f32 * aspect.0),
                            2.0 / (height as f32 * aspect.1),
                        );
                        update = true;
                    },
                    _ => {}
                }
            }
            if update {
                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Uniform1f(scale_loc as i32, scale);
                    gl::Uniform2f(offset_loc as i32, offset.0, offset.1);
                    gl::DrawArrays(gl::TRIANGLES, 0, ui_offset as i32);
                    gl::Uniform2f(offset_loc as i32, -1.0, 0.0);
                    gl::Uniform1f(scale_loc as i32, 1.0);
                    gl::DrawArrays(gl::TRIANGLES, ui_offset as i32, ui_size as i32);
                }
                window.gl_swap_window();
            }
        }
    }
}

fn mix(a: f32, b: f32, c: f32) -> f32 {
    b * c + a * (1.0 - c)
}

fn set_aspect_uniform(
    ascpect_loc: u32,
    aspect: &mut (f32, f32),
    width: i32,
    height: i32,
) -> Result<(), String> {
    if width > height {
        *aspect = (height as f32 / width as f32, 1.0);
    } else {
        *aspect = (1.0, width as f32 / height as f32);
    }
    unsafe {
        gl::Uniform2f(ascpect_loc as i32, aspect.0, aspect.1);
    }
    Ok(())
}

fn put_unit_square(vec: &mut Vec<Vertex>, idx: usize, x: f32, y: f32) {
    put_rect(vec, idx, x, y, 1.0, 1.0);
}

fn put_rect(vec: &mut Vec<Vertex>, idx: usize, x: f32, y: f32, w: f32, h: f32) {
    vec[idx + 0].coord = [x, y];
    vec[idx + 1].coord = [x, y + h];
    vec[idx + 2].coord = [x + w, y + h];
    vec[idx + 3].coord = [x + w, y + h];
    vec[idx + 4].coord = [x + w, y];
    vec[idx + 5].coord = [x, y];
}

fn reset_rect(vec: &mut Vec<Vertex>, idx: usize) {
    vec[idx + 0].coord = Default::default();
    vec[idx + 1].coord = Default::default();
    vec[idx + 2].coord = Default::default();
    vec[idx + 3].coord = Default::default();
    vec[idx + 4].coord = Default::default();
    vec[idx + 5].coord = Default::default();
}

fn apply_texture_rect(
    vec: &mut Vec<Vertex>,
    idx: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    texture_idx: i32,
) {
    vec[idx + 0].texture_coord = [x + 1e-5, y + h];
    vec[idx + 1].texture_coord = [x + 1e-5, y];
    vec[idx + 2].texture_coord = [x + w, y];
    vec[idx + 3].texture_coord = [x + w, y];
    vec[idx + 4].texture_coord = [x + w, y + h];
    vec[idx + 5].texture_coord = [x + 1e-5, y + h];

    vec[idx + 0].texture_idx = texture_idx;
    vec[idx + 1].texture_idx = texture_idx;
    vec[idx + 2].texture_idx = texture_idx;
    vec[idx + 3].texture_idx = texture_idx;
    vec[idx + 4].texture_idx = texture_idx;
    vec[idx + 5].texture_idx = texture_idx;
}

fn rotate_tecture_rect(vec: &mut Vec<Vertex>, idx: usize) {
    vec[idx + 0].texture_coord = vec[idx + 1].texture_coord;
    vec[idx + 1].texture_coord = vec[idx + 2].texture_coord;
    vec[idx + 2].texture_coord = vec[idx + 4].texture_coord;
    vec[idx + 3].texture_coord = vec[idx + 4].texture_coord;
    vec[idx + 4].texture_coord = vec[idx + 5].texture_coord;
    vec[idx + 5].texture_coord = vec[idx + 0].texture_coord;
}

fn advance_frame(vec: &mut Vec<Vertex>, idx: usize, amount: f32) -> bool {
    if vec[idx].texture_coord[1] + amount > 1.0 {
        vec[idx + 0].texture_coord[1] = amount;
        vec[idx + 1].texture_coord[1] = 0.0;
        vec[idx + 2].texture_coord[1] = 0.0;
        vec[idx + 3].texture_coord[1] = 0.0;
        vec[idx + 4].texture_coord[1] = amount;
        vec[idx + 5].texture_coord[1] = amount;

        true
    } else {
        vec[idx + 0].texture_coord[1] += amount;
        vec[idx + 1].texture_coord[1] += amount;
        vec[idx + 2].texture_coord[1] += amount;
        vec[idx + 3].texture_coord[1] += amount;
        vec[idx + 4].texture_coord[1] += amount;
        vec[idx + 5].texture_coord[1] += amount;

        false
    }
}
// Probability: 1/2^n
fn random_bool(probability: u32) -> bool {
    let mut res = true;
    for _ in 0..probability {
        res = res && rand::random();
    }
    return res;
}

fn update_borders(data: &mut Vec<Vertex>, flags: &mut Vec<bool>, w: usize, h: usize, size: f32) {
    let mut border_index;
    let mut index;
    for y in 1..h + 1 {
        // Left border
        if flags[y] {
            index = y * (w + 2) * 6;
            flags[y] = !advance_frame(data, index, size);
        } else {
            flags[y] = random_bool(3);
        }

        // Right border
        border_index = h + w + 3 + y;
        if flags[border_index] {
            index = (y * (w + 2) + w + 1) * 6;
            flags[border_index] = !advance_frame(data, index, size);
        } else {
            flags[border_index] = random_bool(3);
        }
    }
    for x in 1..w + 1 {
        // Lower border
        border_index = 2 * h + 3 + w + x;
        if flags[border_index] {
            index = x * 6;
            flags[border_index] = !advance_frame(data, index, size);
        } else {
            flags[border_index] = random_bool(3);
        }

        // Upper border
        border_index = h + 1 + x;
        if flags[border_index] {
            index = ((h + 1) * (w + 2) + x) * 6;
            flags[border_index] = !advance_frame(data, index, size);
        } else {
            flags[border_index] = random_bool(3);
        }
    }
    // Lower left border corner
    if flags[0] {
        flags[0] = !advance_frame(data, 0, size);
    } else {
        flags[0] = random_bool(3);
    }
    // Upper left border corner
    border_index = h + 1;
    if flags[border_index] {
        index = (h + 1) * (w + 2) * 6;
        flags[border_index] = !advance_frame(data, index, size);
    } else {
        flags[border_index] = random_bool(3);
    }
    // Upper right border corner
    border_index = 2 + h + w;
    if flags[border_index] {
        index = ((h + 1) * (w + 2) + w + 1) * 6;
        flags[border_index] = !advance_frame(data, index, size);
    } else {
        flags[border_index] = random_bool(3);
    }
    // Lower right border corner
    border_index = 3 + 2 * h + w;
    if flags[border_index] {
        index = (w + 1) * 6;
        flags[border_index] = !advance_frame(data, index, size);
    } else {
        flags[border_index] = random_bool(3);
    }
}

use rand::rngs::ThreadRng;
fn setup_ground(
    data: &mut Vec<Vertex>,
    w: usize,
    h: usize,
    texture_idx: u32,
    size: (f32, f32),
    rng: &mut ThreadRng,
    distr: &Uniform<u8>,
) {
    let mut index;
    for y in 1..h + 1 {
        for x in 1..w + 1 {
            index = (y * (w + 2) + x) * 6; // get the 1D index of a 2D array
            put_unit_square(data, index, x as f32, y as f32);
            apply_texture_rect(
                data,
                index,
                0.0,
                // Generate a rando variation for each tile
                distr.sample(rng) as f32 * size.1,
                size.0,
                size.1,
                texture_idx as i32,
            );
        }
        // Set up the vertical borders
        // Left border
        index = y * (w + 2) * 6;
        put_unit_square(data, index, 0.0, y as f32);
        apply_texture_rect(
            data,
            index,
            size.0 * 48.0,
            0.0, // distr.sample(rng) as f32 * size.1,
            size.0,
            size.1,
            texture_idx as i32,
        );
        // Right border
        index = (y * (w + 2) + w + 1) * 6;
        put_unit_square(data, index, (w + 1) as f32, y as f32);
        apply_texture_rect(
            data,
            index,
            size.0 * 50.0,
            0.0, // distr.sample(rng) as f32 * size.1,
            size.0,
            size.1,
            texture_idx as i32,
        );
    }
    // Set up the horizontal borders
    for x in 1..w + 1 {
        // Lower border
        index = x * 6;
        put_unit_square(data, index, x as f32, 0.0);
        apply_texture_rect(
            data,
            index,
            size.0 * 49.0,
            0.0, // distr.sample(rng) as f32 * size.1,
            size.0,
            size.1,
            texture_idx as i32,
        );
        // Upper border
        index = ((h + 1) * (w + 2) + x) * 6;
        put_unit_square(data, index, x as f32, (h + 1) as f32);
        apply_texture_rect(
            data,
            index,
            size.0 * 51.0,
            0.0, // distr.sample(rng) as f32 * size.1,
            size.0,
            size.1,
            texture_idx as i32,
        );
    }
    // Lower left border corner
    put_unit_square(data, 0, 0.0, 0.0);
    apply_texture_rect(
        data,
        0,
        size.0 * 52.0,
        0.0, // distr.sample(rng) as f32 * size.1,
        size.0,
        size.1,
        texture_idx as i32,
    );
    // Upper left border corner
    index = (h + 1) * (w + 2) * 6;
    put_unit_square(data, index, 0.0, (h + 1) as f32);
    apply_texture_rect(
        data,
        index,
        size.0 * 53.0,
        0.0, // distr.sample(rng) as f32 * size.1,
        size.0,
        size.1,
        texture_idx as i32,
    );
    // Upper right border corner
    index = ((h + 1) * (w + 2) + w + 1) * 6;
    put_unit_square(data, index, (w + 1) as f32, (h + 1) as f32);
    apply_texture_rect(
        data,
        index,
        size.0 * 54.0,
        0.0, // distr.sample(rng) as f32 * size.1,
        size.0,
        size.1,
        texture_idx as i32,
    );
    // Lower right border corner
    index = (w + 1) * 6;
    put_unit_square(data, index, (w + 1) as f32, 0.0);
    apply_texture_rect(
        data,
        index,
        size.0 * 55.0,
        0.0, // distr.sample(rng) as f32 * size.1,
        size.0,
        size.1,
        texture_idx as i32,
    );
}

fn move_texture_rect(
    vec: &mut Vec<Vertex>,
    idx: usize,
    xw: Option<(f32, f32)>,
    yh: Option<(f32, f32)>,
) {
    if let Some((x, w)) = xw {
        vec[idx + 0].texture_coord[0] = x + 1e-5;
        vec[idx + 1].texture_coord[0] = x + 1e-5;
        vec[idx + 2].texture_coord[0] = x + w;
        vec[idx + 3].texture_coord[0] = x + w;
        vec[idx + 4].texture_coord[0] = x + w;
        vec[idx + 5].texture_coord[0] = x + 1e-5;
    }
    if let Some((y, h)) = yh {
        vec[idx + 0].texture_coord[1] = y + h;
        vec[idx + 1].texture_coord[1] = y;
        vec[idx + 2].texture_coord[1] = y;
        vec[idx + 3].texture_coord[1] = y;
        vec[idx + 4].texture_coord[1] = y + h;
        vec[idx + 5].texture_coord[1] = y + h;
    }
}

fn move_rect(vec: &mut Vec<Vertex>, idx: usize, xw: Option<(f32, f32)>, yh: Option<(f32, f32)>) {
    if let Some((x, w)) = xw {
        vec[idx + 0].coord[0] = x;
        vec[idx + 1].coord[0] = x;
        vec[idx + 2].coord[0] = x + w;
        vec[idx + 3].coord[0] = x + w;
        vec[idx + 4].coord[0] = x + w;
        vec[idx + 5].coord[0] = x;
    }
    if let Some((y, h)) = yh {
        vec[idx + 0].coord[1] = y;
        vec[idx + 1].coord[1] = y + h;
        vec[idx + 2].coord[1] = y + h;
        vec[idx + 3].coord[1] = y + h;
        vec[idx + 4].coord[1] = y;
        vec[idx + 5].coord[1] = y;
    }
}
