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

    
    // draw the framebuffer and replace it
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
#[derive(Copy, Clone)]
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
    fn rotateCW(self) -> Self {
        match self {
            Rotation::North => Rotation::East,
            Rotation:: East => Rotation::South,
            Rotation::South => Rotation::West,
            Rotation:: West => Rotation::North
        }
    }
    fn rotateCCW(self) -> Self {
        match self {
            Rotation::North => Rotation::West,
            Rotation:: East => Rotation::North,
            Rotation::South => Rotation::East,
            Rotation:: West => Rotation::South
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

    fn make_space(&self) {
        for _ in 0..self.height {
            println!("");
        }
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

    fn get_rounded_center(&self) -> Vec2<i32> {
        // need this to round half away towards 0
        // (0 -> 0, 0.5 -> 0, 0.6 -> 1, 1 -> 1)
        self.get_center().map(|x: f32| if x.fract() > 0.5 {x.ceil()} else {x.floor()} as i32)
    }
    fn clone_with_rotation(&self, rot: Rotation) -> Self {
        if rot == Rotation::North {
            return self.clone();
        }

        let center = self.get_center();
        let mut res = self.clone();
        let mat = rot.get_mat2();
        
        for block in res.blocks.iter_mut() {
            let boff = block.to_type::<f32>() - center;
            let rotated_boff = mat.mul_vec(boff);
            *block = (center + rotated_boff).map(|v| v as u16);
            // lower the shape by one if the rotation in 180°
            // to follow the DTET rotation standard
            if rot == Rotation::South && self.should_shape_be_lowered_on_180_rotation {
                block.y += 1;
            }
        }
        res
    }

    fn get_blocks_in_absolute_coordinates(&self, x: u16, y: u16) -> Vec<Vec2<u16>>{
        let center = self.get_rounded_center();
        let pos = Vec2{x, y};
        self.blocks.iter()
            .map(|v| (pos.to_type::<i32>() + v.to_type::<i32>() - center).map(|a| a as u16))
            .collect::<Vec<Vec2<u16>>>()
    }

    fn draw_to_framebuffer(&self, buff: &mut Framebuffer, x: u16, y: u16) {
        for b in self.get_blocks_in_absolute_coordinates(x, y).iter() {
            buff.set_pixel(b.x, b.y, self.render_as.clone());
        }
    }
}

struct Tetriminos {
    shape: TetrisShape,
    pos: Vec2<i32>,
    rotation: Rotation
}

impl Tetriminos {
    fn new(pos: Vec2<i32>, shape: TetrisShape) -> Self {
        Tetriminos {
            shape,
            pos,
            rotation: Rotation::North
        }
    }
    fn set_rotation(&mut self, rot: Rotation) {
        self.rotation = rot;
        self.shape = self.shape.clone_with_rotation(self.rotation);
    }
    fn rotateCW(&mut self) {
        self.set_rotation(self.rotation.rotateCW());
    }
    fn rotateCCW(&mut self) {
        self.set_rotation(self.rotation.rotateCCW());
    }
    fn draw_to_framebuffer(&self,framebuffer:&mut Framebuffer) {
        self.shape.draw_to_framebuffer(framebuffer, self.pos.x as u16, self.pos.y as u16);
    }
    fn into_leftover_block(self) {
        let poses = self.shape.get_blocks_in_absolute_coordinates(self.pos.x as u16, self.pos.y as u16);
    }
}

struct LeftoverBlock {
    pos: Vec2<i32>,
    render_as: Pixel
}

impl LeftoverBlock {
    fn draw_to_framebuffer(&self, framebuffer: &mut Framebuffer) {
        framebuffer.set_pixel(self.pos.x as u16, self.pos.y as u16, self.render_as);
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
    let mut xoff = 0u16;
    let mut yoff = 1u16;
    let margin = 1u16;
    let mut lastheight = 0u16;
    for shape in shapes.iter() {
        let shapequeue = [
            shape.clone_with_rotation(Rotation::North),
            shape.clone_with_rotation(Rotation::East),
            shape.clone_with_rotation(Rotation::South),
            shape.clone_with_rotation(Rotation::West)
        ];
        let bboxes = shapequeue.iter()
            .map(|v| v.get_bounding_box());
        let width = bboxes.clone().map(|bbox| bbox.width()).reduce(|a, b| a + b + margin).unwrap();
        let height = bboxes       .map(|bbox| bbox.height()).reduce(std::cmp::max).unwrap() + margin;
        lastheight = std::cmp::max(height, lastheight);
        if xoff as i32 + width as i32 + margin as i32 >= ctx.framebuffer.width as i32 {
            xoff = 0;
            yoff += lastheight;
            lastheight = 0;
        }

        for s in shapequeue.iter() {
            let bbox = s.get_bounding_box();
            let w = bbox.width();
            let off = Vec2 {
                x: xoff,
                y: yoff
            };
            let coord = off + s.get_rounded_center().map(|v| v as u16) - bbox.start;
            s.draw_to_framebuffer(&mut ctx.framebuffer, coord.x, coord.y);
            xoff += w + margin;
        }
    }
    ctx.framebuffer.make_space();
    ctx.render();
}
