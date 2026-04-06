use eframe::egui;
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

const LANES: usize = 7;
const CELL_W: f32 = 80.0;
const CELL_H: f32 = 70.0;
const COLS: usize = 6;
const TOP_BAR: f32 = 60.0;
const SIDEBAR: f32 = 100.0;
const TOTAL_WAVES: usize = 6;
const MINIWAVES: usize = 25;
const MINIWAVE_INTERVAL: f32 = 3.0;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([
                SIDEBAR + COLS as f32 * CELL_W,
                TOP_BAR + LANES as f32 * CELL_H,
            ])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Pencils vs Erasers",
        options,
        Box::new(|_cc| Ok(Box::new(Game::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(Game::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

// --- Defenders ---

#[derive(Clone, Copy, PartialEq)]
enum DefenderKind {
    Pencil,
    Notebook,
    Highlighter,
    Pen,
    Marker,
}

impl DefenderKind {
    fn cost(self) -> i32 {
        match self {
            Self::Pencil => 100,
            Self::Notebook => 50,
            Self::Highlighter => 125,
            Self::Pen => 200,
            Self::Marker => 600,
        }
    }
    fn hp(self) -> f32 {
        match self {
            Self::Pencil => 80.0,
            Self::Notebook => 300.0,
            Self::Highlighter => 60.0,
            Self::Pen => 70.0,
            Self::Marker => 50.0,
        }
    }
    fn color(self) -> egui::Color32 {
        match self {
            Self::Pencil => egui::Color32::from_rgb(140, 140, 150),
            Self::Notebook => egui::Color32::from_rgb(100, 140, 200),
            Self::Highlighter => egui::Color32::from_rgb(220, 255, 50),
            Self::Pen => egui::Color32::from_rgb(30, 30, 35),
            Self::Marker => egui::Color32::from_rgb(200, 40, 40),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Pencil => "Pencil",
            Self::Notebook => "Notebook",
            Self::Highlighter => "Highlight",
            Self::Pen => "Pen",
            Self::Marker => "Marker",
        }
    }
    fn symbol(self) -> &'static str {
        match self {
            Self::Pencil => "/",
            Self::Notebook => "[]",
            Self::Highlighter => "//",
            Self::Pen => "|",
            Self::Marker => "!",
        }
    }
}

struct Defender {
    kind: DefenderKind,
    lane: usize,
    col: usize,
    hp: f32,
    attack_timer: f32,
    attack_flash: f32,
}
impl Defender {
    fn new(kind: DefenderKind, lane: usize, col: usize) -> Self {
        Self {
            kind,
            lane,
            col,
            hp: kind.hp(),
            attack_timer: 0.0,
            attack_flash: 0.0,
        }
    }
}

struct Projectile {
    lane: usize,
    x: f32,
    damage: f32,
    pierce: bool,
    hits: usize,
    is_highlighter: bool,
}

// --- Enemies ---

#[derive(Clone, Copy, PartialEq, Debug)]
enum EnemyKind {
    PinkEraser,
    WhitePolymer,
    WrappedPolymer,
    BlackEraser,
    WrappedBlackEraser,
    ElectronicEraser,
    EraserHolder,
    BlueEraserHolder,
    Scissors,
    BlueScissors,
    WhiteOut,
    KneadedEraser,
    ShinyPlastic,
    LargeShinyPlastic,
}

impl EnemyKind {
    fn spawn_cost(self) -> f32 {
        match self {
            Self::PinkEraser => 1.0,
            Self::WhitePolymer | Self::ShinyPlastic => 2.0,
            Self::EraserHolder => 2.5,
            Self::WrappedPolymer | Self::KneadedEraser => 3.0,
            Self::Scissors => 3.5,
            Self::BlueEraserHolder | Self::LargeShinyPlastic => 4.0,
            Self::BlueScissors => 5.5,
            Self::BlackEraser => 5.0,
            Self::WhiteOut => 5.5,
            Self::WrappedBlackEraser => 6.0,
            Self::ElectronicEraser => 7.0,
        }
    }
    fn is_scissors(self) -> bool {
        matches!(self, Self::Scissors | Self::BlueScissors)
    }
    fn instant_kills(self) -> bool {
        matches!(
            self,
            Self::Scissors | Self::BlueScissors | Self::WhiteOut | Self::ElectronicEraser
        )
    }
    fn is_holder(self) -> bool {
        matches!(self, Self::EraserHolder | Self::BlueEraserHolder)
    }
    fn is_shiny(self) -> bool {
        matches!(self, Self::ShinyPlastic | Self::LargeShinyPlastic)
    }
    fn is_black(self) -> bool {
        matches!(self, Self::BlackEraser | Self::WrappedBlackEraser)
    }

    fn base_hp(self) -> f32 {
        match self {
            Self::PinkEraser => 100.0,
            Self::WhitePolymer | Self::WrappedPolymer => 200.0,
            Self::BlackEraser | Self::WrappedBlackEraser => 500.0,
            Self::EraserHolder => 100.0,
            Self::BlueEraserHolder => 200.0,
            Self::Scissors => 100.0,
            Self::BlueScissors => 200.0,
            Self::WhiteOut => 300.0,
            Self::KneadedEraser => 200.0,
            Self::ShinyPlastic => 100.0,
            Self::LargeShinyPlastic => 200.0,
            Self::ElectronicEraser => 500.0,
        }
    }
    fn shell_hp(self) -> f32 {
        match self {
            Self::WrappedPolymer => 100.0,
            Self::WrappedBlackEraser => 300.0,
            _ => 0.0,
        }
    }
    fn base_speed(self) -> f32 {
        match self {
            Self::ElectronicEraser => 45.0,
            Self::Scissors => 40.0,
            Self::BlueScissors => 35.0,
            Self::WhiteOut => 30.0,
            Self::ShinyPlastic | Self::LargeShinyPlastic => 22.0,
            Self::KneadedEraser => 20.0,
            _ => 20.0,
        }
    }
    fn base_damage(self) -> f32 {
        match self {
            Self::BlueEraserHolder => 30.0,
            Self::ShinyPlastic => 40.0,
            Self::LargeShinyPlastic => 60.0,
            _ if self.instant_kills() => 0.0,
            _ => 20.0,
        }
    }
    fn attack_range(self) -> f32 {
        match self {
            Self::EraserHolder => CELL_W,
            Self::BlueEraserHolder => CELL_W * 2.0,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum EnemyState {
    Moving,
    Eating,
    HolderStopping(f32),
    HolderExtending,
    HolderAttacking,
    Retreating,
}

struct Enemy {
    kind: EnemyKind,
    lane: usize,
    x: f32,
    hp: f32,
    shell_hp: f32,
    speed: f32,
    damage: f32,
    attack_timer: f32,
    attack_flash: f32,
    hit_flash: f32,
    state: EnemyState,
    has_jumped: bool,
    should_jump: bool,
    extend_offset: f32,
    extend_target: f32,
}

impl Enemy {
    fn new(kind: EnemyKind, lane: usize) -> Self {
        Self {
            kind,
            lane,
            x: COLS as f32 * CELL_W + SIDEBAR,
            hp: kind.base_hp(),
            shell_hp: kind.shell_hp(),
            speed: kind.base_speed(),
            damage: kind.base_damage(),
            attack_timer: 0.0,
            attack_flash: 0.0,
            hit_flash: 0.0,
            state: EnemyState::Moving,
            has_jumped: false,
            should_jump: false,
            extend_offset: 0.0,
            extend_target: 0.0,
        }
    }

    fn take_damage(&mut self, amount: f32, from_highlighter: bool) {
        if self.kind.is_shiny() && from_highlighter {
            return;
        }

        // Black erasers: flag jump (processed in update with access to defenders)
        if self.kind.is_black() && !self.has_jumped {
            self.has_jumped = true;
            self.should_jump = true;
        }

        self.hit_flash = 0.15;

        if self.shell_hp > 0.0 {
            self.shell_hp -= amount;
            if self.shell_hp <= 0.0 {
                let overflow = -self.shell_hp;
                self.shell_hp = 0.0;
                self.hp -= overflow;
            }
        } else {
            self.hp -= amount;
        }
    }

    fn alive(&self) -> bool {
        self.hp > 0.0
    }
}

// --- Pathways ---

#[derive(Clone)]
enum PathwayLevel {
    Single(EnemyKind),
    CoinFlip(EnemyKind, EnemyKind),
}

impl PathwayLevel {
    fn pick(&self, rng: &mut impl Rng) -> EnemyKind {
        match self {
            Self::Single(k) => *k,
            Self::CoinFlip(a, b) => {
                if rng.random_range(0.0_f32..1.0) < 0.5 {
                    *a
                } else {
                    *b
                }
            }
        }
    }
}

#[derive(Clone)]
struct Pathway {
    levels: Vec<PathwayLevel>,
}

fn create_pathways() -> Vec<Pathway> {
    vec![
        // A: Pink -> Polymer/Wrapped -> Black/WrappedBlack
        Pathway {
            levels: vec![
                PathwayLevel::Single(EnemyKind::PinkEraser),
                PathwayLevel::CoinFlip(EnemyKind::WhitePolymer, EnemyKind::WrappedPolymer),
                PathwayLevel::CoinFlip(EnemyKind::BlackEraser, EnemyKind::WrappedBlackEraser),
            ],
        },
        // B: EraserHolder -> BlueEraserHolder
        Pathway {
            levels: vec![
                PathwayLevel::Single(EnemyKind::EraserHolder),
                PathwayLevel::Single(EnemyKind::BlueEraserHolder),
            ],
        },
        // C: Scissors -> BlueScissors
        Pathway {
            levels: vec![
                PathwayLevel::Single(EnemyKind::Scissors),
                PathwayLevel::Single(EnemyKind::BlueScissors),
            ],
        },
        // D: WhiteOut -> ElectronicEraser
        Pathway {
            levels: vec![
                PathwayLevel::Single(EnemyKind::WhiteOut),
                PathwayLevel::Single(EnemyKind::ElectronicEraser),
            ],
        },
        // E: KneadedEraser
        Pathway {
            levels: vec![PathwayLevel::Single(EnemyKind::KneadedEraser)],
        },
        // F: ShinyPlastic -> LargeShinyPlastic
        Pathway {
            levels: vec![
                PathwayLevel::Single(EnemyKind::ShinyPlastic),
                PathwayLevel::Single(EnemyKind::LargeShinyPlastic),
            ],
        },
    ]
}

// --- Tools ---

#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Place(DefenderKind),
    InkBlob,
    PencilCase,
}

// --- Game ---

#[derive(PartialEq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

// Difficulty ramp per wave: (start, end) - linearly interpolated across miniwaves
fn wave_difficulty(wave: usize, miniwave: usize) -> f32 {
    let (start, end) = match wave {
        1 => (0.5, 1.5),
        2 => (2.0, 2.5),
        3 => (3.0, 3.5),
        4 => (4.5, 5.5),
        5 => (6.5, 8.0),
        6 => (9.5, 11.5),
        _ => (9.5, 11.5),
    };
    let t = miniwave as f32 / (MINIWAVES - 1).max(1) as f32;
    start + (end - start) * t
}

#[cfg(not(target_arch = "wasm32"))]
fn highscore_path() -> PathBuf {
    let mut p = std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    p.push("pvz_erasers_highscore.txt");
    p
}

#[cfg(not(target_arch = "wasm32"))]
fn load_highscore() -> Option<f64> {
    std::fs::read_to_string(highscore_path())
        .ok()?
        .trim()
        .parse()
        .ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn save_highscore(time: f64) {
    let _ = std::fs::write(highscore_path(), format!("{:.2}", time));
}

#[cfg(target_arch = "wasm32")]
fn load_highscore() -> Option<f64> {
    None
}

#[cfg(target_arch = "wasm32")]
fn save_highscore(_time: f64) {}

struct Game {
    defenders: Vec<Defender>,
    projectiles: Vec<Projectile>,
    enemies: Vec<Enemy>,
    ink: i32,
    ink_timer: f32,
    selected: Option<Tool>,
    wave: usize,
    state: GameState,
    time: f64,
    ink_blob_available: bool,
    bonus_msg_timer: f32,
    bonus_msg_amount: i32,

    // Pathway system
    pathways: Vec<Pathway>,
    pathway_introduced: Vec<bool>,
    pathway_level: Vec<usize>,

    // Mini-wave system
    miniwave: usize,
    miniwave_timer: f32,
    wallet: f32,
    pending_purchase: Option<(EnemyKind, f32)>,
    boosted_pathways: Vec<usize>,
    start_intro_done: bool,
    mid_intro_done: bool,
    coin_was_heads: bool,

    // Foreshadow
    foreshadow_r: Option<usize>,
    preview_miniwave: usize,
    preview_sent: bool,
    preview_pathway: Option<usize>,
    preview_cost: f32,

    // Misc
    burst_timer: f32,
    initial_delay: f32,
    timeskip_given: bool,
    invested: bool,
    high_score: Option<f64>,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        let mut rng = rand::rng();
        let pathways = create_pathways();
        let num_p = pathways.len();
        let mut pathway_introduced = vec![false; num_p];
        pathway_introduced[0] = true; // Pathway A starts introduced
        let preview_mw = rng.random_range(0..MINIWAVES as u32) as usize;

        Self {
            defenders: Vec::new(),
            projectiles: Vec::new(),
            enemies: Vec::new(),
            ink: 200,
            ink_timer: 0.0,
            selected: Some(Tool::Place(DefenderKind::Pencil)),
            wave: 1,
            state: GameState::Playing,
            time: 0.0,
            ink_blob_available: true,
            bonus_msg_timer: 0.0,
            bonus_msg_amount: 0,
            pathways,
            pathway_introduced,
            pathway_level: vec![0; num_p],
            miniwave: 0,
            miniwave_timer: 0.0,
            wallet: 0.0,
            pending_purchase: None,
            boosted_pathways: Vec::new(),
            start_intro_done: false,
            mid_intro_done: false,
            coin_was_heads: true, // wave 1 always heads
            foreshadow_r: None,
            preview_miniwave: preview_mw,
            preview_sent: false,
            preview_pathway: None,
            preview_cost: 0.0,
            burst_timer: 0.0,
            initial_delay: 2.0,
            timeskip_given: false,
            invested: false,
            high_score: load_highscore(),
            paused: false,
        }
    }

    fn upgradeable_pathways(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for i in 0..self.pathways.len() {
            if !self.pathway_introduced[i] {
                result.push(i);
            } else if self.pathway_level[i] + 1 < self.pathways[i].levels.len() {
                // ElectronicEraser only available in wave 6+
                let next_level = self.pathway_level[i] + 1;
                let has_electronic = matches!(
                    self.pathways[i].levels[next_level],
                    PathwayLevel::Single(EnemyKind::ElectronicEraser)
                        | PathwayLevel::CoinFlip(EnemyKind::ElectronicEraser, _)
                        | PathwayLevel::CoinFlip(_, EnemyKind::ElectronicEraser)
                );
                if has_electronic && self.wave < 6 {
                    continue;
                }
                result.push(i);
            }
        }
        result
    }

    fn introduce_or_upgrade(&mut self, pidx: usize) {
        if !self.pathway_introduced[pidx] {
            self.pathway_introduced[pidx] = true;
        } else if self.pathway_level[pidx] + 1 < self.pathways[pidx].levels.len() {
            self.pathway_level[pidx] += 1;
        }
    }

    // Returns Some(pathway) or None (= invest). None with no pathways also possible.
    fn pick_weighted_pathway(&self, rng: &mut impl Rng, allow_invest: bool) -> Option<usize> {
        let mut options: Vec<(Option<usize>, f32)> = Vec::new();
        for i in 0..self.pathways.len() {
            if self.pathway_introduced[i] {
                let w = if self.boosted_pathways.contains(&i) {
                    2.0
                } else {
                    1.0
                };
                options.push((Some(i), w));
            }
        }
        if options.is_empty() {
            return None;
        }
        if allow_invest {
            options.push((None, 1.0));
        }
        let total: f32 = options.iter().map(|(_, w)| w).sum();
        let mut r = rng.random_range(0.0..total);
        for &(idx, w) in &options {
            r -= w;
            if r <= 0.0 {
                return idx;
            }
        }
        options.last().unwrap().0
    }

    fn process_miniwave(&mut self) {
        let mut rng = rand::rng();

        // If invested last mini-wave, wallet carries over as-is (no doubling)
        if self.invested {
            self.invested = false;
        }

        // Add income (difficulty ramps within wave), minus preview cost spread across miniwaves
        self.wallet += wave_difficulty(self.wave, self.miniwave)
            - self.preview_cost / MINIWAVES as f32;

        // Start-of-wave introduction (miniwave 0)
        if self.miniwave == 0 && !self.start_intro_done {
            self.start_intro_done = true;

            if self.wave == 1 {
                // Wave 1: pinks already in, boost pathway A
                self.boosted_pathways.push(0);
            } else if let Some(r_pidx) = self.foreshadow_r {
                if self.coin_was_heads {
                    // Heads: r joins at start with 2x
                    self.introduce_or_upgrade(r_pidx);
                    self.boosted_pathways.push(r_pidx);
                    self.foreshadow_r = None;
                } else {
                    // Tails: new type at start with 2x, r saved for mid-wave
                    let upgradeable = self.upgradeable_pathways();
                    let others: Vec<usize> =
                        upgradeable.into_iter().filter(|&p| p != r_pidx).collect();
                    if !others.is_empty() {
                        let pidx = others[rng.random_range(0..others.len())];
                        self.introduce_or_upgrade(pidx);
                        self.boosted_pathways.push(pidx);
                    }
                    // r stays for mid-wave
                }
            } else {
                // No foreshadow, introduce random
                let upgradeable = self.upgradeable_pathways();
                if !upgradeable.is_empty() {
                    let pidx = upgradeable[rng.random_range(0..upgradeable.len())];
                    self.introduce_or_upgrade(pidx);
                    self.boosted_pathways.push(pidx);
                }
            }
        }

        // Mid-wave introduction (miniwave 15)
        if self.miniwave == MINIWAVES / 2 && !self.mid_intro_done {
            self.mid_intro_done = true;

            if self.wave == 1 || self.coin_was_heads {
                // Introduce new random type with 2x
                let upgradeable = self.upgradeable_pathways();
                if !upgradeable.is_empty() {
                    let pidx = upgradeable[rng.random_range(0..upgradeable.len())];
                    self.introduce_or_upgrade(pidx);
                    self.boosted_pathways.push(pidx);
                }
            } else {
                // Tails: r joins at mid-wave with 2x
                if let Some(r_pidx) = self.foreshadow_r {
                    self.introduce_or_upgrade(r_pidx);
                    self.boosted_pathways.push(r_pidx);
                    self.foreshadow_r = None;
                }
            }
        }

        // Foreshadow preview (random miniwave)
        if self.miniwave == self.preview_miniwave && !self.preview_sent {
            self.preview_sent = true;
            let upgradeable = self.upgradeable_pathways();
            if !upgradeable.is_empty() {
                let pidx = upgradeable[rng.random_range(0..upgradeable.len())];
                let level = if self.pathway_introduced[pidx] {
                    self.pathway_level[pidx] + 1
                } else {
                    0
                };
                if level < self.pathways[pidx].levels.len() {
                    let kind = self.pathways[pidx].levels[level].pick(&mut rng);
                    let lane = rng.random_range(0..LANES);
                    self.preview_cost = kind.spawn_cost();
                    self.enemies.push(Enemy::new(kind, lane));
                    self.preview_pathway = Some(pidx);
                }
            }
        }

        // Spending loop
        let mut spawns: Vec<Enemy> = Vec::new();

        // First try pending purchase
        if let Some((kind, cost)) = self.pending_purchase {
            if self.wallet >= cost {
                self.wallet -= cost;
                let lane = rng.random_range(0..LANES);
                spawns.push(Enemy::new(kind, lane));
                self.pending_purchase = None;
            } else {
                self.enemies.extend(spawns);
                return;
            }
        }

        // Keep buying (invest is in the pathway pool)
        let mut first_pick = true;
        loop {
            let allow_invest = first_pick && self.pending_purchase.is_none();
            match self.pick_weighted_pathway(&mut rng, allow_invest) {
                None => {
                    if allow_invest {
                        self.invested = true;
                    }
                    break;
                }
                Some(pidx) => {
                    first_pick = false;
                    let level = self.pathway_level[pidx];
                    let kind = self.pathways[pidx].levels[level].pick(&mut rng);
                    let cost = kind.spawn_cost();

                    if self.wallet >= cost {
                        self.wallet -= cost;
                        let lane = rng.random_range(0..LANES);
                        spawns.push(Enemy::new(kind, lane));
                    } else {
                        self.pending_purchase = Some((kind, cost));
                        break;
                    }
                }
            }
        }

        self.enemies.extend(spawns);
    }

    fn draw_defender_sprite(
        painter: &egui::Painter,
        kind: DefenderKind,
        c: egui::Pos2,
        scale: f32,
    ) {
        let s = scale;
        match kind {
            DefenderKind::Pencil => {
                let body = egui::Rect::from_min_size(
                    egui::pos2(c.x - 18.0 * s, c.y - 5.0 * s),
                    egui::vec2(28.0 * s, 10.0 * s),
                );
                painter.rect_filled(body, 1.0 * s, egui::Color32::from_rgb(220, 200, 60));
                painter.rect_stroke(
                    body,
                    1.0 * s,
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(180, 160, 40)),
                    egui::StrokeKind::Outside,
                );
                let tip = vec![
                    egui::pos2(body.max.x, c.y - 5.0 * s),
                    egui::pos2(body.max.x + 12.0 * s, c.y),
                    egui::pos2(body.max.x, c.y + 5.0 * s),
                ];
                painter.add(egui::Shape::convex_polygon(
                    tip,
                    egui::Color32::from_rgb(210, 180, 140),
                    egui::Stroke::NONE,
                ));
                let gpt = vec![
                    egui::pos2(body.max.x + 8.0 * s, c.y - 2.0 * s),
                    egui::pos2(body.max.x + 12.0 * s, c.y),
                    egui::pos2(body.max.x + 8.0 * s, c.y + 2.0 * s),
                ];
                painter.add(egui::Shape::convex_polygon(
                    gpt,
                    egui::Color32::from_rgb(60, 60, 60),
                    egui::Stroke::NONE,
                ));
                let eraser = egui::Rect::from_min_size(
                    egui::pos2(body.min.x - 6.0 * s, c.y - 5.0 * s),
                    egui::vec2(6.0 * s, 10.0 * s),
                );
                painter.rect_filled(eraser, 1.0 * s, egui::Color32::from_rgb(240, 150, 150));
                let band = egui::Rect::from_min_size(
                    egui::pos2(body.min.x - 1.0 * s, c.y - 5.0 * s),
                    egui::vec2(3.0 * s, 10.0 * s),
                );
                painter.rect_filled(band, 0.0, egui::Color32::from_rgb(180, 180, 190));
            }
            DefenderKind::Pen => {
                let body = egui::Rect::from_min_size(
                    egui::pos2(c.x - 18.0 * s, c.y - 4.0 * s),
                    egui::vec2(30.0 * s, 8.0 * s),
                );
                painter.rect_filled(body, 1.0 * s, egui::Color32::from_rgb(25, 25, 35));
                painter.rect_stroke(
                    body,
                    1.0 * s,
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 60, 70)),
                    egui::StrokeKind::Outside,
                );
                let tip = vec![
                    egui::pos2(body.max.x, c.y - 4.0 * s),
                    egui::pos2(body.max.x + 10.0 * s, c.y - 1.5 * s),
                    egui::pos2(body.max.x + 14.0 * s, c.y),
                    egui::pos2(body.max.x + 10.0 * s, c.y + 1.5 * s),
                    egui::pos2(body.max.x, c.y + 4.0 * s),
                ];
                painter.add(egui::Shape::convex_polygon(
                    tip,
                    egui::Color32::from_rgb(50, 50, 60),
                    egui::Stroke::NONE,
                ));
                painter.circle_filled(
                    egui::pos2(body.max.x + 14.0 * s, c.y),
                    1.5 * s,
                    egui::Color32::from_rgb(180, 180, 190),
                );
                painter.line_segment(
                    [
                        egui::pos2(c.x - 8.0 * s, c.y - 4.0 * s),
                        egui::pos2(c.x + 8.0 * s, c.y - 4.0 * s),
                    ],
                    egui::Stroke::new(2.0 * s, egui::Color32::from_rgb(180, 180, 190)),
                );
                let cap = egui::Rect::from_min_size(
                    egui::pos2(body.min.x - 4.0 * s, c.y - 4.0 * s),
                    egui::vec2(4.0 * s, 8.0 * s),
                );
                painter.rect_filled(cap, 2.0 * s, egui::Color32::from_rgb(25, 25, 35));
            }
            DefenderKind::Notebook => {
                let r = egui::Rect::from_center_size(c, egui::vec2(30.0 * s, 40.0 * s));
                painter.rect_filled(r, 3.0 * s, egui::Color32::from_rgb(100, 140, 200));
                painter.rect_stroke(
                    r,
                    3.0 * s,
                    egui::Stroke::new(1.5 * s, egui::Color32::from_rgb(60, 90, 150)),
                    egui::StrokeKind::Outside,
                );
                for j in 0..3 {
                    let y = r.min.y + 10.0 * s + j as f32 * 10.0 * s;
                    painter.line_segment(
                        [
                            egui::pos2(r.min.x + 5.0 * s, y),
                            egui::pos2(r.max.x - 5.0 * s, y),
                        ],
                        egui::Stroke::new(0.5 * s, egui::Color32::from_rgb(70, 100, 160)),
                    );
                }
            }
            DefenderKind::Highlighter => {
                // Slim body
                let r = egui::Rect::from_center_size(c, egui::vec2(30.0 * s, 10.0 * s));
                painter.rect_filled(r, 2.0 * s, egui::Color32::from_rgb(220, 255, 50));
                painter.rect_stroke(
                    r,
                    2.0 * s,
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(180, 210, 30)),
                    egui::StrokeKind::Outside,
                );
                // Chisel tip
                let pts = vec![
                    egui::pos2(r.max.x, c.y - 5.0 * s),
                    egui::pos2(r.max.x + 10.0 * s, c.y - 2.0 * s),
                    egui::pos2(r.max.x + 10.0 * s, c.y + 2.0 * s),
                    egui::pos2(r.max.x, c.y + 5.0 * s),
                ];
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    egui::Color32::from_rgb(200, 240, 30),
                    egui::Stroke::NONE,
                ));
                // Cap end
                let cap = egui::Rect::from_min_size(
                    egui::pos2(r.min.x - 4.0 * s, c.y - 5.0 * s),
                    egui::vec2(4.0 * s, 10.0 * s),
                );
                painter.rect_filled(cap, 1.0 * s, egui::Color32::from_rgb(180, 210, 30));
            }
            DefenderKind::Marker => {
                // Dark body (like a Sharpie)
                let body = egui::Rect::from_min_size(
                    egui::pos2(c.x - 16.0 * s, c.y - 5.0 * s),
                    egui::vec2(26.0 * s, 10.0 * s),
                );
                painter.rect_filled(body, 2.0 * s, egui::Color32::from_rgb(40, 40, 45));
                painter.rect_stroke(
                    body,
                    2.0 * s,
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(25, 25, 30)),
                    egui::StrokeKind::Outside,
                );
                // Tapered tip section
                let tip = vec![
                    egui::pos2(body.max.x, c.y - 5.0 * s),
                    egui::pos2(body.max.x + 8.0 * s, c.y - 3.0 * s),
                    egui::pos2(body.max.x + 8.0 * s, c.y + 3.0 * s),
                    egui::pos2(body.max.x, c.y + 5.0 * s),
                ];
                painter.add(egui::Shape::convex_polygon(
                    tip,
                    egui::Color32::from_rgb(50, 50, 55),
                    egui::Stroke::NONE,
                ));
                // Felt tip
                let felt = egui::Rect::from_min_size(
                    egui::pos2(body.max.x + 8.0 * s, c.y - 2.0 * s),
                    egui::vec2(4.0 * s, 4.0 * s),
                );
                painter.rect_filled(felt, 0.5 * s, egui::Color32::from_rgb(70, 70, 75));
                // Red cap on the back
                let cap = egui::Rect::from_min_size(
                    egui::pos2(body.min.x - 6.0 * s, c.y - 6.0 * s),
                    egui::vec2(6.0 * s, 12.0 * s),
                );
                painter.rect_filled(cap, 2.0 * s, egui::Color32::from_rgb(200, 40, 40));
                // Red band/ring on body
                let band = egui::Rect::from_min_size(
                    egui::pos2(body.min.x + 2.0 * s, c.y - 5.0 * s),
                    egui::vec2(4.0 * s, 10.0 * s),
                );
                painter.rect_filled(band, 0.0, egui::Color32::from_rgb(200, 40, 40));
            }
        }
    }

    fn grid_origin() -> egui::Pos2 {
        egui::pos2(SIDEBAR, TOP_BAR)
    }
    fn cell_center(col: usize, lane: usize) -> egui::Pos2 {
        let o = Self::grid_origin();
        egui::pos2(
            o.x + col as f32 * CELL_W + CELL_W / 2.0,
            o.y + lane as f32 * CELL_H + CELL_H / 2.0,
        )
    }

    // Find nearest defender distance for a holder enemy
    fn save_score(&mut self) {
        if self.high_score.is_none_or(|hs| self.time > hs) {
            self.high_score = Some(self.time);
            save_highscore(self.time);
        }
    }

    fn nearest_defender_dist(
        defenders: &[Defender],
        lane: usize,
        enemy_x: f32,
        max_range: f32,
    ) -> Option<f32> {
        let grid_x = SIDEBAR;
        let mut best: Option<f32> = None;
        for d in defenders {
            if d.lane == lane {
                let dx = grid_x + d.col as f32 * CELL_W + CELL_W / 2.0;
                let dist = enemy_x - dx;
                if dist > 0.0 && dist <= max_range {
                    best = Some(best.map_or(dist, |b: f32| b.min(dist)));
                }
            }
        }
        best
    }

    fn update(&mut self, dt: f32) {
        if self.state != GameState::Playing {
            return;
        }
        self.bonus_msg_timer = (self.bonus_msg_timer - dt).max(0.0);
        if self.bonus_msg_timer <= 0.0 {
            self.bonus_msg_amount = 0;
        }

        // Initial delay — timer paused, no passive income (player gets starting ink instead)
        if self.initial_delay > 0.0 {
            self.initial_delay -= dt;
            return;
        }

        self.time += dt as f64;

        // Ink generation — only when enemies are on screen
        if !self.enemies.is_empty() {
            self.ink_timer += dt;
            if self.ink_timer >= 1.5 {
                self.ink_timer -= 1.5;
                self.ink += 25;
            }
        }

        // Mini-wave spawning
        self.miniwave_timer += dt;
        if self.miniwave_timer >= MINIWAVE_INTERVAL && self.miniwave < MINIWAVES {
            self.miniwave_timer -= MINIWAVE_INTERVAL;
            self.process_miniwave();
            self.miniwave += 1;
            self.timeskip_given = false;
        }

        // Timeskip: instantly spawn next miniwave when enemies are cleared
        // Don't give bonus if AI just invested (no enemies were spawned)
        if self.enemies.is_empty()
            && self.miniwave > 0
            && self.miniwave < MINIWAVES
            && !self.timeskip_given
        {
            self.miniwave_timer = MINIWAVE_INTERVAL;
            if !self.invested {
                self.ink += 75;
                self.bonus_msg_amount += 75;
                self.bonus_msg_timer = 1.5;
            }
            self.timeskip_given = true;
        }

        // Wave completion — starts immediately when all miniwaves have been spawned
        // (remaining enemies carry over into next wave)
        if self.miniwave >= MINIWAVES {
            if self.wave >= TOTAL_WAVES {
                if self.enemies.is_empty() {
                    self.state = GameState::Won;
                    self.save_score();
                    return;
                }
                // Last wave: don't start a new wave, let remaining enemies play out
            } else {
                // Foreshadow transition: preview becomes next round's r
                self.foreshadow_r = self.preview_pathway;

                self.wave += 1;
                self.miniwave = 0;
                self.miniwave_timer = MINIWAVE_INTERVAL;
                self.wallet = 0.0;
                self.pending_purchase = None;
                self.boosted_pathways.clear();
                self.start_intro_done = false;
                self.mid_intro_done = false;
                self.preview_sent = false;
                self.preview_pathway = None;
                self.preview_cost = 0.0;
                self.burst_timer = 0.0;
                self.timeskip_given = false;
                self.invested = false;

                let mut rng = rand::rng();
                self.coin_was_heads = rng.random_range(0.0_f32..1.0) < 0.5;
                self.preview_miniwave = rng.random_range(0..MINIWAVES as u32) as usize;
            }
        }

        // Late-game scissors bursts (waves 5-6)
        if self.wave >= 5 {
            self.burst_timer += dt;
            if self.burst_timer >= 12.0 {
                self.burst_timer = 0.0;
                let mut rng = rand::rng();
                let count = if self.wave >= 6 { 3 } else { 2 };
                for _ in 0..count {
                    let k = if rng.random_range(0.0_f32..1.0) < 0.5 {
                        EnemyKind::BlueScissors
                    } else {
                        EnemyKind::Scissors
                    };
                    let lane = rng.random_range(0..LANES);
                    self.enemies.push(Enemy::new(k, lane));
                }
            }
        }

        // --- Defender attacks ---
        let mut new_proj = Vec::new();
        for def in &mut self.defenders {
            def.attack_timer += dt;
            def.attack_flash = (def.attack_flash - dt).max(0.0);
            match def.kind {
                DefenderKind::Pencil => {
                    if def.attack_timer >= 0.8 {
                        let cx = Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W;
                        let range = CELL_W * 1.15;
                        for enemy in &mut self.enemies {
                            if enemy.lane == def.lane
                                && enemy.x > cx - 10.0
                                && enemy.x < cx + range
                                && enemy.alive()
                            {
                                enemy.take_damage(20.0, false);
                                def.attack_timer = 0.0;
                                def.attack_flash = 0.15;
                                break;
                            }
                        }
                    }
                }
                DefenderKind::Pen => {
                    if def.attack_timer >= 0.8 {
                        let cx = Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W;
                        let range = CELL_W * 2.15;
                        for enemy in &mut self.enemies {
                            if enemy.lane == def.lane
                                && enemy.x > cx - 10.0
                                && enemy.x < cx + range
                                && enemy.alive()
                            {
                                enemy.take_damage(30.0, false);
                                def.attack_timer = 0.0;
                                def.attack_flash = 0.15;
                                break;
                            }
                        }
                    }
                }
                DefenderKind::Highlighter => {
                    if def.attack_timer >= 1.5 {
                        if self.enemies.iter().any(|e| e.lane == def.lane && e.alive()) {
                            def.attack_timer = 0.0;
                            let cx = Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W;
                            new_proj.push(Projectile {
                                lane: def.lane,
                                x: cx,
                                damage: 13.0,
                                pierce: true,
                                hits: 0,
                                is_highlighter: true,
                            });
                        }
                    }
                }
                DefenderKind::Marker => {
                    // Ranged, 3s cooldown, instant kills 1 normal wrapper (100 shell + overflow)
                    // Damage = wrapper shell (100) + some extra = 120 to break shell and hurt inside
                    if def.attack_timer >= 3.0 {
                        if self.enemies.iter().any(|e| e.lane == def.lane && e.alive()) {
                            def.attack_timer = 0.0;
                            def.attack_flash = 0.15;
                            let cx = Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W;
                            new_proj.push(Projectile {
                                lane: def.lane,
                                x: cx,
                                damage: 200.0,
                                pierce: false,
                                hits: 0,
                                is_highlighter: false,
                            });
                        }
                    }
                }
                DefenderKind::Notebook => {}
            }
        }
        self.projectiles.extend(new_proj);

        // Move projectiles
        for p in &mut self.projectiles {
            p.x += 200.0 * dt;
        }

        // Projectile-enemy collision
        for proj in &mut self.projectiles {
            for enemy in &mut self.enemies {
                if enemy.lane != proj.lane || (enemy.x - proj.x).abs() >= 15.0 || !enemy.alive() {
                    continue;
                }
                if enemy.kind.is_shiny() && proj.is_highlighter {
                    proj.x = -100.0;
                    break;
                }
                enemy.take_damage(proj.damage, proj.is_highlighter);
                proj.hits += 1;
                if !proj.pierce || proj.hits >= 4 {
                    proj.x = -100.0;
                    break;
                }
                proj.x = enemy.x + 16.0;
            }
        }
        self.projectiles
            .retain(|p| p.x > 0.0 && p.x < COLS as f32 * CELL_W + SIDEBAR + 100.0);

        // --- Black eraser jump processing (needs access to defenders) ---
        for enemy in &mut self.enemies {
            if enemy.should_jump {
                enemy.should_jump = false;
                let mut target_x = enemy.x - CELL_W * 2.0;
                // Don't jump past any defender in the same lane
                for def in self.defenders.iter() {
                    if def.lane == enemy.lane {
                        let dx = Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W / 2.0;
                        // If defender is in front (left) and we'd jump past it, stop just past it
                        if dx < enemy.x && dx >= target_x {
                            target_x = dx + CELL_W * 0.6;
                        }
                    }
                }
                enemy.x = target_x.max(SIDEBAR);
            }
        }

        // --- Enemy movement and combat ---
        for enemy in &mut self.enemies {
            enemy.attack_flash = (enemy.attack_flash - dt).max(0.0);
            enemy.hit_flash = (enemy.hit_flash - dt).max(0.0);

            match enemy.state {
                EnemyState::Moving => {
                    if enemy.kind.instant_kills() && enemy.kind != EnemyKind::WhiteOut {
                        // Scissors
                        enemy.x -= enemy.speed * dt;
                        for def in &mut self.defenders {
                            if def.lane == enemy.lane {
                                let dx =
                                    Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W / 2.0;
                                if (enemy.x - dx).abs() < CELL_W * 0.5 {
                                    def.hp = 0.0;
                                }
                            }
                        }
                    } else if enemy.kind == EnemyKind::WhiteOut
                        || enemy.kind == EnemyKind::ElectronicEraser
                    {
                        enemy.x -= enemy.speed * dt;
                        for def in &mut self.defenders {
                            if def.lane == enemy.lane {
                                let dx =
                                    Self::grid_origin().x + def.col as f32 * CELL_W + CELL_W / 2.0;
                                if (enemy.x - dx).abs() < CELL_W * 0.5 {
                                    def.hp = 0.0;
                                }
                            }
                        }
                        if enemy.x < SIDEBAR {
                            if enemy.kind == EnemyKind::WhiteOut {
                                enemy.state = EnemyState::Retreating;
                            } else {
                                // ElectronicEraser causes loss
                                self.state = GameState::Lost;
                                self.save_score();
                                return;
                            }
                        }
                    } else if enemy.kind.is_holder() {
                        let max_range = enemy.kind.attack_range() + CELL_W * 0.7;
                        if let Some(_dist) = Self::nearest_defender_dist(
                            &self.defenders,
                            enemy.lane,
                            enemy.x,
                            max_range,
                        ) {
                            enemy.state = EnemyState::HolderStopping(0.5);
                        } else {
                            enemy.x -= enemy.speed * dt;
                        }
                    } else {
                        let blocked = self.defenders.iter().any(|d| {
                            d.lane == enemy.lane && {
                                let dx =
                                    Self::grid_origin().x + d.col as f32 * CELL_W + CELL_W / 2.0;
                                (enemy.x - dx).abs() < CELL_W * 0.6 && enemy.x > dx
                            }
                        });
                        if blocked {
                            enemy.state = EnemyState::Eating;
                        } else {
                            enemy.x -= enemy.speed * dt;
                        }
                    }
                }
                EnemyState::Eating => {
                    let still_blocked = self.defenders.iter().any(|d| {
                        d.lane == enemy.lane && {
                            let dx = Self::grid_origin().x + d.col as f32 * CELL_W + CELL_W / 2.0;
                            (enemy.x - dx).abs() < CELL_W * 0.8 && enemy.x > dx
                        }
                    });
                    if !still_blocked {
                        enemy.state = EnemyState::Moving;
                        continue;
                    }
                    enemy.attack_timer += dt;
                    if enemy.attack_timer >= 0.8 {
                        enemy.attack_timer = 0.0;
                        enemy.attack_flash = 0.15;
                        let ex = enemy.x;
                        let lane = enemy.lane;
                        let dmg = enemy.damage;
                        if let Some(def) = self
                            .defenders
                            .iter_mut()
                            .filter(|d| {
                                d.lane == lane && {
                                    let dx = Self::grid_origin().x
                                        + d.col as f32 * CELL_W
                                        + CELL_W / 2.0;
                                    dx < ex // only attack defenders in front
                                }
                            })
                            .min_by(|a, b| {
                                let ax =
                                    Self::grid_origin().x + a.col as f32 * CELL_W + CELL_W / 2.0;
                                let bx =
                                    Self::grid_origin().x + b.col as f32 * CELL_W + CELL_W / 2.0;
                                (ex - ax).partial_cmp(&(ex - bx)).unwrap()
                            })
                        {
                            def.hp -= dmg;
                        }
                    }
                }
                EnemyState::HolderStopping(ref mut t) => {
                    *t -= dt;
                    // Check if defender still in range
                    let max_range = enemy.kind.attack_range() + CELL_W * 0.7;
                    let has_target = Self::nearest_defender_dist(
                        &self.defenders,
                        enemy.lane,
                        enemy.x,
                        max_range,
                    )
                    .is_some();
                    if !has_target {
                        enemy.state = EnemyState::Moving;
                    } else if *t <= 0.0 {
                        // Compute extend target based on nearest defender
                        if let Some(dist) = Self::nearest_defender_dist(
                            &self.defenders,
                            enemy.lane,
                            enemy.x,
                            max_range,
                        ) {
                            enemy.extend_target = (dist - CELL_W * 0.3)
                                .max(0.0)
                                .min(enemy.kind.attack_range());
                        }
                        enemy.state = EnemyState::HolderExtending;
                    }
                }
                EnemyState::HolderExtending => {
                    let max_range = enemy.kind.attack_range() + CELL_W * 0.7;
                    // Retarget if defender changed
                    if let Some(dist) =
                        Self::nearest_defender_dist(&self.defenders, enemy.lane, enemy.x, max_range)
                    {
                        enemy.extend_target = (dist - CELL_W * 0.3)
                            .max(0.0)
                            .min(enemy.kind.attack_range());
                    } else {
                        // No target, retract
                        enemy.state = EnemyState::Moving;
                        enemy.extend_offset = 0.0;
                        enemy.extend_target = 0.0;
                        continue;
                    }

                    // Extend toward target
                    let speed = enemy.kind.attack_range() * 2.0; // extend speed
                    if enemy.extend_offset < enemy.extend_target {
                        enemy.extend_offset =
                            (enemy.extend_offset + speed * dt).min(enemy.extend_target);
                    } else {
                        enemy.extend_offset =
                            (enemy.extend_offset - speed * dt).max(enemy.extend_target);
                    }

                    if (enemy.extend_offset - enemy.extend_target).abs() < 1.0 {
                        enemy.extend_offset = enemy.extend_target;
                        enemy.state = EnemyState::HolderAttacking;
                    }
                }
                EnemyState::HolderAttacking => {
                    let max_range = enemy.kind.attack_range() + CELL_W * 0.7;
                    // Retarget to nearest defender
                    if let Some(dist) =
                        Self::nearest_defender_dist(&self.defenders, enemy.lane, enemy.x, max_range)
                    {
                        let new_target = (dist - CELL_W * 0.3)
                            .max(0.0)
                            .min(enemy.kind.attack_range());
                        if (enemy.extend_offset - new_target).abs() > 1.0 {
                            // Need to readjust, go back to extending
                            enemy.extend_target = new_target;
                            enemy.state = EnemyState::HolderExtending;
                            continue;
                        }
                        enemy.extend_offset = new_target;
                    } else {
                        // No target, retract and move
                        enemy.state = EnemyState::Moving;
                        enemy.extend_offset = 0.0;
                        enemy.extend_target = 0.0;
                        continue;
                    }

                    // Attack
                    enemy.attack_timer += dt;
                    if enemy.attack_timer >= 0.8 {
                        enemy.attack_timer = 0.0;
                        enemy.attack_flash = 0.15;
                        let reach = enemy.x - enemy.extend_offset;
                        let lane = enemy.lane;
                        let dmg = enemy.damage;
                        if let Some(def) = self
                            .defenders
                            .iter_mut()
                            .filter(|d| {
                                d.lane == lane && {
                                    let dx = Self::grid_origin().x
                                        + d.col as f32 * CELL_W
                                        + CELL_W / 2.0;
                                    dx < reach + CELL_W * 0.5 // only in front of reach point
                                }
                            })
                            .min_by(|a, b| {
                                let ax =
                                    Self::grid_origin().x + a.col as f32 * CELL_W + CELL_W / 2.0;
                                let bx =
                                    Self::grid_origin().x + b.col as f32 * CELL_W + CELL_W / 2.0;
                                (reach - ax).abs().partial_cmp(&(reach - bx).abs()).unwrap()
                            })
                        {
                            def.hp -= dmg;
                        }
                    }
                }
                EnemyState::Retreating => {
                    enemy.x += enemy.speed * dt;
                    if enemy.x > COLS as f32 * CELL_W + SIDEBAR {
                        enemy.hp = 0.0;
                    }
                }
            }

            // Loss conditions
            if enemy.kind != EnemyKind::WhiteOut
                && enemy.state != EnemyState::Retreating
                && !enemy.kind.is_scissors()
                && enemy.x < SIDEBAR
            {
                self.state = GameState::Lost;
                self.save_score();
                return;
            }
            if enemy.kind.is_scissors() && enemy.x < SIDEBAR {
                self.state = GameState::Lost;
                self.save_score();
                return;
            }
        }

        self.enemies.retain(|e| e.alive());
        self.defenders.retain(|d| d.hp > 0.0);
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let focused = ctx.input(|i| i.focused);
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.paused = !focused;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = focused;
        }
        let dt = if self.paused {
            0.0
        } else {
            ctx.input(|i| i.stable_dt).min(0.05)
        };
        self.update(dt);

        ctx.set_visuals(egui::Visuals::light());

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let grid_o = Self::grid_origin();

            // Background
            painter.rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(245, 240, 230));

            // Grid
            for lane in 0..LANES {
                for col in 0..COLS {
                    let r = egui::Rect::from_min_size(
                        egui::pos2(
                            grid_o.x + col as f32 * CELL_W,
                            grid_o.y + lane as f32 * CELL_H,
                        ),
                        egui::vec2(CELL_W, CELL_H),
                    );
                    let c = if (lane + col) % 2 == 0 {
                        egui::Color32::from_rgb(235, 230, 215)
                    } else {
                        egui::Color32::from_rgb(225, 220, 205)
                    };
                    painter.rect_filled(r, 0.0, c);
                    painter.rect_stroke(
                        r,
                        0.0,
                        egui::Stroke::new(0.5, egui::Color32::from_rgb(200, 195, 180)),
                        egui::StrokeKind::Outside,
                    );
                }
            }

            // Top bar
            let bar = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(SIDEBAR + COLS as f32 * CELL_W, TOP_BAR),
            );
            painter.rect_filled(bar, 0.0, egui::Color32::from_rgb(240, 238, 230));
            painter.rect_stroke(
                bar,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 195, 180)),
                egui::StrokeKind::Outside,
            );
            painter.text(
                egui::pos2(15.0, 20.0),
                egui::Align2::LEFT_CENTER,
                format!("Ink: {}", self.ink),
                egui::FontId::proportional(18.0),
                egui::Color32::from_rgb(30, 30, 40),
            );
            painter.text(
                egui::pos2(15.0, 42.0),
                egui::Align2::LEFT_CENTER,
                format!("Wave {}/{}", self.wave, TOTAL_WAVES),
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(100, 100, 110),
            );
            // Timer
            let mins = (self.time / 60.0) as u32;
            let secs = (self.time % 60.0) as u32;
            let time_str = format!("{}:{:02}", mins, secs);
            painter.text(
                egui::pos2(SIDEBAR + COLS as f32 * CELL_W - 15.0, 20.0),
                egui::Align2::RIGHT_CENTER,
                time_str,
                egui::FontId::proportional(18.0),
                egui::Color32::from_rgb(30, 30, 40),
            );
            if let Some(hs) = self.high_score {
                let hm = (hs / 60.0) as u32;
                let hs_s = (hs % 60.0) as u32;
                painter.text(
                    egui::pos2(SIDEBAR + COLS as f32 * CELL_W - 15.0, 42.0),
                    egui::Align2::RIGHT_CENTER,
                    format!("Best: {}:{:02}", hm, hs_s),
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(100, 100, 110),
                );
            }

            // Bonus message
            if self.bonus_msg_timer > 0.0 {
                let alpha = (self.bonus_msg_timer / 1.5 * 255.0) as u8;
                painter.text(
                    egui::pos2(SIDEBAR + COLS as f32 * CELL_W / 2.0, TOP_BAR + 20.0),
                    egui::Align2::CENTER_CENTER,
                    format!("+{} INK", self.bonus_msg_amount),
                    egui::FontId::proportional(22.0),
                    egui::Color32::from_rgba_premultiplied(30, 100, 30, alpha),
                );
            }

            // Sidebar
            let sidebar_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, TOP_BAR),
                egui::vec2(SIDEBAR, LANES as f32 * CELL_H),
            );
            painter.rect_filled(sidebar_rect, 0.0, egui::Color32::from_rgb(215, 210, 200));

            // Defender buttons — sprite + cost
            let kinds = [
                DefenderKind::Pencil,
                DefenderKind::Notebook,
                DefenderKind::Highlighter,
                DefenderKind::Pen,
                DefenderKind::Marker,
            ];
            let btn_h = 55.0_f32;
            let btn_gap = 4.0_f32;
            for (i, &kind) in kinds.iter().enumerate() {
                let btn = egui::Rect::from_min_size(
                    egui::pos2(4.0, TOP_BAR + 4.0 + i as f32 * (btn_h + btn_gap)),
                    egui::vec2(SIDEBAR - 8.0, btn_h),
                );
                let sel = self.selected == Some(Tool::Place(kind));
                let afford = self.ink >= kind.cost();
                let bg = if sel {
                    egui::Color32::from_rgb(190, 210, 245)
                } else if afford {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_rgb(195, 193, 188)
                };
                painter.rect_filled(btn, 4.0, bg);
                painter.rect_stroke(
                    btn,
                    4.0,
                    egui::Stroke::new(
                        if sel { 2.0 } else { 1.0 },
                        if sel {
                            egui::Color32::from_rgb(60, 80, 160)
                        } else {
                            egui::Color32::from_rgb(150, 145, 135)
                        },
                    ),
                    egui::StrokeKind::Outside,
                );
                // Draw sprite (scaled down)
                let sprite_c = egui::pos2(btn.min.x + 28.0, btn.center().y);
                Self::draw_defender_sprite(painter, kind, sprite_c, 1.0);
                // Cost text
                let cost_str = format!("{}", kind.cost());
                painter.text(
                    egui::pos2(btn.max.x - 6.0, btn.center().y),
                    egui::Align2::RIGHT_CENTER,
                    cost_str,
                    egui::FontId::proportional(16.0),
                    if afford {
                        egui::Color32::from_rgb(30, 30, 40)
                    } else {
                        egui::Color32::from_rgb(150, 150, 150)
                    },
                );
                if ui.input(|i| i.pointer.any_pressed()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if btn.contains(pos) {
                            self.selected = if sel { None } else { Some(Tool::Place(kind)) };
                        }
                    }
                }
            }

            // Ink Blob button
            let bottom_y = TOP_BAR + 4.0 + 5.0 * (btn_h + btn_gap);
            let small_h = 42.0;
            let blob_btn = egui::Rect::from_min_size(
                egui::pos2(4.0, bottom_y),
                egui::vec2(SIDEBAR - 8.0, small_h),
            );
            let blob_sel = self.selected == Some(Tool::InkBlob);
            let blob_avail = self.ink_blob_available;
            let blob_bg = if blob_sel {
                egui::Color32::from_rgb(210, 200, 240)
            } else if blob_avail {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(195, 193, 188)
            };
            painter.rect_filled(blob_btn, 4.0, blob_bg);
            painter.rect_stroke(
                blob_btn,
                4.0,
                egui::Stroke::new(
                    if blob_sel { 2.0 } else { 1.0 },
                    if blob_sel {
                        egui::Color32::from_rgb(80, 60, 160)
                    } else {
                        egui::Color32::from_rgb(150, 145, 135)
                    },
                ),
                egui::StrokeKind::Outside,
            );
            // Ink blob splatter sprite
            let bc = egui::pos2(blob_btn.min.x + 24.0, blob_btn.center().y);
            let ink_c = egui::Color32::from_rgb(30, 15, 80);
            painter.circle_filled(bc, 7.0, ink_c);
            painter.circle_filled(egui::pos2(bc.x + 5.0, bc.y - 4.0), 3.0, ink_c);
            painter.circle_filled(egui::pos2(bc.x - 4.0, bc.y + 5.0), 3.5, ink_c);
            painter.circle_filled(egui::pos2(bc.x + 6.0, bc.y + 3.0), 2.5, ink_c);
            // "x1" or "used" label
            painter.text(
                egui::pos2(blob_btn.max.x - 6.0, blob_btn.center().y),
                egui::Align2::RIGHT_CENTER,
                if blob_avail { "x1" } else { "--" },
                egui::FontId::proportional(14.0),
                if blob_avail {
                    egui::Color32::from_rgb(30, 30, 40)
                } else {
                    egui::Color32::from_rgb(150, 150, 150)
                },
            );
            if ui.input(|i| i.pointer.any_pressed()) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if blob_btn.contains(pos) && blob_avail {
                        self.selected = if blob_sel { None } else { Some(Tool::InkBlob) };
                    }
                }
            }

            // Trash button
            let case_y = bottom_y + small_h + btn_gap;
            let case_btn = egui::Rect::from_min_size(
                egui::pos2(4.0, case_y),
                egui::vec2(SIDEBAR - 8.0, small_h),
            );
            let case_sel = self.selected == Some(Tool::PencilCase);
            let case_bg = if case_sel {
                egui::Color32::from_rgb(245, 210, 205)
            } else {
                egui::Color32::WHITE
            };
            painter.rect_filled(case_btn, 4.0, case_bg);
            painter.rect_stroke(
                case_btn,
                4.0,
                egui::Stroke::new(
                    if case_sel { 2.0 } else { 1.0 },
                    if case_sel {
                        egui::Color32::from_rgb(200, 80, 60)
                    } else {
                        egui::Color32::from_rgb(150, 145, 135)
                    },
                ),
                egui::StrokeKind::Outside,
            );
            // Pencil case — zipper on top
            let cc = case_btn.center();
            // Main pouch
            let case_body =
                egui::Rect::from_center_size(egui::pos2(cc.x, cc.y + 2.0), egui::vec2(50.0, 18.0));
            painter.rect_filled(case_body, 5.0, egui::Color32::from_rgb(140, 110, 75));
            painter.rect_stroke(
                case_body,
                5.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(110, 80, 50)),
                egui::StrokeKind::Outside,
            );
            // Top flap (slightly lighter)
            let flap = egui::Rect::from_min_size(
                egui::pos2(case_body.min.x, case_body.min.y - 4.0),
                egui::vec2(50.0, 6.0),
            );
            painter.rect_filled(flap, 3.0, egui::Color32::from_rgb(155, 125, 85));
            // Zipper across the top
            painter.line_segment(
                [
                    egui::pos2(case_body.min.x + 8.0, case_body.min.y - 1.0),
                    egui::pos2(case_body.max.x - 8.0, case_body.min.y - 1.0),
                ],
                egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 180, 50)),
            );
            // Zipper pull tab
            painter.circle_filled(
                egui::pos2(cc.x + 6.0, case_body.min.y - 1.0),
                2.5,
                egui::Color32::from_rgb(210, 190, 60),
            );
            painter.circle_stroke(
                egui::pos2(cc.x + 6.0, case_body.min.y - 1.0),
                2.5,
                egui::Stroke::new(0.5, egui::Color32::from_rgb(160, 140, 30)),
            );
            if ui.input(|i| i.pointer.any_pressed()) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if case_btn.contains(pos) {
                        self.selected = if case_sel {
                            None
                        } else {
                            Some(Tool::PencilCase)
                        };
                    }
                }
            }

            // Grid click handling
            if self.state == GameState::Playing {
                if ui.input(|i| i.pointer.any_pressed()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if pos.x >= grid_o.x && pos.y >= grid_o.y {
                            let col = ((pos.x - grid_o.x) / CELL_W) as usize;
                            let lane = ((pos.y - grid_o.y) / CELL_H) as usize;
                            if col < COLS && lane < LANES {
                                match self.selected {
                                    Some(Tool::Place(kind)) => {
                                        if self.ink >= kind.cost() {
                                            let occ = self
                                                .defenders
                                                .iter()
                                                .any(|d| d.lane == lane && d.col == col);
                                            if !occ {
                                                self.ink -= kind.cost();
                                                self.defenders.push(Defender::new(kind, lane, col));
                                            }
                                        }
                                    }
                                    Some(Tool::InkBlob) => {
                                        if self.ink_blob_available {
                                            self.ink_blob_available = false;
                                            let cell_left = grid_o.x + col as f32 * CELL_W;
                                            let cell_right = cell_left + CELL_W;
                                            for e in &mut self.enemies {
                                                if e.lane == lane
                                                    && e.x >= cell_left
                                                    && e.x <= cell_right
                                                {
                                                    e.hp = 0.0;
                                                }
                                            }
                                            self.selected = None;
                                        }
                                    }
                                    Some(Tool::PencilCase) => {
                                        self.defenders
                                            .retain(|d| !(d.lane == lane && d.col == col));
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                }
            }

            // --- Draw defenders ---
            for def in &self.defenders {
                let c = Self::cell_center(def.col, def.lane);
                Self::draw_defender_sprite(painter, def.kind, c, 1.0);
                // Attack flash
                if def.attack_flash > 0.0 {
                    let alpha = (def.attack_flash / 0.15 * 200.0) as u8;
                    match def.kind {
                        DefenderKind::Pencil => {
                            let sx = c.x + 28.0;
                            painter.line_segment(
                                [
                                    egui::pos2(sx, c.y - 14.0),
                                    egui::pos2(sx + 10.0, c.y + 14.0),
                                ],
                                egui::Stroke::new(
                                    2.5,
                                    egui::Color32::from_rgba_premultiplied(80, 80, 90, alpha),
                                ),
                            );
                        }
                        DefenderKind::Pen => {
                            let sx = c.x + 28.0;
                            painter.line_segment(
                                [
                                    egui::pos2(sx, c.y - 14.0),
                                    egui::pos2(sx + 10.0, c.y + 14.0),
                                ],
                                egui::Stroke::new(
                                    2.5,
                                    egui::Color32::from_rgba_premultiplied(20, 20, 25, alpha),
                                ),
                            );
                        }
                        DefenderKind::Marker => {
                            painter.circle_filled(
                                egui::pos2(c.x + 24.0, c.y),
                                5.0,
                                egui::Color32::from_rgba_premultiplied(255, 100, 50, alpha),
                            );
                        }
                        _ => {}
                    }
                }
            }

            // --- Draw projectiles ---
            for proj in &self.projectiles {
                let y = grid_o.y + proj.lane as f32 * CELL_H + CELL_H / 2.0;
                if proj.is_highlighter {
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(proj.x, y), egui::vec2(14.0, 5.0)),
                        1.0,
                        egui::Color32::from_rgb(240, 255, 50),
                    );
                } else if proj.damage >= 200.0 {
                    // Marker projectile: red ink blob
                    painter.circle_filled(
                        egui::pos2(proj.x, y),
                        6.0,
                        egui::Color32::from_rgb(200, 40, 40),
                    );
                } else {
                    painter.circle_filled(
                        egui::pos2(proj.x, y),
                        4.0,
                        egui::Color32::from_rgb(30, 30, 30),
                    );
                }
            }

            // --- Draw enemies ---
            for enemy in &self.enemies {
                let y = grid_o.y + enemy.lane as f32 * CELL_H + CELL_H / 2.0;
                let x = enemy.x;

                match enemy.kind {
                    EnemyKind::PinkEraser => {
                        let r =
                            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(28.0, 20.0));
                        painter.rect_filled(r, 4.0, egui::Color32::from_rgb(240, 150, 170));
                        painter.rect_stroke(
                            r,
                            4.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 100, 130)),
                            egui::StrokeKind::Outside,
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(6.0, 20.0)),
                            0.0,
                            egui::Color32::from_rgb(220, 120, 145),
                        );
                    }
                    EnemyKind::WhitePolymer => {
                        let r =
                            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(28.0, 20.0));
                        painter.rect_filled(r, 4.0, egui::Color32::from_rgb(240, 240, 245));
                        painter.rect_stroke(
                            r,
                            4.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 190)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    EnemyKind::WrappedPolymer => {
                        if enemy.shell_hp > 0.0 {
                            let outer = egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(34.0, 26.0),
                            );
                            painter.rect_filled(outer, 2.0, egui::Color32::from_rgb(160, 130, 90));
                            painter.rect_stroke(
                                outer,
                                2.0,
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(120, 95, 60)),
                                egui::StrokeKind::Outside,
                            );
                            let inner = egui::Rect::from_center_size(
                                egui::pos2(x, y - 2.0),
                                egui::vec2(26.0, 14.0),
                            );
                            painter.rect_filled(inner, 2.0, egui::Color32::from_rgb(240, 240, 245));
                        } else {
                            let r = egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(28.0, 20.0),
                            );
                            painter.rect_filled(r, 4.0, egui::Color32::from_rgb(240, 240, 245));
                            painter.rect_stroke(
                                r,
                                4.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 190)),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                    EnemyKind::BlackEraser => {
                        let r =
                            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(28.0, 20.0));
                        painter.rect_filled(r, 4.0, egui::Color32::from_rgb(40, 40, 45));
                        painter.rect_stroke(
                            r,
                            4.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 25)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    EnemyKind::WrappedBlackEraser => {
                        if enemy.shell_hp > 0.0 {
                            let outer = egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(36.0, 28.0),
                            );
                            painter.rect_filled(outer, 2.0, egui::Color32::from_rgb(80, 30, 30));
                            painter.rect_stroke(
                                outer,
                                2.0,
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(50, 15, 15)),
                                egui::StrokeKind::Outside,
                            );
                            let inner = egui::Rect::from_center_size(
                                egui::pos2(x, y - 2.0),
                                egui::vec2(28.0, 16.0),
                            );
                            painter.rect_filled(inner, 2.0, egui::Color32::from_rgb(40, 40, 45));
                        } else {
                            let r = egui::Rect::from_center_size(
                                egui::pos2(x, y),
                                egui::vec2(28.0, 20.0),
                            );
                            painter.rect_filled(r, 4.0, egui::Color32::from_rgb(40, 40, 45));
                            painter.rect_stroke(
                                r,
                                4.0,
                                egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 25)),
                                egui::StrokeKind::Outside,
                            );
                        }
                    }
                    EnemyKind::EraserHolder | EnemyKind::BlueEraserHolder => {
                        let barrel = egui::Rect::from_center_size(
                            egui::pos2(x + 4.0, y),
                            egui::vec2(32.0, 10.0),
                        );
                        painter.rect_filled(barrel, 3.0, egui::Color32::from_rgb(35, 35, 40));
                        painter.rect_stroke(
                            barrel,
                            3.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 20, 25)),
                            egui::StrokeKind::Outside,
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                egui::pos2(x + 8.0, y),
                                egui::vec2(4.0, 12.0),
                            ),
                            1.0,
                            egui::Color32::from_rgb(80, 80, 85),
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                egui::pos2(x + 22.0, y),
                                egui::vec2(6.0, 8.0),
                            ),
                            1.0,
                            egui::Color32::from_rgb(60, 60, 65),
                        );
                        let tip_color = if enemy.kind == EnemyKind::BlueEraserHolder {
                            egui::Color32::from_rgb(100, 140, 220)
                        } else {
                            egui::Color32::from_rgb(240, 240, 245)
                        };
                        let ext = enemy.extend_offset;
                        let tip_len = 10.0 + ext;
                        let tip = egui::Rect::from_min_size(
                            egui::pos2(x - 12.0 - ext, y - 4.0),
                            egui::vec2(tip_len, 8.0),
                        );
                        painter.rect_filled(tip, 2.0, tip_color);
                        painter.rect_stroke(
                            tip,
                            2.0,
                            egui::Stroke::new(0.5, egui::Color32::from_rgb(180, 180, 190)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    EnemyKind::Scissors | EnemyKind::BlueScissors => {
                        let (blade_c, handle_c, handle_stroke, sz) =
                            if enemy.kind == EnemyKind::BlueScissors {
                                (
                                    egui::Color32::from_rgb(100, 140, 220),
                                    egui::Color32::from_rgb(50, 80, 180),
                                    egui::Color32::from_rgb(30, 50, 130),
                                    1.2_f32,
                                )
                            } else {
                                (
                                    egui::Color32::from_rgb(180, 180, 190),
                                    egui::Color32::from_rgb(200, 60, 60),
                                    egui::Color32::from_rgb(150, 40, 40),
                                    1.0_f32,
                                )
                            };
                        let s = sz;
                        painter.line_segment(
                            [
                                egui::pos2(x - 14.0 * s, y - 10.0 * s),
                                egui::pos2(x + 14.0 * s, y + 10.0 * s),
                            ],
                            egui::Stroke::new(3.0 * s, blade_c),
                        );
                        painter.line_segment(
                            [
                                egui::pos2(x - 14.0 * s, y + 10.0 * s),
                                egui::pos2(x + 14.0 * s, y - 10.0 * s),
                            ],
                            egui::Stroke::new(3.0 * s, blade_c),
                        );
                        // Handles on RIGHT (rear)
                        painter.circle_filled(
                            egui::pos2(x + 14.0 * s, y - 10.0 * s),
                            5.0 * s,
                            handle_c,
                        );
                        painter.circle_filled(
                            egui::pos2(x + 14.0 * s, y + 10.0 * s),
                            5.0 * s,
                            handle_c,
                        );
                        painter.circle_stroke(
                            egui::pos2(x + 14.0 * s, y - 10.0 * s),
                            5.0 * s,
                            egui::Stroke::new(1.0, handle_stroke),
                        );
                        painter.circle_stroke(
                            egui::pos2(x + 14.0 * s, y + 10.0 * s),
                            5.0 * s,
                            egui::Stroke::new(1.0, handle_stroke),
                        );
                    }
                    EnemyKind::WhiteOut => {
                        let body =
                            egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(30.0, 16.0));
                        painter.rect_filled(body, 3.0, egui::Color32::from_rgb(245, 245, 250));
                        painter.rect_stroke(
                            body,
                            3.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 180, 190)),
                            egui::StrokeKind::Outside,
                        );
                        let tip = vec![
                            egui::pos2(body.min.x, y - 3.0),
                            egui::pos2(body.min.x - 8.0, y),
                            egui::pos2(body.min.x, y + 3.0),
                        ];
                        painter.add(egui::Shape::convex_polygon(
                            tip,
                            egui::Color32::from_rgb(220, 220, 225),
                            egui::Stroke::NONE,
                        ));
                        painter.circle_filled(
                            egui::pos2(x + 4.0, y - 2.0),
                            5.0,
                            egui::Color32::from_rgb(230, 230, 235),
                        );
                        painter.circle_stroke(
                            egui::pos2(x + 4.0, y - 2.0),
                            5.0,
                            egui::Stroke::new(0.5, egui::Color32::from_rgb(180, 180, 190)),
                        );
                        painter.rect_filled(
                            egui::Rect::from_center_size(
                                egui::pos2(x + 2.0, y + 4.0),
                                egui::vec2(16.0, 3.0),
                            ),
                            1.0,
                            egui::Color32::from_rgb(50, 130, 200),
                        );
                    }
                    EnemyKind::KneadedEraser => {
                        let hp_frac = enemy.hp / enemy.kind.base_hp();
                        let w = 24.0 + (1.0 - hp_frac) * 16.0;
                        let h = 20.0 - (1.0 - hp_frac) * 8.0;
                        let r = egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(w, h));
                        painter.rect_filled(r, 8.0, egui::Color32::from_rgb(160, 160, 165));
                        painter.rect_stroke(
                            r,
                            8.0,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 125)),
                            egui::StrokeKind::Outside,
                        );
                    }
                    EnemyKind::ShinyPlastic | EnemyKind::LargeShinyPlastic => {
                        let sz = if enemy.kind == EnemyKind::LargeShinyPlastic {
                            1.5
                        } else {
                            1.0_f32
                        };
                        let r = egui::Rect::from_center_size(
                            egui::pos2(x, y),
                            egui::vec2(26.0 * sz, 18.0 * sz),
                        );
                        painter.rect_filled(
                            r,
                            4.0,
                            egui::Color32::from_rgba_premultiplied(160, 220, 240, 180),
                        );
                        painter.rect_stroke(
                            r,
                            4.0,
                            egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_premultiplied(100, 180, 220, 200),
                            ),
                            egui::StrokeKind::Outside,
                        );
                        painter.line_segment(
                            [
                                egui::pos2(r.min.x + 4.0, r.min.y + 3.0),
                                egui::pos2(r.max.x - 8.0, r.min.y + 3.0),
                            ],
                            egui::Stroke::new(
                                1.5,
                                egui::Color32::from_rgba_premultiplied(255, 255, 255, 140),
                            ),
                        );
                    }
                    EnemyKind::ElectronicEraser => {
                        // Toothbrush-shaped body (handle)
                        let handle = egui::Rect::from_center_size(
                            egui::pos2(x + 4.0, y),
                            egui::vec2(24.0, 8.0),
                        );
                        painter.rect_filled(handle, 3.0, egui::Color32::from_rgb(180, 180, 190));
                        painter.rect_stroke(
                            handle,
                            3.0,
                            egui::Stroke::new(0.5, egui::Color32::from_rgb(140, 140, 150)),
                            egui::StrokeKind::Outside,
                        );
                        // Neck (thinner part connecting handle to head)
                        let neck = egui::Rect::from_center_size(
                            egui::pos2(x - 10.0, y),
                            egui::vec2(6.0, 6.0),
                        );
                        painter.rect_filled(neck, 1.0, egui::Color32::from_rgb(170, 170, 180));
                        // Fan-shaped eraser tip (semicircle / fan)
                        let fan_cx = x - 16.0;
                        let fan_r = 9.0;
                        // Draw fan as filled semicircle facing left using triangle segments
                        let fan_color = egui::Color32::from_rgb(60, 60, 70);
                        let steps = 8;
                        for i in 0..steps {
                            let a1 = std::f32::consts::PI * 0.5
                                + std::f32::consts::PI * (i as f32 / steps as f32);
                            let a2 = std::f32::consts::PI * 0.5
                                + std::f32::consts::PI * ((i + 1) as f32 / steps as f32);
                            let tri = vec![
                                egui::pos2(fan_cx, y),
                                egui::pos2(fan_cx + fan_r * a1.cos(), y + fan_r * a1.sin()),
                                egui::pos2(fan_cx + fan_r * a2.cos(), y + fan_r * a2.sin()),
                            ];
                            painter.add(egui::Shape::convex_polygon(
                                tri,
                                fan_color,
                                egui::Stroke::NONE,
                            ));
                        }
                        // Fan edge outline
                        painter.circle_stroke(
                            egui::pos2(fan_cx, y),
                            fan_r,
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 50)),
                        );
                        // Button on handle
                        painter.circle_filled(
                            egui::pos2(x + 10.0, y),
                            2.5,
                            egui::Color32::from_rgb(50, 180, 230),
                        );
                        // Power indicator line
                        painter.line_segment(
                            [egui::pos2(x + 2.0, y - 3.0), egui::pos2(x + 2.0, y + 3.0)],
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 200, 240)),
                        );
                    }
                }

                // Hit flash
                if enemy.hit_flash > 0.0 {
                    let alpha = (enemy.hit_flash / 0.15 * 200.0) as u8;
                    painter.line_segment(
                        [egui::pos2(x - 2.0, y - 14.0), egui::pos2(x + 6.0, y + 14.0)],
                        egui::Stroke::new(
                            2.5,
                            egui::Color32::from_rgba_premultiplied(60, 60, 70, alpha),
                        ),
                    );
                }

                // Eraser attack flash
                if enemy.attack_flash > 0.0 && !enemy.kind.instant_kills() {
                    let alpha = (enemy.attack_flash / 0.15 * 200.0) as u8;
                    painter.line_segment(
                        [
                            egui::pos2(x - 18.0, y - 12.0),
                            egui::pos2(x - 5.0, y + 12.0),
                        ],
                        egui::Stroke::new(
                            2.5,
                            egui::Color32::from_rgba_premultiplied(220, 20, 20, alpha),
                        ),
                    );
                }
            }

            // Pause overlay
            if self.paused && self.state == GameState::Playing {
                let screen = egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(
                        SIDEBAR + COLS as f32 * CELL_W,
                        TOP_BAR + LANES as f32 * CELL_H,
                    ),
                );
                painter.rect_filled(
                    screen,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 100),
                );
                painter.text(
                    screen.center(),
                    egui::Align2::CENTER_CENTER,
                    "Paused",
                    egui::FontId::proportional(40.0),
                    egui::Color32::WHITE,
                );
            }

            // Game over / win
            if self.state != GameState::Playing {
                let screen = egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(
                        SIDEBAR + COLS as f32 * CELL_W,
                        TOP_BAR + LANES as f32 * CELL_H,
                    ),
                );
                let overlay_color = if self.state == GameState::Won {
                    egui::Color32::from_rgba_premultiplied(255, 255, 255, 180)
                } else {
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 140)
                };
                painter.rect_filled(screen, 0.0, overlay_color);
                let msg = if self.state == GameState::Won {
                    "You Win!"
                } else {
                    "Game Over"
                };
                painter.text(
                    screen.center(),
                    egui::Align2::CENTER_CENTER,
                    msg,
                    egui::FontId::proportional(48.0),
                    egui::Color32::WHITE,
                );
                // Show time
                let mins = (self.time / 60.0) as u32;
                let secs = (self.time % 60.0) as u32;
                let sub_color = if self.state == GameState::Won {
                    egui::Color32::from_rgb(60, 60, 70)
                } else {
                    egui::Color32::from_rgb(200, 200, 210)
                };
                painter.text(
                    egui::pos2(screen.center().x, screen.center().y + 40.0),
                    egui::Align2::CENTER_CENTER,
                    format!("Time: {}:{:02}", mins, secs),
                    egui::FontId::proportional(22.0),
                    sub_color,
                );
                if let Some(hs) = self.high_score {
                    let hm = (hs / 60.0) as u32;
                    let hs_s = (hs % 60.0) as u32;
                    painter.text(
                        egui::pos2(screen.center().x, screen.center().y + 65.0),
                        egui::Align2::CENTER_CENTER,
                        format!("Best: {}:{:02}", hm, hs_s),
                        egui::FontId::proportional(16.0),
                        sub_color,
                    );
                }
                painter.text(
                    egui::pos2(screen.center().x, screen.center().y + 90.0),
                    egui::Align2::CENTER_CENTER,
                    "Click to restart",
                    egui::FontId::proportional(18.0),
                    sub_color,
                );
                if ui.input(|i| i.pointer.any_pressed()) {
                    *self = Game::new();
                }
            }
        });

        ctx.request_repaint();
    }
}
