use cgmath::Vector2;
use super::renderer::{Renderer, Vertex};

type Ratio = (u32, u32);

macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        [
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            1.0
        ]
    }
}

fn ratio_f32(ratio: Ratio) -> f32 {
    ratio.0 as f32 / ratio.1 as f32
}

fn triangle(center: [f32; 2], size: f32, color: [f32; 4]) -> [Vertex; 3] {
    let v1 = Vertex {
        color,
        pos: [center[0], center[1] - size / 2.0],
    };
    let v2 = Vertex {
        color,
        pos: [center[0] - size/2.0, center[1] + size / 2.0],
    };
    let v3 = Vertex {
        color,
        pos: [center[0] + size/2.0, center[1] + size / 2.0],
    };

    [v1, v2, v3]
}

#[derive(Debug, Clone, PartialEq)]
pub struct SbTree {
    layers: Vec<Vec<SbLeaf>>,
}

impl SbTree {
    pub fn new() -> Self {
        let root = SbLeaf {
            ratio: (3, 2),
            left: (1, 1),
            right: (2, 1)
        };

        SbTree {
            layers: vec![vec![root]]
        }
    }

    pub fn add_layer(&mut self) {
        let mut layer = vec![];

        for leaf in self.layers.last().unwrap() {
            let (l, r) = children(*leaf);
            layer.push(l);
            layer.push(r);
        }

        self.layers.push(layer)
    }

    pub fn set_layers_number(&mut self, num: usize) {
        self.layers.truncate(1);
        for _ in 0..num {
            self.add_layer()
        }
    }

    pub fn draw(&self, screen_size: [f32; 2], renderer: &mut Renderer) {
        let size = 8.0;
        let colors = [
            rgb!(0x34, 0x3d, 0x46),
            rgb!(0xbf, 0x61, 0x6a),
        ];

        for (i, layer) in self.layers.iter().enumerate() {
            let y = screen_size[1] * 0.90 * (0.05 + i as f32 / (self.layers.len() - 1) as f32);
            for (i, point) in layer.iter().enumerate() {
                let x = screen_size[0] * ratio_f32(point.ratio).log2();
                renderer.render_triangle(triangle([x, y], size, colors[i % 2]));
            }
        }
    }
}

pub struct Grid {
    pub edos: Vec<usize>,
}

impl Grid {
    pub fn draw(&self, screen_size: [f32; 2], renderer: &mut Renderer) {
        let size = 2.0;
        let colors = [
            rgb!(0x4f, 0x5b, 0x66),
            rgb!(0xbf, 0x61, 0x6a),
            rgb!(0x8f, 0xa1, 0xb3),
        ];

        let (screen_skip, screen_height) = {
            if self.edos.len() > 1 {
                let screen_skip = screen_size[1] / 24.0;
                let screen_height = screen_size[1] - screen_skip;

                for (e, &bars) in self.edos.iter().enumerate() {
                    let step = screen_size[0] / bars as f32;
                    for i in 0..bars {
                        let a0 = Vector2::new(
                            i as f32 * step - size / 2.0,
                            0.0
                        );
                        let a1 = Vector2::new(
                            i as f32 * step + size / 2.0,
                            screen_skip
                        );
                        renderer.render_rect(a0, a1, colors[e % 3])
                    }
                }

                (screen_skip, screen_height)
            }
            else {
                (0.0, screen_size[1])
            }
        };

        let screen_step = screen_height / self.edos.len() as f32;
        for (e, &bars) in self.edos.iter().enumerate() {
            let step = screen_size[0] / bars as f32;
            for i in 0..bars {
                let a0 = Vector2::new(
                    i as f32 * step - size / 2.0,
                    screen_skip + e as f32 * screen_step
                );
                let a1 = Vector2::new(
                    i as f32 * step + size / 2.0,
                    screen_skip + (e as f32 + 1.0) * screen_step
                );
                renderer.render_rect(a0, a1, colors[e % 3])
            }
        }
    }
}

pub struct Cursor {
    pub pos: f32,
}

impl Cursor {
    pub fn set_position(&mut self, pos: f32) {
        self.pos = pos
    }

    pub fn draw(&self, screen_size: [f32; 2], renderer: &mut Renderer) {
        let mut l_color = rgb![0xeb, 0xcb, 0x8b];
        l_color[3] = 0.7;
        let l_a0 = Vector2::new(self.pos - screen_size[0] / 240.0, 0.0);
        let l_a1 = Vector2::new(self.pos + screen_size[0] / 240.0, screen_size[1]);

        renderer.render_rect(l_a0, l_a1, l_color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SbLeaf {
    ratio: Ratio,
    left: Ratio,
    right: Ratio,
}

fn median(left: Ratio, right: Ratio) -> Ratio {
    (left.0 + right.0, left.1 + right.1)
}

fn children(parrent: SbLeaf) -> (SbLeaf, SbLeaf) {
    let left = SbLeaf {
        ratio: median(parrent.left, parrent.ratio),
        left: parrent.left,
        right: parrent.ratio,
    };
    let right = SbLeaf {
        ratio: median(parrent.ratio, parrent.right),
        left: parrent.ratio,
        right: parrent.right,
    };

    (left, right)
}
