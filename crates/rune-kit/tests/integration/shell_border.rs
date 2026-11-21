//! The main pane's border stays whole beside content that fills the pane to its edges: along
//! each edge and round each corner it looks as it does beside an empty part of the pane.
use rune::render::{Gpu, OFFSCREEN_FORMAT, Offscreen, Renderer};
use rune::{Color, Cx, El, Env, Runtime, col, el};
use rune_kit::Kit;
use rune_kit::shell::SplitView;

const W: f32 = 400.0;
const H: f32 = 300.0;
const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };

fn half_filled_pane(_: &(), _: &mut Cx<()>) -> El {
    let k = Kit::neutral(false);
    let main = col().w_full().child(el().w_full().grow(1.0)).child(el().w_full().grow(1.0).fill(RED));
    SplitView::new(main).build(&k)
}

#[test]
fn a_full_bleed_child_leaves_the_main_panes_border_whole() {
    let gpu = Gpu::headless();
    for scale in [1.0, 2.0] {
        let mut rt = Runtime::new((), half_filled_pane, (W, H), Env::new(false, scale));
        rt.frame(0.0);
        let card = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("main-pane")).expect("SplitView keys its main pane").rect;
        border_is_whole(&render(&gpu, &mut rt, scale), card, scale);
    }
}

struct Shot {
    rgba: Vec<u8>,
    width: usize,
}

impl Shot {
    fn px(&self, x: usize, y: usize) -> [u8; 3] {
        let i = (y * self.width + x) * 4;
        [self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]]
    }
}

fn render(gpu: &Gpu, rt: &mut Runtime<()>, scale: f32) -> Shot {
    let (w, h) = ((W * scale) as u32, (H * scale) as u32);
    let mut renderer = Renderer::new(&gpu.device, OFFSCREEN_FORMAT);
    let target = Offscreen::new(&gpu.device, w, h);
    renderer.prepare(&gpu.device, &gpu.queue, &mut rt.text.atlas, &rt.instances, (w, h));
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.draw(&mut encoder, &target.view, [0.0, 0.0, 0.0, 1.0]);
    target.copy(&mut encoder);
    renderer.submit(&gpu.queue, encoder);
    Shot { rgba: target.read(&gpu.device), width: w as usize }
}

fn close(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter().zip(b).all(|(&a, b)| a.abs_diff(b) <= 2)
}

fn border_is_whole(shot: &Shot, card: [f32; 4], scale: f32) {
    let [x, y, cw, ch] = card.map(|v| (v * scale).round() as usize);
    let (right, bottom) = (x + cw - 1, y + ch - 1);
    let (top_y, low_y) = (y + ch / 4, y + ch * 3 / 4);
    assert!(close(shot.px(x + cw / 2, low_y), [255, 0, 0]), "the block is red at {scale}x");
    let beside_empty = shot.px(right, top_y);
    assert!(!close(beside_empty, [255, 0, 0]), "the border beside the empty half isn't red at {scale}x");
    for (at, (px, py)) in [("right", (right, low_y)), ("left", (x, low_y)), ("bottom", (x + cw / 2, bottom))] {
        let got = shot.px(px, py);
        assert!(close(got, beside_empty), "the {at} edge beside the block is {got:?}, not the border's {beside_empty:?}, at {scale}x");
    }

    let r = Kit::neutral(false).radius.lg * scale;
    let border_px = scale.floor().max(1.0);
    let reaches_inside_border = |d: f32| d < r - border_px + 0.75;
    let side = r.ceil() as usize + 2;
    for (name, left) in [("bottom-left", true), ("bottom-right", false)] {
        let centre_x = if left { x as f32 + r } else { (x + cw) as f32 - r };
        let centre_y = (y + ch) as f32 - r;
        for dy in 0..side {
            for dx in 0..side {
                let px = if left { x + dx } else { right - dx };
                let py = bottom - dy;
                let (ox, oy) = (px as f32 + 0.5 - centre_x, py as f32 + 0.5 - centre_y);
                let in_corner = (if left { -ox } else { ox }) > 0.0 && oy > 0.0;
                if !in_corner || reaches_inside_border(ox.hypot(oy)) {
                    continue;
                }
                let (got, want) = (shot.px(px, py), shot.px(px, y + dy));
                assert!(close(got, want), "the {name} corner at ({dx}, {dy}) is {got:?}, the top corner's is {want:?}, at {scale}x");
            }
        }
    }
}
