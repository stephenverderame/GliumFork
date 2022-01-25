#[macro_use]
extern crate glium;

use glium::Surface;

mod support;

#[test]
fn no_depth_buffer_depth_test() {
    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                            glium::texture::MipmapsOption::NoMipmap, 128, 128).unwrap();
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();

    let parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            .. Default::default()
        },
        .. Default::default()
    };

    match framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                           &glium::uniforms::EmptyUniforms, &parameters)
    {
        Err(glium::DrawError::NoDepthBuffer) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error(None);
}

#[test]
fn layered_framebuffer_test() {
    use glium::framebuffer::{ToDepthAttachment, ToColorAttachment};
    let display = support::build_display();

    #[derive(Copy, Clone)]
    struct Vertex {
        pos: [f32; 2],
    }
    glium::implement_vertex!(Vertex, pos);

    let vbo = glium::VertexBuffer::new(&display, &[
        Vertex { pos: [-1.0,  1.0] }, Vertex { pos: [1.0,  1.0] },
        Vertex { pos: [-1.0, -1.0] }, Vertex { pos: [1.0, -1.0] },
    ]).unwrap();

    let ebo = glium::IndexBuffer::new(&display, 
        glium::index::PrimitiveType::TrianglesList, &[0u8, 2, 3, 1, 0, 3]).unwrap();

    let program = glium::Program::from_source(&display,
    r#"
    #version 330 core
    layout (location = 0) in vec2 pos;
    void main() {
        gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
    }
    "#,
    r#"
    #version 330 core
    out vec4 frag_color;
    void main() {
        frag_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
    "#,
    Some(r#"
    #version 330 core
    layout (triangles) in;
    layout (triangle_strip, max_vertices=18) out;
    void main() {
        for (int face = 0; face < 6; ++face) {
            gl_Layer = face;
            for (int vertex = 0; vertex < 3; ++vertex) {
                gl_Position = gl_in[vertex].gl_Position;
                EmitVertex();
            }
            EndPrimitive();
        }
    }
    "#)).unwrap();

    let color_buf = glium::texture::Cubemap::empty_with_format(&display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        128).unwrap();
    let depth_buf = glium::texture::DepthCubemap::empty_with_format(&display,
        glium::texture::DepthFormat::I24,
        glium::texture::MipmapsOption::NoMipmap,
        128).unwrap();

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display, 
        color_buf.main_level().to_color_attachment(),
        depth_buf.main_level().to_depth_attachment()).unwrap();

    let parameters = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            .. Default::default()
        },
        .. Default::default()
    };

    framebuffer.clear_color_and_depth((0., 0., 0., 1.), 1.);
    match framebuffer.draw(&vbo, &ebo, &program,
                           &glium::uniforms::EmptyUniforms, &parameters)
    {
        Result::Ok(_) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error(None);

    let dst_tex = glium::texture::Texture2d::empty_with_format(&display,
        glium::texture::UncompressedFloatFormat::U8U8U8U8,
        glium::texture::MipmapsOption::NoMipmap,
        128, 128).unwrap();
    let mut test_fbo = glium::framebuffer::SimpleFrameBuffer::new(&display, &dst_tex).unwrap();

    let test_program = glium::Program::from_source(&display,
        r#"
        #version 330 core
        layout (location = 0) in vec2 pos;
        out vec2 tex_coords;
        void main() {
            tex_coords = (pos + vec2(1.0)) * 0.5;
            gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
        }
        "#,
        r#"
        #version 330 core
        in vec2 tex_coords;
        out vec4 frag_color;
        uniform samplerCube tex;
        void main() {
            frag_color = texture(tex, vec3(tex_coords.x, cos(tex_coords.y), tex_coords.y));
        }
        "#,
        None).unwrap();
    let tst_params = glium::DrawParameters::default();
    let u = glium::uniform! {
        tex: color_buf.sampled()
    };
    match test_fbo.draw(&vbo, &ebo, &test_program,
                        &u, &tst_params)
    {
        Result::Ok(_) => (),
        a => panic!("{:?}", a)
    };

    let read_back : Vec<Vec<(u8, u8, u8, u8)>> = dst_tex.read();
    for i in 0 .. 128 {
        for j in 0 .. 128 {
            assert_eq!(read_back[i][j], (255, 0, 0, 255));
        }
    }
}

#[test]
fn no_depth_buffer_depth_write() {
    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::texture::Texture2d::empty_with_format(&display,
                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap, 128, 128).unwrap();
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();

    let parameters = glium::DrawParameters {
        depth: glium::Depth {
            write: true,
            .. Default::default()
        },
        .. Default::default()
    };

    match framebuffer.draw(&vertex_buffer, &index_buffer, &program,
                           &glium::uniforms::EmptyUniforms, &parameters)
    {
        Err(glium::DrawError::NoDepthBuffer) => (),
        a => panic!("{:?}", a)
    };

    display.assert_no_error(None);
}

#[test]
fn simple_dimensions() {
    let display = support::build_display();

    let texture = glium::Texture2d::empty_with_format(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                              128, 128).unwrap();

    let framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    display.assert_no_error(None);
}

#[test]
fn simple_render_to_texture() {
    let display = support::build_display();
    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);

    let texture = glium::Texture2d::empty_with_format(&display,
                                              glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                              128, 128).unwrap();

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display, &texture).unwrap();
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    assert_eq!(read_back[0][0], (255, 0, 0, 255));
    assert_eq!(read_back[64][64], (255, 0, 0, 255));
    assert_eq!(read_back[127][127], (255, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn depth_texture2d() {
    use std::iter;

    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    // the program returns a Z coordinate between 0 (left of screen) and 1 (right of screen)
    let program = program!(&display,
        110 => {
            vertex: "
                #version 110

                attribute vec2 position;

                void main() {
                    gl_Position = vec4(position, position.x, 1.0);
                }
            ",
            fragment: "
                #version 110

                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        },
        100 => {
            vertex: "
                #version 100

                attribute lowp vec2 position;

                void main() {
                    gl_Position = vec4(position, position.x, 1.0);
                }
            ",
            fragment: "
                #version 100

                void main() {
                    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
                }
            ",
        }).unwrap();

    // empty color attachment to put the data
    let color = glium::Texture2d::empty_with_format(&display,
                                            glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                            glium::texture::MipmapsOption::NoMipmap,
                                            128, 128).unwrap();

    // depth texture with a value of 0.5 everywhere
    let depth_data = iter::repeat(iter::repeat(0.5f32).take(128).collect::<Vec<_>>())
                                  .take(128).collect::<Vec<_>>();
    let depth = match glium::texture::DepthTexture2d::new(&display, depth_data) {
        Err(_) => return,
        Ok(t) => t
    };

    // drawing with the `IfLess` depth test
    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display,
                                                                                   &color, &depth).unwrap();
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            .. Default::default()
        },
        .. Default::default()
    };

    framebuffer.clear_color(0.0, 0.0, 0.0, 1.0);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms, &params).unwrap();

    // reading back the color
    let read_back: Vec<Vec<(u8, u8, u8, u8)>> = color.read();

    assert_eq!(read_back[0][0], (255, 255, 255, 255));
    assert_eq!(read_back[127][127], (0, 0, 0, 255));

    display.assert_no_error(None);
}

#[test]
fn multioutput() {
    let display = support::build_display();
    let (vb, ib) = support::build_rectangle_vb_ib(&display);

    let program = match glium::Program::from_source(&display,
        "
            #version 110

            attribute vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
        "
            #version 330

            out vec4 color1;
            out vec4 color2;

            void main() {
                color1 = vec4(1.0, 1.0, 1.0, 1.0);
                color2 = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
        None)
    {
        Err(glium::CompilationError(..)) => return,
        Ok(p) => p,
        e => e.unwrap()
    };

    // building two empty color attachments
    let color1 = glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::AutoGeneratedMipmaps,
                                               128, 128).unwrap();
    color1.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    let color2 = glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::AutoGeneratedMipmaps,
                                               128, 128).unwrap();
    color2.as_surface().clear_color(0.0, 0.0, 0.0, 1.0);

    // building the framebuffer
    let mut framebuffer = glium::framebuffer::MultiOutputFrameBuffer::new(&display,
                               [("color1", &color1), ("color2", &color2)].iter().cloned()).unwrap();

    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // checking color1
    let read_back1: Vec<Vec<(u8, u8, u8, u8)>> = color1.read();
    for row in read_back1.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 255, 255, 255));
        }
    }

    // checking color2
    let read_back2: Vec<Vec<(u8, u8, u8, u8)>> = color2.read();
    for row in read_back2.iter() {
        for pixel in row.iter() {
            assert_eq!(pixel, &(255, 0, 0, 255));
        }
    }


    display.assert_no_error(None);
}

#[test]
fn array_level() {
    let display = support::build_display();

    let texture = match glium::texture::Texture2dArray::empty(&display, 128, 128, 4) {
        Ok(t) => t,
        Err(_) => return
    };

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display,
                                                          texture.main_level().layer(2).unwrap()).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // TODO: read the texture to see if it succeeded

    display.assert_no_error(None);
}

#[test]
fn cubemap_layer() {
    // ignoring test on travis
    // TODO: find out why they are failing
    if ::std::env::var("TRAVIS").is_ok() {
        return;
    }

    let display = support::build_display();

    let texture = match glium::texture::Cubemap::empty(&display, 128) {
        Ok(t) => t,
        Err(_) => return
    };

    let mut framebuffer = glium::framebuffer::SimpleFrameBuffer::new(&display,
                    texture.main_level().image(glium::texture::CubeLayer::PositiveY)).unwrap();
    assert_eq!(framebuffer.get_dimensions(), (128, 128));

    let (vb, ib, program) = support::build_fullscreen_red_pipeline(&display);
    framebuffer.draw(&vb, &ib, &program, &glium::uniforms::EmptyUniforms,
                     &Default::default()).unwrap();

    // TODO: read the texture to see if it succeeded

    display.assert_no_error(None);
}

#[test]
#[should_panic]
fn multi_color_attachments_maximum() {
    let display = support::build_display();

    let color_textures = (0 .. 32)
        .map(|_| {
            glium::Texture2d::empty_with_format(&display,
                                               glium::texture::UncompressedFloatFormat::U8U8U8U8,
                                               glium::texture::MipmapsOption::NoMipmap,
                                               128, 128).unwrap()
        })
        .collect::<Vec<_>>();

    let colors = (0 .. color_textures.len()).map(|i| {("attachment", &color_textures[i])} );
    glium::framebuffer::MultiOutputFrameBuffer::new(&display, colors).unwrap();
}

#[test]
#[should_panic]
fn empty_framebuffer_wrong_layers() {
    use glium::framebuffer::EmptyFrameBuffer;

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        panic!();
    }

    let _fb = EmptyFrameBuffer::new(&display, 256, 256, Some(0), None, true);
}

#[test]
#[should_panic]
fn empty_framebuffer_wrong_samples() {
    use glium::framebuffer::EmptyFrameBuffer;

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        panic!();
    }

    let _fb = EmptyFrameBuffer::new(&display, 256, 256, None, Some(0), true);
}

#[test]
fn empty_framebuffer_width_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 4294967295, 256, None, None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_height_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 4294967295, None, None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_layers_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_layered_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 256, Some(4294967295), None, true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_samples_out_of_range() {
    use glium::framebuffer::{EmptyFrameBuffer, ValidationError};

    let display = support::build_display();

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let _fb = match EmptyFrameBuffer::new(&display, 256, 256, None, Some(4294967295), true) {
        Err(ValidationError::EmptyFramebufferUnsupportedDimensions) => (),
        _ => panic!(),
    };

    display.assert_no_error(None);
}

#[test]
fn empty_framebuffer_simple_draw() {
    use glium::framebuffer::{EmptyFrameBuffer};

    let display = support::build_display();
    let (vertex_buffer, index_buffer, program) = support::build_fullscreen_red_pipeline(&display);

    // ignore the test
    if !EmptyFrameBuffer::is_supported(&display) {
        return;
    }

    let mut fb = EmptyFrameBuffer::new(&display, 256, 256, None, None, true).unwrap();
    fb.clear_color(0.0, 0.0, 0.0, 0.0);
    fb.draw(&vertex_buffer, &index_buffer, &program,
            &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();

    display.assert_no_error(None);
}
