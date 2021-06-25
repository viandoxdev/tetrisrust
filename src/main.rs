mod vecs;

use terminal_size::{Width, Height};
use vecs::{Vec2, BoundingBox};

#[derive(Debug)]
struct TetrisContext {
    term_width: u16,
    term_height: u16,
    framebuffer: Framebuffer
}

impl TetrisContext {
    fn init() -> Self {
        let size = terminal_size::terminal_size().unwrap_or((Width(40), Height(40)));

        TetrisContext {
            term_width: size.0.0,
            term_height: size.1.0,
            framebuffer: Framebuffer::new(size.0.0, size.1.0)
        }
    }

    

    fn render(&mut self) {
        self.framebuffer.draw();
        self.framebuffer = Framebuffer::new(self.term_width, self.term_height);
    }
}


struct Mat2([f32; 4]);

impl Mat2 {
    fn new(mat: [f32; 4]) -> Self {
        Mat2(mat)
    }

    fn mul(&self, mat: Mat2) -> Mat2 {
        let a = self.0;
        let b = mat.0;

        Mat2::new([
            a[0] * b[0] + a[1] * b[2], a[0] * b[1] + a[1] * b[3],
            a[2] * b[0] + a[3] * b[2], a[2] * b[1] + a[3] * b[3]
        ])
    }

    fn mul_vec(&self, vec: Vec2<f32>) -> Vec2<f32> {
        let a = self.0;
        let b = vec;
        Vec2::<f32> {
            x: b.x * a[0] + b.y * a[2],
            y: b.x * a[1] + b.y * a[3]
        }
    }
}

#[derive(PartialEq)]
enum Rotation {
    North,
    East,
    South,
    West
}

impl Rotation {
    fn get_mat2(&self) -> Mat2 {
        match self {
            Rotation::North => Mat2::new([
                1f32, 0f32,
                0f32, 1f32
            ]),
            Rotation::East => Mat2::new([
                0f32, 1f32,
               -1f32, 0f32
            ]),
            Rotation::South => Mat2::new([
               -1f32, 0f32,
                0f32,-1f32
            ]),
            Rotation::West => Mat2::new([
                0f32,-1f32,
                1f32, 0f32
            ])
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
enum Pixel {
    Color(ansi_term::Color),
    Background
}

impl Pixel {
    fn get_string(&self) -> String {
       match self {
            &Pixel::Background => 
                ansi_term::Color::Black
                .on(ansi_term::Color::Black)
                .paint(" ").to_string(),
            &Pixel::Color(color) =>
                color
                .on(color)
                .paint("#").to_string()
       } 
    }
}

#[derive(Debug)]
struct Framebuffer {
    term_width: u16,
    term_height: u16,
    xdensity: f32,
    ydensity: f32,
    width: u16,
    height: u16,
    data: Vec<Pixel>
}

impl Framebuffer {
    fn new(width: u16, height: u16) -> Self {
        let xdens = 2.0;
        let ydens = 1.0;
        let h = (height as f32 / ydens).floor() as u16;
        let w = (width as f32 / xdens).floor() as u16;
        println!("{} {} {} {}", width, height, w, h);
        Framebuffer {
            term_width:  width,
            term_height: height,
            xdensity: xdens,
            ydensity: ydens,
            height: h,
            width: w,
            data: vec![Pixel::Background; (w * h) as usize]
        }
    }
    // probably verry fucking slow
    fn to_string(&self) -> String {
        let mut lines = Vec::<String>::new();
        // for each line of pixel
        for chunk in self.data.chunks(self.width as usize) {
            // str is the string representing the pixels
            // of a line, each pixel's string is duplicated
            // self.xdensity times.
            let str = chunk.iter()
                .map(|v| (0..(self.xdensity as i32))
                     .map(|_| v.get_string())
                     .collect::<String>()
                ).collect::<String>();
            // dupe the line ydensity times
            let mut duppedstrs = vec![str; self.ydensity as usize];
            lines.append(&mut duppedstrs);
        }
        lines.iter().map(|v| format!("{}\n", v)).collect::<String>()
    }

    fn set_pixel(&mut self, x: u16, y: u16, value: Pixel) {
        let index = (y * self.width + x) as usize;
        self.data.remove(index);
        self.data.insert(index, value);
    }

    fn draw(&self) {
        print!("{esc}[2J{esc}[1;H", esc = 27 as char);
        print!("{}", self.to_string());
    }
}

#[derive(Clone)]
struct TetrisShape {
    blocks: [Vec2<u16>; 4],
    render_as: Pixel,
    should_shape_be_lowered_on_180_rotation: bool
}

impl TetrisShape {
    fn get_bounding_box(&self) -> BoundingBox<u16> {
        let xs = self.blocks.iter().map(|v| v.x);
        let ys = self.blocks.iter().map(|v| v.y);

        let maxx = xs.clone().max().unwrap() + 1;
        let maxy = ys.clone().max().unwrap() + 1;
        let minx = xs.min().unwrap();
        let miny = ys.min().unwrap();
        
        BoundingBox::<u16>::new(minx, miny, maxx, maxy)
    }

    fn get_center(&self) -> Vec2<f32> {
        let bboxd = self.get_bounding_box().dimensions();
        // longest dimension of the shape's bounding box
        let maxd = std::cmp::max(bboxd.x, bboxd.y);
        // -0.5 here because the blocks coordinates are at the 
        // top left of each block, moving the origin 0.5 closer 
        // to the top left make the rotation work as expected.
        Vec2::<f32> {
            x: maxd as f32 / 2.0 - 0.5,
            y: maxd as f32 / 2.0 - 0.5
        }
    }

    fn get_rotation(&self, rot: Rotation) -> Self {
        if rot == Rotation::North {
            return self.clone();
        }

        let center = self.get_center();
        let mut res = self.clone();
        let mat = rot.get_mat2();
        
        for block in res.blocks.iter_mut() {
            let boff = block.to_type::<f32>() - center;
            let rotated_boff = mat.mul_vec(boff);
            *block += (center + rotated_boff).map(|v| v as u16);
            // lower the shape by one if the rotation in 180°
            // to follow the DTET rotation standard
            if rot == Rotation::South && self.should_shape_be_lowered_on_180_rotation {
                block.y += 1;
            }
        }
        res
    }

    fn draw_to_framebuffer(&self, buff: &mut Framebuffer, x: u16, y: u16) {
        let center = self.get_center();
        // need this to round half away towards 0
        // (0 -> 0, 0.5 -> 0, 0.6 -> 1, 1 -> 1)
        let round = |x: f32| if x.fract() > 0.5 {x.ceil()} else {x.floor()} as i32;
        let center = center.map(round);
        let pos = Vec2 {
            x, y
        }; 
        for block in self.blocks.iter() {
            let off = block.to_type::<i32>() - center;
            let coords = pos.to_type::<i32>() + off;
            buff.set_pixel(coords.x as u16, coords.y as u16, self.render_as.clone());
        }
    }
}


fn main() {
    let mut ctx = TetrisContext::init();
    let a2v = |a: [[u16; 2]; 4]| [
        Vec2::<u16>::from(a[0]),
        Vec2::<u16>::from(a[1]),
        Vec2::<u16>::from(a[2]),
        Vec2::<u16>::from(a[3])
    ];
    let shapes = [
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Cyan),
            blocks: a2v([
                [0, 2], [1, 2], [2, 2], [3, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: true
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Blue),
            blocks: a2v([
                        [1, 0],
                        [1, 1],
                [0, 2], [1, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: false
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Green),
            blocks: a2v([
                [1, 0],
                [1, 1],
                [1, 2], [2, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: false
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Yellow),
            blocks: a2v([
                [0, 0], [1, 0],
                [0, 1], [1, 1]
            ]),
            should_shape_be_lowered_on_180_rotation: false
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Purple),
            blocks: a2v([
                [0, 1], [1, 1], [2, 1],
                        [1, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: true
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Red),
            blocks: a2v([
                        [1, 1], [2, 1],
                [0, 2], [1, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: true
        },
        TetrisShape {
            render_as: Pixel::Color(ansi_term::Color::Green),
            blocks: a2v([
                [0, 1], [1, 1],
                        [1, 2], [2, 2]
            ]),
            should_shape_be_lowered_on_180_rotation: true
        }


    ];
    let mut xoff = 3u16;
    let mut yoff = 3u16;
    let margin = 3u16;
    for shape in shapes.iter() {
        let shapequeue = [
            shape.get_rotation(Rotation::North),
            shape.get_rotation(Rotation::East),
            shape.get_rotation(Rotation::South),
            shape.get_rotation(Rotation::West)
        ];
        let bboxes = shapequeue.iter()
            .map(|v| v.get_bounding_box());
        let width = bboxes.clone().map(|bbox| bbox.width()).reduce(|a, b| a + b + margin).unwrap();
        let height = bboxes       .map(|bbox| bbox.height()).reduce(std::cmp::max).unwrap() + margin;
        if xoff as i32 + width as i32 + margin as i32 >= ctx.framebuffer.width as i32 {
            xoff = 3;
            yoff += height;
        }

        println!("drawing shapes:");
        println!("  width: {}", width);
        println!("  height: {}", height);
        println!("  xoff: {}", xoff);
        println!("  yoff: {}", yoff);
        for s in shapequeue.iter() {
            let bbox = s.get_bounding_box();
            let w = bbox.width();
            println!("  --> drawing (width: {})", w);
            s.draw_to_framebuffer(&mut ctx.framebuffer, xoff, yoff);
            xoff += w + margin;
        }
    }
    println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
    ctx.render();
}
